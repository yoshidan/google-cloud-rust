use serial_test::serial;
use time::OffsetDateTime;

use common::*;
use gcloud_spanner::transaction_rw::CommitResult;
use google_cloud_gax::conn::Environment;
use google_cloud_gax::grpc::{Code, Status};
use google_cloud_gax::retry::TryAs;
use google_cloud_spanner::client::{Client, ClientConfig, Error};
use google_cloud_spanner::key::Key;
use google_cloud_spanner::retry::TransactionRetry;
use google_cloud_spanner::row::Row;
use google_cloud_spanner::session::SessionError;
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

#[derive(thiserror::Error, Debug)]
pub enum DomainError {
    #[error("invalid")]
    UpdateInvalid,
    #[error(transparent)]
    Tx(#[from] Error),
}

impl TryAs<Status> for DomainError {
    fn try_as(&self) -> Option<&Status> {
        match self {
            DomainError::Tx(Error::GRPC(status)) => Some(status),
            _ => None,
        }
    }
}

impl From<Status> for DomainError {
    fn from(status: Status) -> Self {
        Self::Tx(Error::GRPC(status))
    }
}

impl From<SessionError> for DomainError {
    fn from(se: SessionError) -> Self {
        Self::Tx(Error::InvalidSession(se))
    }
}

#[tokio::test]
#[serial]
async fn test_read_write_transaction() {
    // set up data
    let now = OffsetDateTime::now_utc();
    let user_id = format!("user_{}", now.unix_timestamp());
    let data_client = create_data_client().await;
    data_client
        .apply(vec![create_user_mutation(&user_id, &now)])
        .await
        .unwrap();

    // test
    let client = Client::new(DATABASE, ClientConfig::default()).await.unwrap();
    let result: Result<(CommitResult, i64), DomainError> = client
        .read_write_transaction(
            |tx| {
                let user_id= user_id.to_string();
                Box::pin(async move {
                    let ms = vec![create_user_mutation("user_client_1x", &now), create_user_mutation("user_client_2x", &now)];
                    tx.buffer_write(ms);
                    let mut stmt = Statement::new("Insert Into UserItem (UserId,ItemId,Quantity,UpdatedAt) VALUES(@UserId,1,1,PENDING_COMMIT_TIMESTAMP())");
                    stmt.add_param("UserId", &user_id);
                    let updated = tx.update(stmt).await?;
                    if updated == 0 {
                        Err(DomainError::UpdateInvalid)
                    }else {
                        Ok(updated)
                    }
                })
            },
        )
        .await;
    let value = result.unwrap().0;
    let ts = OffsetDateTime::from_unix_timestamp(value.timestamp.as_ref().unwrap().seconds)
        .unwrap()
        .replace_nanosecond(value.timestamp.unwrap().nanos as u32)
        .unwrap();

    let mut ro = client.read_only_transaction().await.unwrap();
    let record = ro
        .read("User", &user_columns(), Key::new(&"user_client_1x"))
        .await
        .unwrap();
    let row = all_rows(record).await.unwrap().pop().unwrap();
    assert_user_row(&row, "user_client_1x", &now, &ts);

    let record = ro
        .read("User", &user_columns(), Key::new(&"user_client_2x"))
        .await
        .unwrap();
    let row = all_rows(record).await.unwrap().pop().unwrap();
    assert_user_row(&row, "user_client_2x", &now, &ts);

    let record = ro
        .read("UserItem", &["UpdatedAt"], Key::composite(&[&user_id, &1]))
        .await
        .unwrap();
    let row = all_rows(record).await.unwrap().pop().unwrap();
    let cts = row.column_by_name::<OffsetDateTime>("UpdatedAt").unwrap();
    assert_eq!(cts.unix_timestamp(), ts.unix_timestamp());
}

#[tokio::test]
#[serial]
async fn test_apply() {
    let users: Vec<String> = (0..2).map(|x| format!("user_client_{x}")).collect();
    let client = Client::new(DATABASE, ClientConfig::default()).await.unwrap();
    let now = OffsetDateTime::now_utc();
    let ms = users.iter().map(|id| create_user_mutation(id, &now)).collect();
    let value = client.apply(ms).await.unwrap();
    let ts = OffsetDateTime::from_unix_timestamp(value.timestamp.as_ref().unwrap().seconds)
        .unwrap()
        .replace_nanosecond(value.timestamp.unwrap().nanos as u32)
        .unwrap();

    let mut ro = client.read_only_transaction().await.unwrap();
    for x in users {
        let record = ro.read("User", &user_columns(), Key::new(&x)).await.unwrap();
        let row: Row = all_rows(record).await.unwrap().pop().unwrap();
        assert_user_row(&row, &x, &now, &ts);
    }
}

#[tokio::test]
#[serial]
async fn test_apply_at_least_once() {
    let users: Vec<String> = (0..2).map(|x| format!("user_client_x_{x}")).collect();
    let client = Client::new(DATABASE, ClientConfig::default()).await.unwrap();
    let now = OffsetDateTime::now_utc();
    let ms = users.iter().map(|id| create_user_mutation(id, &now)).collect();
    let value = client.apply_at_least_once(ms).await.unwrap().unwrap();
    let ts = OffsetDateTime::from_unix_timestamp(value.timestamp.as_ref().unwrap().seconds)
        .unwrap()
        .replace_nanosecond(value.timestamp.unwrap().nanos as u32)
        .unwrap();

    let mut ro = client.read_only_transaction().await.unwrap();
    for x in users {
        let record = ro.read("User", &user_columns(), Key::new(&x)).await.unwrap();
        let row = all_rows(record).await.unwrap().pop().unwrap();
        assert_user_row(&row, &x, &now, &ts);
    }
}

#[tokio::test]
#[serial]
async fn test_partitioned_update() {
    //set up test data
    let now = OffsetDateTime::now_utc();
    let user_id = format!("user_{}", now.unix_timestamp());
    let data_client = create_data_client().await;
    data_client
        .apply(vec![create_user_mutation(&user_id, &now)])
        .await
        .unwrap();

    // test
    let client = Client::new(DATABASE, ClientConfig::default()).await.unwrap();
    let stmt = Statement::new("UPDATE User SET NullableString = 'aaa' WHERE NullableString IS NOT NULL");
    client.partitioned_update(stmt).await.unwrap();

    let mut single = client.single().await.unwrap();
    let rows = single
        .read("User", &["NullableString"], Key::new(&user_id))
        .await
        .unwrap();
    let row = all_rows(rows).await.unwrap().pop().unwrap();
    let value = row.column_by_name::<String>("NullableString").unwrap();
    assert_eq!(value, "aaa");
}

#[tokio::test]
#[serial]
async fn test_batch_read_only_transaction() {
    //set up test data
    let now = OffsetDateTime::now_utc();
    let many = (0..20000)
        .map(|x| create_user_mutation(&format!("user_partition_{}_{}", now.unix_timestamp(), x), &now))
        .collect();
    let data_client = create_data_client().await;
    data_client.apply(many).await.unwrap();

    // test
    let client = Client::new(DATABASE, ClientConfig::default()).await.unwrap();
    let mut tx = client.batch_read_only_transaction().await.unwrap();

    let stmt = Statement::new(format!(
        "SELECT * FROM User p WHERE p.UserId LIKE 'user_partition_{}_%' ",
        now.unix_timestamp()
    ));
    let rows = execute_partitioned_query(&mut tx, stmt).await;
    assert_eq!(20000, rows.len());
}

#[tokio::test]
#[serial]
async fn test_begin_read_write_transaction_retry() {
    let client = Client::new(DATABASE, ClientConfig::default()).await.unwrap();
    let tx = &mut client.begin_read_write_transaction().await.unwrap();
    let retry = &mut TransactionRetry::new();
    let mut retry_count = 0;
    loop {
        let result: Result<(), Status> = Err(Status::new(Code::Aborted, "test"));
        match tx.end(result, None).await {
            Ok(_) => {
                unreachable!("must never success");
            }
            Err(err) => {
                if retry.next(err).await.is_err() {
                    break;
                } else {
                    retry_count += 1;
                }
            }
        }
    }
    assert_eq!(retry_count, 5);
}

#[tokio::test]
async fn test_with_auth() {
    let config = ClientConfig::default().with_auth().await.unwrap();
    if let Environment::GoogleCloud(_) = config.environment {
        unreachable!()
    }
}
