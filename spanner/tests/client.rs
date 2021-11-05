use anyhow::Context;
use google_cloud_spanner::client::{Client, TxError};
use google_cloud_spanner::mutation::insert;
use google_cloud_spanner::statement::{Statement, ToKind};

mod common;
use common::*;
use chrono::{Utc, NaiveDateTime};
use google_cloud_spanner::key::{KeySet, Key};
use google_cloud_spanner::value::CommitTimestamp;

const DATABASE: &str = "projects/local-project/instances/test-instance/databases/local-database";

#[tokio::test]
async fn test_read_write_transaction() -> Result<(), anyhow::Error> {
    std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");

    // test data
    let now = Utc::now().naive_utc();
    let mut session = create_session().await;
    let user_id = format!("user_{}", now.timestamp());
    replace_test_data(&mut session, vec![create_user_mutation(&user_id, &now)])
        .await
        .unwrap();

    //TODO check copy
    let user_id_ref = &user_id;
    let client = Client::new(DATABASE, None).await.context("error")?;
    let value = client
        .read_write_transaction(
            |mut tx| async move {
                let result = async {
                    let tx2 = &mut tx;
                    let ms = vec![create_user_mutation("user_client_1x", &now), create_user_mutation("user_client_2x", &now)];
                    tx2.buffer_write(ms);
                    let mut stmt = Statement::new("Insert Into UserItem (UserId,ItemId,Quantity,UpdatedAt) VALUES(@UserId,1,1,PENDING_COMMIT_TIMESTAMP())");
                    stmt.add_param("UserId",(*user_id_ref).clone());
                    tx2.update(stmt, None).await.map_err(TxError::TonicStatus)
                }
                .await;
                return (tx, result);
            },
            None,
        )
        .await.unwrap().0.unwrap();
    let ts = NaiveDateTime::from_timestamp(value.seconds, value.nanos as u32);

    let mut ro = client.read_only_transaction(None).await?;
    let record= ro.read("User", user_columns(), KeySet::from(Key::one("user_client_1x")), None).await?;
    let row = all_rows(record).await.pop().unwrap();
    assert_user_row(&row, "user_client_1x", &now, &ts);

    let record = ro.read("User", user_columns(), KeySet::from(Key::one("user_client_2x")), None).await?;
    let row = all_rows(record).await.pop().unwrap();
    assert_user_row(&row, "user_client_2x", &now, &ts);

    let record= ro.read("UserItem", vec!["UpdatedAt"], KeySet::from(Key::new(vec![user_id.to_kind(), 1.to_kind()])), None).await?;
    let row = all_rows(record).await.pop().unwrap();
    let cts = row.column_by_name::<NaiveDateTime>("UpdatedAt").unwrap();
    assert_eq!(cts.timestamp(), ts.timestamp());
    Ok(())
}

#[tokio::test]
async fn test_apply() -> Result<(), anyhow::Error> {
    std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
    let client = Client::new(DATABASE, None).await.context("error")?;
    let now = Utc::now().naive_utc();
    let ms = vec![create_user_mutation("user_client_1", &now), create_user_mutation("user_client_2", &now)];
    let value = client.apply(ms, None).await.unwrap().unwrap();
    let ts = NaiveDateTime::from_timestamp(value.seconds, value.nanos as u32);

    let mut ro = client.read_only_transaction(None).await?;
    let record= ro.read("User", user_columns(), KeySet::from(Key::one("user_client_1")), None).await?;
    let row = all_rows(record).await.pop().unwrap();
    assert_user_row(&row, "user_client_1", &now, &ts);

    let record = ro.read("User", user_columns(), KeySet::from(Key::one("user_client_2")), None).await?;
    let row = all_rows(record).await.pop().unwrap();
    assert_user_row(&row, "user_client_2", &now, &ts);
    Ok(())
}