//! Integration tests against a local **Spanner Omni** container.
//!
//! Omni is a full Spanner server in a single container. Unlike the Cloud Spanner emulator
//! it accepts **only multiplexed sessions**, so a client that creates regular sessions
//! fails at startup with:
//!
//! ```text
//! InvalidArgument: Please use multiplexed sessions.
//! Only Multiplexed sessions are supported in this environment.
//! ```
//!
//! [`regular_sessions_are_rejected_by_omni`] pins that failure, and
//! [`multiplexed_sessions_work`] walks every code path multiplexed sessions change:
//!
//! 1. session creation (`CreateSession { multiplexed: true }`)
//! 2. a DML-only read-write transaction        -> precommit token from `ExecuteSql`
//! 3. a batch-DML read-write transaction       -> precommit token from `ExecuteBatchDml`
//! 4. a streaming query in a read-write transaction -> token from `PartialResultSet`
//! 5. a mutation-only read-write transaction   -> needs `mutation_key` on `BeginTransaction`
//! 6. a single-use read-only transaction
//!
//! Both tests are `#[ignore]`d because they need the container, and neither reads any
//! environment variable. From the `spanner/` directory:
//!
//! ```sh
//! docker compose -f docker-compose.omni.yml up -d
//! docker compose -f docker-compose.omni.yml wait spanner-omni-init
//! cargo test -p gcloud-spanner --test omni_test -- --ignored --nocapture
//! ```
//!
//! Omni serves a single implicit instance and ignores the project/instance segments of the
//! database path on the data plane, so `DATABASE` keeps the shape used against real Cloud
//! Spanner.

use gcloud_spanner::client::{ChannelConfig, Client, ClientConfig, Error};
use gcloud_spanner::key::Key;
use gcloud_spanner::mutation::insert_or_update;
use gcloud_spanner::statement::Statement;
use gcloud_spanner::value::CommitTimestamp;
use google_cloud_gax::conn::Environment;
use google_cloud_gax::grpc::Code;

const DATABASE: &str = "projects/local-project/instances/test-instance/databases/local-database";
/// Matches the default `SPANNER_OMNI_PORT` in docker-compose.omni.yml.
const OMNI_HOST: &str = "localhost:9011";

const OWNER_ID: &str = "omni-owner";

const NEEDS_OMNI: &str = "requires a local Spanner Omni container; see spanner/docker-compose.omni.yml";

/// A client pointed at Omni. `Environment::Emulator` is what skips auth, so nothing here
/// depends on `SPANNER_EMULATOR_HOST` or any other variable being set.
fn omni_config(multiplexed: bool) -> ClientConfig {
    let mut config = ClientConfig {
        environment: Environment::Emulator(OMNI_HOST.to_string()),
        channel_config: ChannelConfig {
            num_channels: 1,
            ..Default::default()
        },
        ..Default::default()
    };
    config.session_config.min_opened = 1;
    config.session_config.max_opened = 4;
    config.session_config.multiplexed = multiplexed;
    config
}

/// The bug this feature fixes: without multiplexed sessions the client cannot even connect.
#[tokio::test]
#[ignore = "requires a local Spanner Omni container; see spanner/docker-compose.omni.yml"]
async fn regular_sessions_are_rejected_by_omni() {
    let result = Client::new(DATABASE, omni_config(false)).await;

    let Err(Error::GRPC(status)) = result else {
        panic!("expected Omni to reject regular sessions, got {:?}", result.err());
    };
    assert_eq!(
        status.code(),
        Code::InvalidArgument,
        "unexpected status: {status:?} ({NEEDS_OMNI})"
    );
    assert!(
        status.message().to_lowercase().contains("multiplexed"),
        "expected the rejection to mention multiplexed sessions, got: {}",
        status.message()
    );
}

