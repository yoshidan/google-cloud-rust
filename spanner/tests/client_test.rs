use anyhow::Context;
use google_cloud_spanner::client::{Client, RunInTxError};

use google_cloud_spanner::statement::Statement;

mod common;
use common::*;
use google_cloud_spanner::key::Key;

use google_cloud_gax::grpc::{Code, Status};
use google_cloud_spanner::retry::TransactionRetry;
use google_cloud_spanner::value::Timestamp;
use serial_test::serial;
use time::OffsetDateTime;

const DATABASE: &str = "projects/local-project/instances/test-instance/databases/local-database";

#[ctor::ctor]
fn init() {
    let _ = tracing_subscriber::fmt().try_init();
}

#[derive(thiserror::Error, Debug)]
pub enum DomainError {
    #[error("invalid")]
    UpdateInvalid(),
}

#[tokio::test]
#[serial]
async fn test_read_write_transaction() -> Result<(), anyhow::Error> {
    std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");

    // test data
    let now = OffsetDateTime::now_utc();
    let mut session = create_session().await;
    let user_id = format!("user_{}", now.unix_timestamp());
    replace_test_data(&mut session, vec![create_user_mutation(&user_id, &now)])
        .await
        .unwrap();

    let client = Client::new(DATABASE).await.context("error")?;
    let result: Result<(Option<Timestamp>, i64), RunInTxError> = client
        .read_write_transaction(
            |tx, _cancel| {
                let user_id= user_id.to_string();
                Box::pin(async move {
                    let ms = vec![create_user_mutation("user_client_1x", &now), create_user_mutation("user_client_2x", &now)];
                    tx.buffer_write(ms);
                    let mut stmt = Statement::new("Insert Into UserItem (UserId,ItemId,Quantity,UpdatedAt) VALUES(@UserId,1,1,PENDING_COMMIT_TIMESTAMP())");
                    stmt.add_param("UserId", &user_id);
                    let updated = tx.update(stmt).await?;
                    if updated == 0 {
                        Err(anyhow::Error::msg("error").into())
                    }else {
                        Ok(updated)
                    }
                })
            },
        )
        .await;
    let value = result.unwrap().0.unwrap();
    let ts = OffsetDateTime::from_unix_timestamp(value.seconds)
        .unwrap()
        .replace_nanosecond(value.nanos as u32)
        .unwrap();

    let mut ro = client.read_only_transaction().await?;
    let record = ro.read("User", &user_columns(), Key::new(&"user_client_1x")).await?;
    let row = all_rows(record).await.pop().unwrap();
    assert_user_row(&row, "user_client_1x", &now, &ts);

    let record = ro.read("User", &user_columns(), Key::new(&"user_client_2x")).await?;
    let row = all_rows(record).await.pop().unwrap();
    assert_user_row(&row, "user_client_2x", &now, &ts);

    let record = ro
        .read("UserItem", &["UpdatedAt"], Key::composite(&[&user_id, &1]))
        .await?;
    let row = all_rows(record).await.pop().unwrap();
    let cts = row.column_by_name::<OffsetDateTime>("UpdatedAt").unwrap();
    assert_eq!(cts.unix_timestamp(), ts.unix_timestamp());
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_apply() -> Result<(), anyhow::Error> {
    std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
    let users: Vec<String> = (0..2).map(|x| format!("user_client_{}", x)).collect();
    let client = Client::new(DATABASE).await.context("error")?;
    let now = OffsetDateTime::now_utc();
    let ms = users.iter().map(|id| create_user_mutation(id, &now)).collect();
    let value = client.apply(ms).await.unwrap().unwrap();
    let ts = OffsetDateTime::from_unix_timestamp(value.seconds)
        .unwrap()
        .replace_nanosecond(value.nanos as u32)
        .unwrap();

    let mut ro = client.read_only_transaction().await?;
    for x in users {
        let record = ro.read("User", &user_columns(), Key::new(&x)).await?;
        let row = all_rows(record).await.pop().unwrap();
        assert_user_row(&row, &x, &now, &ts);
    }
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_apply_at_least_once() -> Result<(), anyhow::Error> {
    std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
    let users: Vec<String> = (0..2).map(|x| format!("user_client_x_{}", x)).collect();
    let client = Client::new(DATABASE).await.context("error")?;
    let now = OffsetDateTime::now_utc();
    let ms = users.iter().map(|id| create_user_mutation(id, &now)).collect();
    let value = client.apply_at_least_once(ms).await.unwrap().unwrap();
    let ts = OffsetDateTime::from_unix_timestamp(value.seconds)
        .unwrap()
        .replace_nanosecond(value.nanos as u32)
        .unwrap();

    let mut ro = client.read_only_transaction().await?;
    for x in users {
        let record = ro.read("User", &user_columns(), Key::new(&x)).await?;
        let row = all_rows(record).await.pop().unwrap();
        assert_user_row(&row, &x, &now, &ts);
    }
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_partitioned_update() -> Result<(), anyhow::Error> {
    std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
    let now = OffsetDateTime::now_utc();
    let user_id = format!("user_{}", now.unix_timestamp());
    let mut session = create_session().await;
    replace_test_data(&mut session, vec![create_user_mutation(&user_id, &now)])
        .await
        .unwrap();
    let client = Client::new(DATABASE).await.context("error")?;
    let stmt = Statement::new("UPDATE User SET NullableString = 'aaa' WHERE NullableString IS NOT NULL");
    client.partitioned_update(stmt).await.unwrap();

    let mut single = client.single().await.unwrap();
    let rows = single
        .read("User", &["NullableString"], Key::new(&user_id))
        .await
        .unwrap();
    let row = all_rows(rows).await.pop().unwrap();
    let value = row.column_by_name::<String>("NullableString").unwrap();
    assert_eq!(value, "aaa");
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_batch_read_only_transaction() -> Result<(), anyhow::Error> {
    std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
    let now = OffsetDateTime::now_utc();

    let mut session = create_session().await;
    let many = (0..20000)
        .map(|x| create_user_mutation(&format!("user_partition_{}_{}", now.unix_timestamp(), x), &now))
        .collect();
    replace_test_data(&mut session, many).await.unwrap();

    let client = Client::new(DATABASE).await.context("error")?;
    let mut tx = client.batch_read_only_transaction().await.unwrap();

    let stmt = Statement::new(format!(
        "SELECT * FROM User p WHERE p.UserId LIKE 'user_partition_{}_%' ",
        now.unix_timestamp()
    ));
    let rows = execute_partitioned_query(&mut tx, stmt).await;
    assert_eq!(20000, rows.len());
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_begin_read_write_transaction_retry() -> Result<(), anyhow::Error> {
    std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
    let client = Client::new(DATABASE).await.context("error")?;

    let tx = &mut client.begin_read_write_transaction().await?;
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
    Ok(())
}
