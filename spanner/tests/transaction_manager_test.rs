use serial_test::serial;
use time::OffsetDateTime;

use common::*;
use google_cloud_spanner::key::Key;
use google_cloud_spanner::retry::TransactionRetry;
use google_cloud_spanner::row::Row;
use google_cloud_spanner::statement::Statement;

mod common;

#[ctor::ctor]
fn init() {
    let filter = tracing_subscriber::filter::EnvFilter::from_default_env()
        .add_directive("google_cloud_spanner=trace".parse().unwrap());
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
    std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
}

#[tokio::test]
#[serial]
async fn test_transaction_manager_basic() {
    // Set up test data
    let now = OffsetDateTime::now_utc();
    let data_client = create_data_client().await;
    let user_id = format!("user_tm_basic_{}", now.unix_timestamp());

    // Create initial user
    let cr = data_client
        .apply(vec![create_user_mutation(&user_id, &now)])
        .await
        .unwrap();

    // Test TransactionManager
    let mut tm = data_client.transaction_manager().await.unwrap();
    let retry = &mut TransactionRetry::new();

    let commit_timestamp = loop {
        let tx = tm.begin_read_write_transaction().await.unwrap();

        let result = async {
            // Add character and item
            let mut stmt1 = Statement::new(
                "INSERT INTO UserCharacter (UserId,CharacterId,Level,UpdatedAt) \
                 VALUES(@UserId,1,10,PENDING_COMMIT_TIMESTAMP())",
            );
            stmt1.add_param("UserId", &user_id);

            let mut stmt2 = Statement::new(
                "INSERT INTO UserItem (UserId,ItemId,Quantity,UpdatedAt) \
                 VALUES(@UserId,100,500,PENDING_COMMIT_TIMESTAMP())",
            );
            stmt2.add_param("UserId", &user_id);

            tx.update(stmt1).await?;
            tx.update(stmt2).await
        }
        .await;

        match tx.end(result, None).await {
            Ok((commit_result, _)) => {
                assert!(commit_result.timestamp.is_some());
                let ts = commit_result.timestamp.unwrap();
                break OffsetDateTime::from_unix_timestamp(ts.seconds)
                    .unwrap()
                    .replace_nanosecond(ts.nanos as u32)
                    .unwrap();
            }
            Err(err) => retry.next(err).await.unwrap(),
        }
    };

    // Verify the data was written
    let ts = cr.timestamp.unwrap();
    let user_commit_timestamp = OffsetDateTime::from_unix_timestamp(ts.seconds)
        .unwrap()
        .replace_nanosecond(ts.nanos as u32)
        .unwrap();

    verify_transaction_manager_data(&user_id, &now, &user_commit_timestamp, &commit_timestamp).await;
}

#[tokio::test]
#[serial]
async fn test_transaction_manager_rollback() {
    // Set up test data
    let now = OffsetDateTime::now_utc();
    let data_client = create_data_client().await;
    let user_id = format!("user_tm_rollback_{}", now.unix_timestamp());

    let cr = data_client
        .apply(vec![create_user_mutation(&user_id, &now)])
        .await
        .unwrap();

    // Test TransactionManager with rollback
    {
        let mut tm = data_client.transaction_manager().await.unwrap();
        let tx = tm.begin_read_write_transaction().await.unwrap();

        let result = async {
            // Try to update non-existent table (will cause rollback)
            let mut stmt = Statement::new("UPDATE User SET NullableString = 'test' WHERE UserId = @UserId");
            stmt.add_param("UserId", &user_id);
            tx.update(stmt).await?;

            // This should fail
            let stmt2 = Statement::new("UPDATE NonExistentTable SET Column = 'value'");
            tx.update(stmt2).await
        }
        .await;

        let _ = tx.end(result, None).await;
    }

    // Verify the data wasn't modified (rollback worked)
    let mut tx = data_client.read_only_transaction().await.unwrap();
    let reader = tx.read("User", &user_columns(), Key::new(&user_id)).await.unwrap();
    let row: Row = all_rows(reader).await.unwrap().pop().unwrap();

    let ts = cr.timestamp.unwrap();
    let ts = OffsetDateTime::from_unix_timestamp(ts.seconds)
        .unwrap()
        .replace_nanosecond(ts.nanos as u32)
        .unwrap();
    assert_user_row(&row, &user_id, &now, &ts);
}

