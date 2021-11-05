use anyhow::Context;
use google_cloud_spanner::client::{Client, TxError};
use google_cloud_spanner::mutation::insert;
use google_cloud_spanner::statement::{Statement, ToKind};

mod common;
use chrono::{NaiveDateTime, Utc};
use common::*;
use google_cloud_spanner::key::{Key, KeySet};
use google_cloud_spanner::value::CommitTimestamp;
use serial_test::serial;

const DATABASE: &str = "projects/local-project/instances/test-instance/databases/local-database";

#[tokio::test]
#[serial]
async fn test_read_write_transaction() -> Result<(), anyhow::Error> {
    std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");

    // test data
    let now = Utc::now().naive_utc();
    let mut session = create_session().await;
    let user_id = format!("user_{}", now.timestamp());
    replace_test_data(&mut session, vec![create_user_mutation(&user_id, &now)])
        .await
        .unwrap();

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
    let record = ro
        .read(
            "User",
            user_columns(),
            KeySet::from(Key::one("user_client_1x")),
            None,
        )
        .await?;
    let row = all_rows(record).await.pop().unwrap();
    assert_user_row(&row, "user_client_1x", &now, &ts);

    let record = ro
        .read(
            "User",
            user_columns(),
            KeySet::from(Key::one("user_client_2x")),
            None,
        )
        .await?;
    let row = all_rows(record).await.pop().unwrap();
    assert_user_row(&row, "user_client_2x", &now, &ts);

    let record = ro
        .read(
            "UserItem",
            vec!["UpdatedAt"],
            KeySet::from(Key::new(vec![user_id.to_kind(), 1.to_kind()])),
            None,
        )
        .await?;
    let row = all_rows(record).await.pop().unwrap();
    let cts = row.column_by_name::<NaiveDateTime>("UpdatedAt").unwrap();
    assert_eq!(cts.timestamp(), ts.timestamp());
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_apply() -> Result<(), anyhow::Error> {
    std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
    let users: Vec<String> = (0..2).map(|x| format!("user_client_{}", x)).collect();
    let client = Client::new(DATABASE, None).await.context("error")?;
    let now = Utc::now().naive_utc();
    let ms = users
        .iter()
        .map(|id| create_user_mutation(id, &now))
        .collect();
    let value = client.apply(ms, None).await.unwrap().unwrap();
    let ts = NaiveDateTime::from_timestamp(value.seconds, value.nanos as u32);

    let mut ro = client.read_only_transaction(None).await?;
    for x in users {
        let record = ro
            .read(
                "User",
                user_columns(),
                KeySet::from(Key::one(x.clone())),
                None,
            )
            .await?;
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
    let client = Client::new(DATABASE, None).await.context("error")?;
    let now = Utc::now().naive_utc();
    let ms = users
        .iter()
        .map(|id| create_user_mutation(id, &now))
        .collect();
    let value = client.apply_at_least_once(ms, None).await.unwrap().unwrap();
    let ts = NaiveDateTime::from_timestamp(value.seconds, value.nanos as u32);

    let mut ro = client.read_only_transaction(None).await?;
    for x in users {
        let record = ro
            .read(
                "User",
                user_columns(),
                KeySet::from(Key::one(x.clone())),
                None,
            )
            .await?;
        let row = all_rows(record).await.pop().unwrap();
        assert_user_row(&row, &x, &now, &ts);
    }
    Ok(())
}

#[tokio::test]
#[serial]
async fn test_partitioned_update() -> Result<(), anyhow::Error> {
    std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
    let now = Utc::now().naive_utc();
    let user_id = format!("user_{}", now.timestamp());
    let mut session = create_session().await;
    replace_test_data(&mut session, vec![create_user_mutation(&user_id, &now)])
        .await
        .unwrap();
    let client = Client::new(DATABASE, None).await.context("error")?;
    let stmt =
        Statement::new("UPDATE User SET NullableString = 'aaa' WHERE NullableString IS NOT NULL");
    client.partitioned_update(stmt, None).await.unwrap();

    let mut single = client.single(None).await.unwrap();
    let rows = single
        .read(
            "User",
            vec!["NullableString"],
            KeySet::from(Key::one(user_id.clone())),
            None,
        )
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
    let now = Utc::now().naive_utc();

    let mut session = create_session().await;
    let many = (0..20000)
        .map(|x| create_user_mutation(&format!("user_partition2_{}_{}", x,now.timestamp()), &now))
        .collect();
    let cr = replace_test_data(&mut session, many).await.unwrap();

    let client = Client::new(DATABASE, None).await.context("error")?;
    let mut tx = client.batch_read_only_transaction(None).await.unwrap();

    let ts = cr.commit_timestamp.as_ref().unwrap();
    let ts = NaiveDateTime::from_timestamp(ts.seconds, ts.nanos as u32);
    let stmt = Statement::new("SELECT * FROM User p WHERE p.UserId LIKE 'user_partition2_%' ");
    let rows = execute_partitioned_query(&mut tx, stmt).await;
    assert_eq!(20000, rows.len());
    Ok(())
}
