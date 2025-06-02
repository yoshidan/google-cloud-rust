use std::time::Duration;
use tokio::time::sleep;

use serial_test::serial;
use time::OffsetDateTime;
use tokio::task::JoinHandle;

use google_cloud_gax::conn::Environment;
use google_cloud_gax::conn::Environment::GoogleCloud;
use google_cloud_gax::grpc::Code;
use google_cloud_googleapis::spanner::admin::database::v1::UpdateDatabaseDdlRequest;

use common::*;
use google_cloud_spanner::admin;
use google_cloud_spanner::admin::AdminClientConfig;
use google_cloud_spanner::client::{Client, ClientConfig};

use google_cloud_spanner::reader::{Reader, RowIterator};
use google_cloud_spanner::row::{Error, Row, Struct, TryFromStruct};
use google_cloud_spanner::statement::Statement;
use google_cloud_spanner::transaction::QueryOptions;
use google_cloud_spanner::transaction_ro::ReadOnlyTransaction;

mod common;

#[ctor::ctor]
fn init() {
    let filter = tracing_subscriber::filter::EnvFilter::from_default_env()
        .add_directive("google_cloud_spanner=trace".parse().unwrap());
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

#[allow(dead_code)]
#[derive(Debug)]
struct ChangeRecord {
    pub data_change_record: Vec<DataChangeRecord>,
    pub child_partitions_record: Vec<ChildPartitionsRecord>,
}

impl TryFromStruct for ChangeRecord {
    fn try_from_struct(s: Struct<'_>) -> Result<Self, Error> {
        Ok(Self {
            data_change_record: s.column_by_name("data_change_record")?,
            child_partitions_record: s.column_by_name("child_partitions_record")?,
        })
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct ChildPartitionsRecord {
    pub start_timestamp: OffsetDateTime,
    pub record_sequence: String,
    pub child_partitions: Vec<ChildPartition>,
}

impl TryFromStruct for ChildPartitionsRecord {
    fn try_from_struct(s: Struct<'_>) -> Result<Self, Error> {
        Ok(Self {
            start_timestamp: s.column_by_name("start_timestamp")?,
            record_sequence: s.column_by_name("record_sequence")?,
            child_partitions: s.column_by_name("child_partitions")?,
        })
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct ChildPartition {
    pub token: String,
    pub parent_partition_tokens: Vec<String>,
}

impl TryFromStruct for ChildPartition {
    fn try_from_struct(s: Struct<'_>) -> Result<Self, Error> {
        Ok(Self {
            token: s.column_by_name("token")?,
            parent_partition_tokens: s.column_by_name("parent_partition_tokens")?,
        })
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct DataChangeRecord {
    pub commit_timestamp: OffsetDateTime,
    pub record_sequence: String,
    pub server_transaction_id: String,
    pub is_last_record_in_transaction_in_partition: bool,
    pub table_name: String,
    pub mod_type: String,
    pub value_capture_type: String,
    pub number_of_records_in_transaction: i64,
    pub number_of_partitions_in_transaction: i64,
    pub transaction_tag: String,
    pub is_system_transaction: bool,
}
impl TryFromStruct for DataChangeRecord {
    fn try_from_struct(s: Struct<'_>) -> Result<Self, Error> {
        Ok(Self {
            commit_timestamp: s.column_by_name("commit_timestamp")?,
            record_sequence: s.column_by_name("record_sequence")?,
            server_transaction_id: s.column_by_name("server_transaction_id")?,
            is_last_record_in_transaction_in_partition: s
                .column_by_name("is_last_record_in_transaction_in_partition")?,
            table_name: s.column_by_name("table_name")?,
            mod_type: s.column_by_name("mod_type")?,
            value_capture_type: s.column_by_name("value_capture_type")?,
            number_of_records_in_transaction: s.column_by_name("number_of_records_in_transaction")?,
            number_of_partitions_in_transaction: s.column_by_name("number_of_partitions_in_transaction")?,
            transaction_tag: s.column_by_name("transaction_tag")?,
            is_system_transaction: s.column_by_name("is_system_transaction")?,
        })
    }
}

async fn create_environment() -> Environment {
    let ts = google_cloud_auth::token::DefaultTokenSourceProvider::new(
        google_cloud_auth::project::Config::default()
            .with_audience(google_cloud_spanner::apiv1::conn_pool::AUDIENCE)
            .with_scopes(&google_cloud_spanner::apiv1::conn_pool::SCOPES),
    )
    .await
    .unwrap();
    GoogleCloud(Box::new(ts))
}

async fn query_change_record(
    tx: &mut ReadOnlyTransaction,
    now: OffsetDateTime,
    token: Option<String>,
) -> RowIterator<'_, impl Reader> {
    let query = format!(
        "
        SELECT ChangeRecord FROM READ_UserItemChangeStream (
          start_timestamp => @now,
          end_timestamp => NULL,
          partition_token => {},
          heartbeat_milliseconds => 10000
        )",
        match &token {
            Some(_) => "@token",
            None => "NULL",
        }
    );
    tracing::info!("query = {}", query);
    let mut stmt = Statement::new(query);
    stmt.add_param("now", &now);
    if let Some(token) = token {
        stmt.add_param("token", &token);
    }
    tx.query_with_option(
        stmt,
        QueryOptions {
            enable_resume: false,
            ..Default::default()
        },
    )
    .await
    .unwrap()
}

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_read_change_stream() {
    // Create Change Stream
    let cred = google_cloud_auth::credentials::CredentialsFile::new().await.unwrap();
    let project = cred.project_id.unwrap();
    let db = format!("projects/{}/instances/test-instance/databases/local-database", project);
    let admin_client = admin::client::Client::new(AdminClientConfig {
        environment: create_environment().await,
    })
    .await
    .unwrap();
    let _ = admin_client
        .database()
        .update_database_ddl(
            UpdateDatabaseDdlRequest {
                database: db.to_string(),
                statements: vec!["CREATE CHANGE STREAM UserItemChangeStream FOR UserItem".to_string()],
                operation_id: "".to_string(),
                proto_descriptors: vec![],
                throughput_mode: false,
            },
            None,
        )
        .await;

    sleep(Duration::from_secs(20)).await;

    let now = OffsetDateTime::now_utc();

    // Select Changed Data
    let config = ClientConfig {
        environment: create_environment().await,
        ..Default::default()
    };
    let client = Client::new(db.clone(), config).await.unwrap();
    let mut tx = client.single().await.unwrap();
    let mut row = query_change_record(&mut tx, now, None).await;
    let mut tasks = vec![];
    let mut index = 0;
    while let Some(row) = row.next().await.unwrap() {
        tasks.push(create_watcher(client.clone(), index, now, row).await);
        index += 1;
    }

    sleep(Duration::from_secs(30)).await;

    // Drop change stream
    tracing::info!("drop change stream");
    admin_client
        .database()
        .update_database_ddl(
            UpdateDatabaseDdlRequest {
                database: db.to_string(),
                statements: vec!["DROP CHANGE STREAM UserItemChangeStream".to_string()],
                operation_id: "".to_string(),
                proto_descriptors: vec![],
                throughput_mode: false,
            },
            None,
        )
        .await
        .unwrap();

    for task in tasks {
        let _ = task.await;
    }
}

async fn create_watcher(client: Client, i: usize, now: OffsetDateTime, row: Row) -> JoinHandle<()> {
    tokio::spawn(async move {
        let change_record: Vec<ChangeRecord> = row.column(0).unwrap();
        tracing::info!("change_{}={:?}", i, change_record);
        let mut tasks = vec![];
        for change in change_record {
            for child in change.child_partitions_record {
                for p in child.child_partitions {
                    let client = client.clone();
                    tasks.push(tokio::spawn(async move {
                        let mut tx = client.single().await.unwrap();
                        let mut rows = query_change_record(&mut tx, now, Some(p.token)).await;
                        let mut tick = tokio::time::interval(Duration::from_millis(100));
                        loop {
                            tokio::select! {
                                _ = tick.tick() => {
                                    tracing::info!("tick_{}", i);
                                    sleep(Duration::from_secs(10)).await;
                                },
                                row = rows.next() => {
                                    let row = match row {
                                        Ok(row) => match row {
                                            Some(row) => row,
                                            None => unreachable!("")
                                        },
                                        Err(e) => {
                                            // Detect Not Found error
                                            tracing::error!("expected error : {:?}", e);
                                            assert_eq!(e.code(), Code::NotFound);
                                            break;
                                        }
                                    };
                                    let change_record: Vec<ChangeRecord> = row.column(0).unwrap();
                                    tracing::info!("child_{i}={:?}", change_record);
                                }
                            }
                        }
                    }));
                }
            }
        }
        for task in tasks {
            let _ = task.await;
        }
    })
}