#[tokio::test]
#[serial]
async fn test_transaction_manager_multiple_transactions() {
    // Set up test data
    let now = OffsetDateTime::now_utc();
    let data_client = create_data_client().await;
    let user_id = format!("user_tm_multi_{}", now.unix_timestamp());

    data_client
        .apply(vec![create_user_mutation(&user_id, &now)])
        .await
        .unwrap();

    // Test multiple transactions with the same TransactionManager
    let mut tm = data_client.transaction_manager().await.unwrap();

    // First transaction: add character
    {
        let tx = tm.begin_read_write_transaction().await.unwrap();

        let result = async {
            let mut stmt = Statement::new(
                "INSERT INTO UserCharacter (UserId,CharacterId,Level,UpdatedAt) \
                 VALUES(@UserId,1,5,PENDING_COMMIT_TIMESTAMP())",
            );
            stmt.add_param("UserId", &user_id);
            tx.update(stmt).await
        }
        .await;

        match tx.end(result, None).await {
            Ok(_) => (),
            Err(err) => panic!("First transaction failed: {:?}", err),
        }
    }

    // Second transaction: add item (reusing the same manager/session)
    {
        let tx = tm.begin_read_write_transaction().await.unwrap();

        let result = async {
            let mut stmt = Statement::new(
                "INSERT INTO UserItem (UserId,ItemId,Quantity,UpdatedAt) \
                 VALUES(@UserId,200,300,PENDING_COMMIT_TIMESTAMP())",
            );
            stmt.add_param("UserId", &user_id);
            tx.update(stmt).await
        }
        .await;

        match tx.end(result, None).await {
            Ok(_) => (),
            Err(err) => panic!("Second transaction failed: {:?}", err),
        }
    }

    // Verify both transactions were successful
    let mut tx = data_client.read_only_transaction().await.unwrap();
    let mut stmt = Statement::new(
        "SELECT *,
        ARRAY(SELECT AS STRUCT * FROM UserItem WHERE UserId = p.UserId) as UserItem,
        ARRAY(SELECT AS STRUCT * FROM UserCharacter WHERE UserId = p.UserId) as UserCharacter
        FROM User p WHERE UserId = @UserId",
    );
    stmt.add_param("UserId", &user_id);

    let reader = tx.query(stmt).await.unwrap();
    let rows: Vec<Row> = all_rows(reader).await.unwrap();

    assert_eq!(1, rows.len());
    let row = rows.first().unwrap();

    let user_items = row.column_by_name::<Vec<UserItem>>("UserItem").unwrap();
    assert_eq!(1, user_items.len());
    assert_eq!(user_items[0].item_id, 200);
    assert_eq!(user_items[0].quantity, 300);

    let user_characters = row.column_by_name::<Vec<UserCharacter>>("UserCharacter").unwrap();
    assert_eq!(1, user_characters.len());
    assert_eq!(user_characters[0].character_id, 1);
    assert_eq!(user_characters[0].level, 5);
}

async fn verify_transaction_manager_data(
    user_id: &str,
    now: &OffsetDateTime,
    user_commit_timestamp: &OffsetDateTime,
    commit_timestamp: &OffsetDateTime,
) {
    let data_client = create_data_client().await;
    let mut tx = data_client.read_only_transaction().await.unwrap();

    let mut stmt = Statement::new(
        "SELECT *,
        ARRAY(SELECT AS STRUCT * FROM UserItem WHERE UserId = p.UserId) as UserItem,
        ARRAY(SELECT AS STRUCT * FROM UserCharacter WHERE UserId = p.UserId) as UserCharacter
        FROM User p WHERE UserId = @UserId",
    );
    stmt.add_param("UserId", &user_id);

    let reader = tx.query(stmt).await.unwrap();
    let rows: Vec<Row> = all_rows(reader).await.unwrap();

    assert_eq!(1, rows.len());
    let row = rows.first().unwrap();
    assert_user_row(row, user_id, now, user_commit_timestamp);

    let mut user_items = row.column_by_name::<Vec<UserItem>>("UserItem").unwrap();
    let first_item = user_items.pop().unwrap();
    assert_eq!(first_item.user_id, *user_id);
    assert_eq!(first_item.item_id, 100);
    assert_eq!(first_item.quantity, 500);
    assert_eq!(
        OffsetDateTime::from(first_item.updated_at).to_string(),
        commit_timestamp.to_string()
    );
    assert!(user_items.is_empty());

    let mut user_characters = row.column_by_name::<Vec<UserCharacter>>("UserCharacter").unwrap();
    let first_character = user_characters.pop().unwrap();
    assert_eq!(first_character.user_id, *user_id);
    assert_eq!(first_character.character_id, 1);
    assert_eq!(first_character.level, 10);
    assert_eq!(
        OffsetDateTime::from(first_character.updated_at).to_string(),
        commit_timestamp.to_string()
    );
    assert!(user_characters.is_empty());
}

#[tokio::test]
#[serial]
async fn test_transaction_accessor() {
    // Set up test data
    let now = OffsetDateTime::now_utc();
    let data_client = create_data_client().await;
    let user_id = format!("user_tm_accessor_{}", now.unix_timestamp());

    data_client
        .apply(vec![create_user_mutation(&user_id, &now)])
        .await
        .unwrap();

    // Test transaction accessor method
    let mut tm = data_client.transaction_manager().await.unwrap();

    // Initially should return None
    assert!(tm.transaction().is_none());

    // Begin a transaction
    let _tx = tm.begin_read_write_transaction().await.unwrap();

    // Now should return Some
    assert!(tm.transaction().is_some());

    // Should be able to use the accessor to perform operations
    if let Some(tx) = tm.transaction() {
        let mut stmt = Statement::new(
            "INSERT INTO UserItem (UserId,ItemId,Quantity,UpdatedAt) \
             VALUES(@UserId,999,123,PENDING_COMMIT_TIMESTAMP())",
        );
        stmt.add_param("UserId", &user_id);
        tx.update(stmt).await.unwrap();

        // Commit via the accessor
        let result: Result<(), google_cloud_spanner::client::Error> = Ok(());
        match tx.end(result, None).await {
            Ok(_) => (),
            Err(err) => panic!("Commit failed: {:?}", err),
        }
    }

    // Verify the data was actually written
    let mut tx = data_client.read_only_transaction().await.unwrap();
    let mut stmt = Statement::new(
        "SELECT * FROM UserItem WHERE UserId = @UserId AND ItemId = 999",
    );
    stmt.add_param("UserId", &user_id);

    let reader = tx.query(stmt).await.unwrap();
    let rows: Vec<Row> = all_rows(reader).await.unwrap();

    assert_eq!(1, rows.len());
    let row = rows.first().unwrap();
    assert_eq!(row.column_by_name::<i64>("ItemId").unwrap(), 999);
    assert_eq!(row.column_by_name::<i64>("Quantity").unwrap(), 123);
}
