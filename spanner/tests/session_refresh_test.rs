use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serial_test::serial;
use time::OffsetDateTime;
use tokio::time::{timeout, Duration};

use common::*;
use google_cloud_gax::conn::{ConnectionOptions, Environment};
use google_cloud_googleapis::spanner::v1::{DeleteSessionRequest, ListSessionsRequest};
use google_cloud_spanner::apiv1::conn_pool::{ConnectionManager, SPANNER};
use google_cloud_spanner::apiv1::spanner_client::Client as LowLevelClient;
use google_cloud_spanner::client::Error as ClientError;
use google_cloud_spanner::key::Key;
use google_cloud_spanner::statement::Statement;

mod common;

const DATABASE: &str = "projects/local-project/instances/test-instance/databases/local-database";

#[ctor::ctor]
fn init() {
    let filter = tracing_subscriber::filter::EnvFilter::from_default_env()
        .add_directive("google_cloud_spanner=trace".parse().unwrap());
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
    std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
}

#[tokio::test]
#[serial]
async fn single_read_recovers_from_deleted_session() {
    purge_sessions().await;
    let client = create_data_client().await;

    let now = OffsetDateTime::now_utc();
    let user_id = format!("sess_user_{}", now.unix_timestamp());
    let ms = vec![create_user_mutation(&user_id, &now)];
    client.apply(ms).await.unwrap();

    let mut single = client.single().await.unwrap();

    delete_only_session().await;

    let read_result = timeout(Duration::from_secs(30), single.read("User", &["UserId"], Key::new(&user_id)))
        .await
        .expect("read timed out");
    assert!(
        read_result.is_ok(),
        "expected client to transparently replace a deleted session, but saw: {:?}",
        read_result.err()
    );
}

#[tokio::test]
#[serial]
async fn single_query_recovers_from_deleted_session() {
    purge_sessions().await;
    let client = create_data_client().await;

    let now = OffsetDateTime::now_utc();
    let user_id = format!("sess_query_user_{}", now.unix_timestamp());
    let ms = vec![create_user_mutation(&user_id, &now)];
    client.apply(ms).await.unwrap();

    let mut single = client.single().await.unwrap();

    delete_only_session().await;

    let mut stmt = Statement::new("SELECT UserId FROM User WHERE UserId = @UserId");
    stmt.add_param("UserId", &user_id);

    let mut iter = timeout(Duration::from_secs(30), single.query(stmt))
        .await
        .expect("query timed out")
        .expect("query failed after session refresh");

    let row = timeout(Duration::from_secs(30), iter.next())
        .await
        .expect("row fetch timed out")
        .expect("row fetch failed")
        .expect("expected at least one row");

    let fetched_user_id = row.column_by_name::<String>("UserId").unwrap();
    assert_eq!(
        fetched_user_id, user_id,
        "expected iterator to surface the inserted row after session refresh"
    );
}

#[tokio::test]
#[serial]
async fn read_write_tx_recovers_from_deleted_session() {
    purge_sessions().await;
    let client = create_data_client().await;

    let now = OffsetDateTime::now_utc();
    let user_id = format!("rw_sess_user_{}", now.unix_timestamp());
    let ms = vec![create_user_mutation(&user_id, &now)];
    client.apply(ms).await.unwrap();

    let delete_once = Arc::new(AtomicBool::new(false));
    let tx_result = timeout(
        Duration::from_secs(30),
        client.read_write_transaction({
            let delete_once = delete_once.clone();
            move |tx| {
                let user_id = user_id.clone();
                let delete_once = delete_once.clone();
                Box::pin(async move {
                    if !delete_once.swap(true, Ordering::SeqCst) {
                        delete_only_session().await;
                    }
                    let mut reader = tx.read("User", &["UserId"], vec![Key::new(&user_id)]).await?;
                    while reader.next().await?.is_some() {}
                    Ok::<(), ClientError>(())
                })
            }
        }),
    )
    .await
    .expect("transaction timed out");

    assert!(
        tx_result.is_ok(),
        "expected read_write_transaction to replace a deleted session automatically, saw: {:?}",
        tx_result.err()
    );
}

#[tokio::test]
#[serial]
async fn read_only_tx_recovers_from_deleted_session() {
    purge_sessions().await;
    let client = create_data_client().await;

    let now = OffsetDateTime::now_utc();
    let user_id = format!("ro_sess_user_{}", now.unix_timestamp());
    let ms = vec![create_user_mutation(&user_id, &now)];
    client.apply(ms).await.unwrap();

    delete_only_session().await;

    let mut tx = client.read_only_transaction().await.unwrap();

    let mut reader = timeout(Duration::from_secs(30), tx.read("User", &["UserId"], vec![Key::new(&user_id)]))
        .await
        .expect("read-only read timed out")
        .expect("read-only read failed after session refresh");

    let row = timeout(Duration::from_secs(30), reader.next())
        .await
        .expect("read-only iterator timed out")
        .expect("read-only iterator failed")
        .expect("expected at least one row");

    let fetched_user_id = row.column_by_name::<String>("UserId").unwrap();
    assert_eq!(
        fetched_user_id, user_id,
        "expected read_only_transaction to surface the inserted row after session refresh"
    );
}

