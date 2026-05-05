//! Integration tests that exercise the multiplexed-session paths against a
//! running Spanner emulator (or Spanner Omni). The standard Cloud Spanner
//! emulator did not historically support multiplexed sessions; if the
//! emulator at SPANNER_EMULATOR_HOST rejects `multiplexed: true` these tests
//! will fail loudly rather than silently passing on a non-multiplexed path.
//!
//! All tests build a Client with `session_config.use_multiplexed_session =
//! true` so the multiplexed code path is the one under test.

use serial_test::serial;
use time::OffsetDateTime;

use common::*;
use gcloud_spanner::client::{ChannelConfig, Client, ClientConfig};
use gcloud_spanner::session::SessionConfig;
use gcloud_spanner::statement::Statement;
use google_cloud_gax::conn::Environment;

mod common;

const DATABASE: &str = "projects/local-project/instances/test-instance/databases/local-database";
const ENDPOINT: &str = "localhost:9010";

#[ctor::ctor]
fn init() {
    let filter = tracing_subscriber::filter::EnvFilter::from_default_env()
        .add_directive("google_cloud_spanner=trace".parse().unwrap());
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
    std::env::set_var("SPANNER_EMULATOR_HOST", ENDPOINT);
}

async fn multiplexed_client() -> Client {
    let mut session_config = SessionConfig::default();
    session_config.use_multiplexed_session = true;
    session_config.min_opened = 0;
    session_config.max_opened = 1;
    Client::new(
        DATABASE,
        ClientConfig {
            session_config,
            environment: Environment::Emulator(ENDPOINT.to_string()),
            channel_config: ChannelConfig {
                num_channels: 1,
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .await
    .unwrap()
}

/// Inline-begin path: the very first op of a RW transaction is a query.
/// Verifies that tx_id is captured from the first PartialResultSet's metadata
/// and that commit succeeds.
#[tokio::test]
#[serial]
async fn test_multiplexed_inline_begin_query_then_commit() {
    let now = OffsetDateTime::now_utc();
    let user_id = format!("mux_q_{}", now.unix_timestamp_nanos());
    let bootstrap = create_data_client().await;
    bootstrap
        .apply(vec![create_user_mutation(&user_id, &now)])
        .await
        .unwrap();

    let client = multiplexed_client().await;
    let user_id_clone = user_id.clone();
    let result = client
        .read_write_transaction(|tx| {
            let user_id = user_id_clone.clone();
            Box::pin(async move {
                let mut stmt = Statement::new("SELECT NotNullINT64 FROM User WHERE UserId = @uid");
                stmt.add_param("uid", &user_id);
                let mut iter = tx.query(stmt).await?;
                let row = iter.next().await?.expect("row");
                let v: i64 = row.column::<i64>(0)?;
                Ok::<_, gcloud_spanner::client::Error>(v)
            })
        })
        .await;
    let (_cr, v) = result.unwrap();
    assert_eq!(v, 1);
}

/// Mutations-only commit: no read/DML fires, so `commit()` must call
/// `begin_for_mutations_only` (explicit BeginTransaction with mutation_key)
/// and forward the precommit_token from that response on Commit.
#[tokio::test]
#[serial]
async fn test_multiplexed_mutations_only_commit() {
    let now = OffsetDateTime::now_utc();
    let user_id = format!("mux_m_{}", now.unix_timestamp_nanos());
    let client = multiplexed_client().await;
    // Goes through read_write_transaction_sync_with_option -> begin (inline)
    // -> buffer_write -> commit -> begin_for_mutations_only.
    let cr = client
        .apply(vec![create_user_mutation(&user_id, &now)])
        .await
        .unwrap();
    assert!(cr.timestamp.is_some());
}

/// Empty transaction: begin + immediate commit with nothing buffered.
/// Verifies the empty-tx_id guard returns a synthetic empty CommitResponse
/// instead of shipping `TransactionId(empty)` to the server.
#[tokio::test]
#[serial]
async fn test_multiplexed_empty_transaction_commits_cleanly() {
    let client = multiplexed_client().await;
    let result = client
        .read_write_transaction(|_tx| {
            Box::pin(async move { Ok::<(), gcloud_spanner::client::Error>(()) })
        })
        .await;
    let (_cr, _) = result.unwrap();
}

/// Multiple ops on the same multiplexed RW transaction: the first op
/// captures tx_id; subsequent ops should reuse it (no second BeginTransaction
/// or constructor-side pre-fetch divergence).
#[tokio::test]
#[serial]
async fn test_multiplexed_multi_op_transaction() {
    let now = OffsetDateTime::now_utc();
    let user_id = format!("mux_x_{}", now.unix_timestamp_nanos());
    let bootstrap = create_data_client().await;
    bootstrap
        .apply(vec![create_user_mutation(&user_id, &now)])
        .await
        .unwrap();

    let client = multiplexed_client().await;
    let user_id_clone = user_id.clone();
    let result = client
        .read_write_transaction(|tx| {
            let user_id = user_id_clone.clone();
            Box::pin(async move {
                let mut stmt1 = Statement::new("SELECT NotNullINT64 FROM User WHERE UserId = @uid");
                stmt1.add_param("uid", &user_id);
                let mut iter1 = tx.query(stmt1).await?;
                let _ = iter1.next().await?;

                let mut stmt2 = Statement::new("SELECT NotNullFloat64 FROM User WHERE UserId = @uid");
                stmt2.add_param("uid", &user_id);
                let mut iter2 = tx.query(stmt2).await?;
                let row = iter2.next().await?.expect("row");
                let v: f64 = row.column::<f64>(0)?;
                Ok::<_, gcloud_spanner::client::Error>(v)
            })
        })
        .await;
    let (_cr, v) = result.unwrap();
    assert_eq!(v, 1.0);
}

/// `apply_at_least_once` on a multiplexed session must route through the RW
/// transaction path (single-use commits with mutations are not supported on
/// multiplexed sessions: server rejects with UNIMPLEMENTED).
#[tokio::test]
#[serial]
async fn test_multiplexed_apply_at_least_once_routes_through_rw() {
    let now = OffsetDateTime::now_utc();
    let user_id = format!("mux_aalo_{}", now.unix_timestamp_nanos());
    let client = multiplexed_client().await;
    let result = client
        .apply_at_least_once(vec![create_user_mutation(&user_id, &now)])
        .await
        .unwrap();
    assert!(result.is_some());
}

/// `partitioned_update` on a multiplexed session must take the explicit
/// BeginTransaction path. ExecuteSql with `Begin(PartitionedDml)` on a
/// multiplexed session is rejected (NOT_FOUND); the fix gates inline begin
/// on `mode == ReadWrite`.
#[tokio::test]
#[serial]
async fn test_multiplexed_partitioned_update() {
    let client = multiplexed_client().await;
    let stmt = Statement::new("DELETE FROM Guild WHERE GuildId = \"never_exists_mux\"");
    let n = client.partitioned_update(stmt).await.unwrap();
    assert_eq!(n, 0);
}

/// DML followed by buffered mutations in the same RW transaction. Exercises
/// the inline-begin tx_id capture from an ExecuteSql response (different
/// site than the streaming-read or mutations-only paths) followed by a
/// commit that ships both the buffered mutations and the captured
/// precommit_token.
#[tokio::test]
#[serial]
async fn test_multiplexed_dml_and_mutations() {
    let now = OffsetDateTime::now_utc();
    let user_id = format!("mux_dml_{}", now.unix_timestamp_nanos());
    let bootstrap = create_data_client().await;
    bootstrap
        .apply(vec![create_user_mutation(&user_id, &now)])
        .await
        .unwrap();

    let client = multiplexed_client().await;
    let user_id_clone = user_id.clone();
    let result = client
        .read_write_transaction(|tx| {
            let user_id = user_id_clone.clone();
            Box::pin(async move {
                // First op: an UPDATE — captures tx_id via ExecuteSql metadata
                // and populates the precommit_token slot from the DML response.
                let mut stmt = Statement::new("UPDATE User SET NullableINT64 = 7 WHERE UserId = @uid");
                stmt.add_param("uid", &user_id);
                let updated = tx.update(stmt).await?;
                // Second op: buffer a mutation under the same tx. Commit must
                // ship the mutation against the captured tx_id (Id selector,
                // not Begin) along with the precommit_token from the DML.
                tx.buffer_write(vec![create_user_item_mutation(&user_id, 99)]);
                Ok::<_, gcloud_spanner::client::Error>(updated)
            })
        })
        .await;
    let (_cr, updated) = result.unwrap();
    assert_eq!(updated, 1);
}