/// The same client with `multiplexed = true` drives every affected path end to end.
#[tokio::test]
#[ignore = "requires a local Spanner Omni container; see spanner/docker-compose.omni.yml"]
async fn multiplexed_sessions_work() -> Result<(), Error> {
    // Step 1: session creation. This is where the unmodified client dies.
    let client = Client::new(DATABASE, omni_config(true)).await?;

    // Step 2: DML-only read-write transaction. The precommit token arrives on the
    // ExecuteSql response.
    client
        .read_write_transaction::<(), Error, _>(|tx| {
            Box::pin(async move {
                let mut stmt = Statement::new(
                    "INSERT OR UPDATE INTO Guild (GuildId, OwnerUserId, UpdatedAt) \
                     VALUES (@GuildId, @OwnerUserId, PENDING_COMMIT_TIMESTAMP())",
                );
                stmt.add_param("GuildId", &"omni-guild-dml");
                stmt.add_param("OwnerUserId", &OWNER_ID);
                assert_eq!(tx.update(stmt).await?, 1);
                Ok(())
            })
        })
        .await?;

    // Step 3: batch DML. The precommit token arrives on the ExecuteBatchDml response.
    client
        .read_write_transaction::<(), Error, _>(|tx| {
            Box::pin(async move {
                let stmts = (0..2)
                    .map(|i| {
                        let mut stmt = Statement::new(
                            "INSERT OR UPDATE INTO Guild (GuildId, OwnerUserId, UpdatedAt) \
                             VALUES (@GuildId, @OwnerUserId, PENDING_COMMIT_TIMESTAMP())",
                        );
                        stmt.add_param("GuildId", &format!("omni-guild-batch-{i}"));
                        stmt.add_param("OwnerUserId", &OWNER_ID);
                        stmt
                    })
                    .collect();
                assert_eq!(tx.batch_update(stmts).await?, vec![1, 1]);
                Ok(())
            })
        })
        .await?;

    // Step 4: streaming query inside a read-write transaction, then a mutation. Tokens
    // ride on PartialResultSet here, and the iterator holds the session borrow while they
    // arrive.
    let (_, found) = client
        .read_write_transaction::<usize, Error, _>(|tx| {
            Box::pin(async move {
                let mut stmt = Statement::new("SELECT GuildId FROM Guild WHERE OwnerUserId = @OwnerUserId");
                stmt.add_param("OwnerUserId", &OWNER_ID);
                let mut iter = tx.query(stmt).await?;
                let mut ids = vec![];
                while let Some(row) = iter.next().await? {
                    ids.push(row.column_by_name::<String>("GuildId")?);
                }
                drop(iter);
                tx.buffer_write(vec![insert_or_update(
                    "Guild",
                    &["GuildId", "OwnerUserId", "UpdatedAt"],
                    &[&"omni-guild-after-query", &OWNER_ID, &CommitTimestamp::new()],
                )]);
                Ok(ids.len())
            })
        })
        .await?;
    assert!(found >= 3, "steps 2 and 3 wrote three rows, query saw {found}");

    // Step 5: mutation-only read-write transaction. No statement ever runs, so the only
    // precommit token available is the one BeginTransaction returns -- and it only returns
    // one when a `mutation_key` was supplied.
    let guild_id = "omni-guild-mutation-only";
    client
        .read_write_transaction::<(), Error, _>(|tx| {
            Box::pin(async move {
                tx.buffer_write(vec![insert_or_update(
                    "Guild",
                    &["GuildId", "OwnerUserId", "UpdatedAt"],
                    &[&guild_id, &OWNER_ID, &CommitTimestamp::new()],
                )]);
                Ok(())
            })
        })
        .await?;

    // Step 6: read it back with a single-use read-only transaction.
    let mut tx = client.single().await?;
    let row = tx
        .read_row("Guild", &["GuildId", "OwnerUserId"], Key::new(&guild_id))
        .await?
        .expect("the row written in step 5 should be readable");
    assert_eq!(row.column_by_name::<String>("GuildId")?, guild_id);
    assert_eq!(row.column_by_name::<String>("OwnerUserId")?, OWNER_ID);

    client.close().await;
    Ok(())
}