#[tokio::test]
#[serial]
async fn partitioned_update_recovers_from_deleted_session() {
    purge_sessions().await;
    let client = create_data_client().await;

    let now = OffsetDateTime::now_utc();
    let user_id = format!("pdml_sess_user_{}", now.unix_timestamp());
    let ms = vec![create_user_mutation(&user_id, &now)];
    client.apply(ms).await.unwrap();

    delete_only_session().await;

    let mut stmt = Statement::new("UPDATE User SET NullableString = @Value WHERE UserId = @UserId");
    let updated_value = format!("updated {}", user_id);
    stmt.add_param("UserId", &user_id);
    stmt.add_param("Value", &updated_value);

    let updated_rows = timeout(Duration::from_secs(30), client.partitioned_update(stmt))
        .await
        .expect("partitioned_update timed out")
        .expect("partitioned_update failed after session refresh");

    assert_eq!(
        updated_rows, 1,
        "partitioned_update should modify the inserted row even after refreshing the session"
    );
}

#[tokio::test]
#[serial]
async fn batch_partition_read_recovers_from_deleted_session() {
    purge_sessions().await;
    let client = create_data_client().await;

    let now = OffsetDateTime::now_utc();
    let user_id = format!("batch_sess_user_{}", now.unix_timestamp());
    let ms = vec![create_user_mutation(&user_id, &now)];
    client.apply(ms).await.unwrap();

    let mut batch_tx = client.batch_read_only_transaction().await.unwrap();

    delete_only_session().await;

    let partitions = timeout(
        Duration::from_secs(30),
        batch_tx.partition_read("User", &["UserId"], vec![Key::new(&user_id)]),
    )
    .await
    .expect("partition_read timed out")
    .expect("partition_read failed after session refresh");

    assert!(!partitions.is_empty(), "expected at least one partition to be returned");

    let mut found_row = false;
    for partition in partitions {
        let mut rows = timeout(Duration::from_secs(30), batch_tx.execute(partition, None))
            .await
            .expect("batch execute timed out")
            .expect("batch execute failed after session refresh");
        loop {
            let maybe_row = timeout(Duration::from_secs(30), rows.next())
                .await
                .expect("partition row fetch timed out")
                .expect("partition row fetch failed");
            match maybe_row {
                Some(row) => {
                    let fetched_user_id = row.column_by_name::<String>("UserId").unwrap();
                    if fetched_user_id == user_id {
                        found_row = true;
                        break;
                    }
                }
                None => break,
            }
        }
        if found_row {
            break;
        }
    }
    assert!(
        found_row,
        "expected batch partition read to surface the inserted row after session refresh"
    );
}

async fn delete_only_session() {
    let mut raw = new_spanner_client().await;
    let response = raw
        .list_sessions(
            ListSessionsRequest {
                database: DATABASE.to_string(),
                page_size: 0,
                page_token: String::new(),
                filter: String::new(),
            },
            None,
        )
        .await
        .expect("list sessions");
    let sessions = response.into_inner().sessions;
    assert!(
        !sessions.is_empty(),
        "expected at least one session to be present before deletion"
    );
    raw.delete_session(
        DeleteSessionRequest {
            name: sessions[0].name.clone(),
        },
        None,
    )
    .await
    .expect("delete session");
}

async fn purge_sessions() {
    let mut raw = new_spanner_client().await;
    let mut page_token = String::new();
    loop {
        let response = raw
            .list_sessions(
                ListSessionsRequest {
                    database: DATABASE.to_string(),
                    page_size: 0,
                    page_token: page_token.clone(),
                    filter: String::new(),
                },
                None,
            )
            .await
            .expect("list sessions");
        let inner = response.into_inner();
        for session in inner.sessions {
            let _ = raw
                .delete_session(DeleteSessionRequest { name: session.name }, None)
                .await;
        }
        if inner.next_page_token.is_empty() {
            break;
        }
        page_token = inner.next_page_token;
    }
}

async fn new_spanner_client() -> LowLevelClient {
    let host = std::env::var("SPANNER_EMULATOR_HOST").unwrap_or_else(|_| "localhost:9010".to_string());
    let cm = ConnectionManager::new(1, &Environment::Emulator(host), SPANNER, &ConnectionOptions::default())
        .await
        .expect("create spanner connection manager");
    cm.conn()
}
