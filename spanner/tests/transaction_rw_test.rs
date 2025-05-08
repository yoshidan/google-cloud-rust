use serial_test::serial;
use time::OffsetDateTime;

use common::*;
use google_cloud_spanner::key::Key;
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
async fn test_mutation_and_statement() {
    //set up test data
    let now = OffsetDateTime::now_utc();
    let data_client = create_data_client().await;
    let past_user = format!("user_{}", now.unix_timestamp());
    let cr = data_client
        .apply(vec![create_user_mutation(&past_user, &now)])
        .await
        .unwrap();

    //test
    let mut tx = data_client.begin_read_write_transaction().await.unwrap();
    let result = async {
        let user_id_1 = "user_rw_1";
        let user_id_2 = "user_rw_2";
        let user_id_3 = "user_rw_3";
        tx.buffer_write(vec![create_user_mutation(user_id_1, &now)]);
        tx.buffer_write(vec![create_user_mutation(user_id_2, &now)]);
        tx.buffer_write(vec![create_user_mutation(user_id_3, &now)]);

        let mut stmt1 = Statement::new("INSERT INTO UserCharacter (UserId,CharacterId,Level,UpdatedAt) VALUES(@UserId,1,1,PENDING_COMMIT_TIMESTAMP())");
        stmt1.add_param("UserId", &past_user);
        let mut stmt2 = Statement::new("INSERT INTO UserItem (UserId,ItemId,Quantity,UpdatedAt) VALUES(@UserId,10,1000,PENDING_COMMIT_TIMESTAMP())");
        stmt2.add_param("UserId", &past_user);
        tx.update( stmt1).await?;
        tx.update( stmt2).await
    }.await;

    let result = tx.end(result, None).await;
    let commit_timestamp = match result {
        Ok(s) => {
            assert!(s.0.timestamp.is_some());
            let ts = s.0.timestamp.unwrap();
            let dt = OffsetDateTime::from_unix_timestamp(ts.seconds)
                .unwrap()
                .replace_nanosecond(ts.nanos as u32)
                .unwrap();
            println!("commit time stamp is {dt}");
            dt
        }
        Err(e) => panic!("error {e:?}"),
    };

    let ts = cr.timestamp.unwrap();
    let ts = OffsetDateTime::from_unix_timestamp(ts.seconds)
        .unwrap()
        .replace_nanosecond(ts.nanos as u32)
        .unwrap();
    assert_data(&past_user, &now, &ts, &commit_timestamp).await;
}

#[tokio::test]
#[serial]
async fn test_rollback() {
    //set up test data
    let now = OffsetDateTime::now_utc();
    let data_client = create_data_client().await;
    let past_user = format!("user_{}", now.unix_timestamp());
    let cr = data_client
        .apply(vec![create_user_mutation(&past_user, &now)])
        .await
        .unwrap();

    //test
    {
        let mut tx = data_client.begin_read_write_transaction().await.unwrap();
        let result = async {
            let mut stmt1 = Statement::new("UPDATE User SET NullableString = 'aaaaaaa' WHERE UserId = @UserId");
            stmt1.add_param("UserId", &past_user);
            tx.update(stmt1).await?;

            let stmt2 = Statement::new("UPDATE UserNoteFound SET Quantity = 10000");
            tx.update(stmt2).await
        }
        .await;
        let _ = tx.end(result, None).await;
    }
    let mut tx = data_client.read_only_transaction().await.unwrap();
    let reader = tx.read("User", &user_columns(), Key::new(&past_user)).await.unwrap();
    let row: Row = all_rows(reader).await.unwrap().pop().unwrap();
    let ts = cr.timestamp.unwrap();
    let ts = OffsetDateTime::from_unix_timestamp(ts.seconds)
        .unwrap()
        .replace_nanosecond(ts.nanos as u32)
        .unwrap();
    assert_user_row(&row, &past_user, &now, &ts);
}

async fn assert_data(
    user_id: &str,
    now: &OffsetDateTime,
    user_commit_timestamp: &OffsetDateTime,
    commit_timestamp: &OffsetDateTime,
) {
    // get by another transaction
    let data_client = create_data_client().await;
    let mut tx = data_client.begin_read_write_transaction().await.unwrap();
    let result = async {
        let mut stmt = Statement::new(
            "SELECT *,
        ARRAY(SELECT AS STRUCT * FROM UserItem WHERE UserId = p.UserId) as UserItem,
        ARRAY(SELECT AS STRUCT * FROM UserCharacter WHERE UserId = p.UserId) as UserCharacter,
        FROM User p WHERE UserId = @UserId;
    ",
        );
        stmt.add_param("UserId", &user_id);
        let result = tx.query(stmt).await?;
        all_rows(result).await
    }
    .await;

    // commit or rollback is required for rw transaction
    let rows: Vec<Row> = match tx.end(result, None).await {
        Ok(s) => s.1,
        Err(e) => panic!("tx error {e:?}"),
    };

    assert_eq!(1, rows.len());
    let row = rows.first().unwrap();
    assert_user_row(row, user_id, now, user_commit_timestamp);

    let mut user_items = row.column_by_name::<Vec<UserItem>>("UserItem").unwrap();
    let first_item = user_items.pop().unwrap();
    assert_eq!(first_item.user_id, *user_id);
    assert_eq!(first_item.item_id, 10);
    assert_eq!(first_item.quantity, 1000);
    assert_eq!(
        OffsetDateTime::from(first_item.updated_at).to_string(),
        commit_timestamp.to_string()
    );
    assert!(user_items.is_empty());

    let mut user_characters = row.column_by_name::<Vec<UserCharacter>>("UserCharacter").unwrap();
    let first_character = user_characters.pop().unwrap();
    assert_eq!(first_character.user_id, *user_id);
    assert_eq!(first_character.character_id, 1);
    assert_eq!(first_character.level, 1);
    assert_eq!(
        OffsetDateTime::from(first_character.updated_at).to_string(),
        commit_timestamp.to_string()
    );
    assert!(user_characters.is_empty());
}
