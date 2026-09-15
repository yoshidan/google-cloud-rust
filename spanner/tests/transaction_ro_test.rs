use bigdecimal::BigDecimal;
use std::collections::HashMap;
use std::ops::Add;

use serial_test::serial;
use time::{Duration, OffsetDateTime};

use common::*;
use google_cloud_googleapis::spanner::v1::execute_sql_request::QueryMode;
use google_cloud_spanner::key::Key;
use google_cloud_spanner::row::Row;
use google_cloud_spanner::statement::Statement;
use google_cloud_spanner::transaction::QueryOptions;
use google_cloud_spanner::transaction_ro::ReadOnlyTransaction;

mod common;

#[ctor::ctor]
fn init() {
    let filter = tracing_subscriber::filter::EnvFilter::from_default_env()
        .add_directive("google_cloud_spanner=trace".parse().unwrap());
    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
    // Leave an explicitly configured host alone so the suite can be pointed at Spanner Omni.
    if std::env::var("SPANNER_EMULATOR_HOST").is_err() {
        std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
    }
}

async fn assert_read(tx: &mut ReadOnlyTransaction, user_id: &str, now: &OffsetDateTime, cts: &OffsetDateTime) {
    let reader = match tx.read("User", &user_columns(), Key::new(&user_id)).await {
        Ok(tx) => tx,
        Err(status) => panic!("read error {status:?}"),
    };
    let mut rows = all_rows(reader).await.unwrap();
    assert_eq!(1, rows.len(), "row must exists");
    let row = rows.pop().unwrap();
    assert_user_row(&row, user_id, now, cts);
}

async fn assert_query(tx: &mut ReadOnlyTransaction, user_id: &str, now: &OffsetDateTime, cts: &OffsetDateTime) {
    let mut stmt = Statement::new("SELECT * FROM User WHERE UserId = @UserID");
    stmt.add_param("UserId", &user_id);
    let mut rows = execute_query(tx, stmt).await;
    assert_eq!(1, rows.len(), "row must exists");
    let row = rows.pop().unwrap();
    assert_user_row(&row, user_id, now, cts);
}

async fn execute_query(tx: &mut ReadOnlyTransaction, stmt: Statement) -> Vec<Row> {
    let reader = match tx.query(stmt).await {
        Ok(tx) => tx,
        Err(status) => panic!("query error {status:?}"),
    };
    all_rows(reader).await.unwrap()
}

#[tokio::test]
#[serial]
async fn test_query_and_read() {
    //set up test data
    let now = OffsetDateTime::now_utc();
    let data_client = create_data_client().await;
    let user_id_1 = "user_1";
    let user_id_2 = "user_2";
    let user_id_3 = "user_3";
    let cr = data_client
        .apply(vec![
            create_user_mutation(user_id_1, &now),
            create_user_mutation(user_id_2, &now),
            create_user_mutation(user_id_3, &now),
        ])
        .await
        .unwrap();

    //test
    let mut tx = data_client.read_only_transaction().await.unwrap();
    let ts = cr.timestamp.unwrap();
    let ts = OffsetDateTime::from_unix_timestamp(ts.seconds)
        .unwrap()
        .replace_nanosecond(ts.nanos as u32)
        .unwrap();
    assert_query(&mut tx, user_id_1, &now, &ts).await;
    assert_query(&mut tx, user_id_2, &now, &ts).await;
    assert_query(&mut tx, user_id_3, &now, &ts).await;
    assert_read(&mut tx, user_id_1, &now, &ts).await;
    assert_read(&mut tx, user_id_2, &now, &ts).await;
    assert_read(&mut tx, user_id_3, &now, &ts).await;
}

#[tokio::test]
#[serial]
async fn test_single_use_read_timestamp() {
    use google_cloud_spanner::reader::{Reader, RowIterator};
    use google_cloud_spanner::value::TimestampBound;

    async fn check(mut rows: RowIterator<'_, impl Reader>, timestamp: prost_types::Timestamp, has_row: bool) {
        assert!(rows.read_timestamp().is_none());
        assert_eq!(rows.next().await.unwrap().is_some(), has_row);
        assert!(rows.next().await.unwrap().is_none());
        assert_eq!(rows.read_timestamp(), Some(&timestamp));
    }

    let client = create_data_client().await;
    let user_id = "single_use_read_timestamp";
    let timestamp = client
        .apply(vec![create_user_mutation(user_id, &OffsetDateTime::now_utc())])
        .await
        .unwrap()
        .timestamp
        .unwrap();
    for (id, has_row) in [(user_id, true), ("missing_read_timestamp_user", false)] {
        let mut tx = client
            .single_with_timestamp_bound(TimestampBound::read_timestamp(timestamp.clone()))
            .await
            .unwrap();
        let mut statement = Statement::new("SELECT UserId FROM User WHERE UserId = @id");
        statement.add_param("id", &id);
        check(tx.query(statement).await.unwrap(), timestamp.clone().into(), has_row).await;
        drop(tx);

        let mut tx = client
            .single_with_timestamp_bound(TimestampBound::read_timestamp(timestamp.clone()))
            .await
            .unwrap();
        check(
            tx.read("User", &["UserId"], Key::new(&id)).await.unwrap(),
            timestamp.clone().into(),
            has_row,
        )
        .await;
    }
}

#[tokio::test]
#[serial]
async fn test_complex_query() {
    let now = OffsetDateTime::now_utc();
    let data_client = create_data_client().await;
    let user_id_1 = "user_10";
    let cr = data_client
        .apply(vec![
            create_user_mutation(user_id_1, &now),
            create_user_item_mutation(user_id_1, 1),
            create_user_item_history_mutation(user_id_1, 1, &now),
            create_user_item_mutation(user_id_1, 2),
            create_user_item_history_mutation(user_id_1, 2, &now.add(Duration::seconds(-1))),
            create_user_item_history_mutation(user_id_1, 2, &now),
            create_user_character_mutation(user_id_1, 10),
            create_user_character_mutation(user_id_1, 20),
        ])
        .await
        .unwrap();

    let mut tx = data_client.read_only_transaction().await.unwrap();
    let mut stmt = Statement::new(
        "SELECT *,
        ARRAY(
            SELECT AS STRUCT
                *,
                ARRAY(SELECT AS STRUCT * FROM UserItemHistory uih WHERE ui.UserId = uih.UserID AND ui.ItemId = uih.ItemId ORDER BY uih.UsedAt) as UserItemHistory
            FROM UserItem ui WHERE ui.UserId = p.UserId
            ORDER BY ui.ItemID
        ) as UserItem,
        ARRAY(SELECT AS STRUCT * FROM UserCharacter WHERE UserId = p.UserId ORDER BY CharacterID) as UserCharacter,
        FROM User p WHERE UserId = @UserId;
    ",
    );
    stmt.add_param("UserId", &user_id_1);
    let mut rows = execute_query(&mut tx, stmt).await;
    assert_eq!(1, rows.len());
    let row = rows.pop().unwrap();

    // check UserTable
    let ts = cr.timestamp.unwrap();
    let ts = OffsetDateTime::from_unix_timestamp(ts.seconds)
        .unwrap()
        .replace_nanosecond(ts.nanos as u32)
        .unwrap();
    assert_user_row(&row, user_id_1, &now, &ts);

    let mut user_items = row.column_by_name::<Vec<UserItemWithHistory>>("UserItem").unwrap();
    let first_item = user_items.pop().unwrap();
    assert_eq!(first_item.user_id, user_id_1);
    assert_eq!(first_item.item_id, 2);
    assert_eq!(first_item.quantity, 100);
    assert_eq!(first_item.user_item_history.len(), 2);
    assert_ne!(OffsetDateTime::from(first_item.updated_at).to_string(), now.to_string());
    assert_eq!(first_item.user_item_history[0].user_id, user_id_1);
    assert_eq!(first_item.user_item_history[0].item_id, first_item.item_id);
    assert_ne!(
        &(first_item.user_item_history[0].used_at).to_string(),
        &(first_item.user_item_history[1].used_at).to_string()
    );
    assert_eq!(first_item.user_item_history[1].user_id, user_id_1);
    assert_eq!(first_item.user_item_history[1].item_id, first_item.item_id);
    assert_eq!(&(first_item.user_item_history[1].used_at).to_string(), &now.to_string());
    let second_item = user_items.pop().unwrap();
    assert_eq!(second_item.user_id, user_id_1);
    assert_eq!(second_item.item_id, 1);
    assert_eq!(second_item.quantity, 100);
    assert_eq!(second_item.user_item_history.len(), 1);
    assert_eq!(second_item.user_item_history[0].user_id, user_id_1);
    assert_eq!(second_item.user_item_history[0].item_id, second_item.item_id);
    assert_eq!(&(second_item.user_item_history[0].used_at).to_string(), &now.to_string());
    assert_ne!(OffsetDateTime::from(second_item.updated_at).to_string(), now.to_string());
    assert!(user_items.is_empty());

    let mut user_characters = row.column_by_name::<Vec<UserCharacter>>("UserCharacter").unwrap();
    let first_character = user_characters.pop().unwrap();
    assert_eq!(first_character.user_id, user_id_1);
    assert_eq!(first_character.character_id, 20);
    assert_eq!(first_character.level, 1);
    assert_ne!(OffsetDateTime::from(first_character.updated_at).to_string(), now.to_string());
    let second_character = user_characters.pop().unwrap();
    assert_eq!(second_character.user_id, user_id_1);
    assert_eq!(second_character.character_id, 10);
    assert_eq!(second_character.level, 1);
    assert_ne!(OffsetDateTime::from(second_character.updated_at).to_string(), now.to_string());
    assert!(user_characters.is_empty());
}

#[tokio::test]
#[serial]
async fn test_batch_partition_query_and_read() {
    // set up test data
    let now = OffsetDateTime::now_utc();
    let data_client = create_data_client().await;
    let user_id_1 = "user_1";
    let user_id_2 = "user_2";
    let user_id_3 = "user_3";
    let cr = data_client
        .apply(vec![
            create_user_mutation(user_id_1, &now),
            create_user_mutation(user_id_2, &now),
            create_user_mutation(user_id_3, &now),
        ])
        .await
        .unwrap();

    let many = (0..20000)
        .map(|x| create_user_mutation(&format!("user_partitionx_{x}"), &now))
        .collect();
    // Too many mutations for a single transaction, so each chunk lands at its own commit
    // timestamp and rows have to be checked against the chunk that wrote them.
    let cr2 = apply_in_chunks(&data_client, many, USER_ROWS_PER_COMMIT).await;

    // test
    let mut tx = data_client.batch_read_only_transaction().await.unwrap();
    let ts = cr.timestamp.unwrap();
    let ts = OffsetDateTime::from_unix_timestamp(ts.seconds)
        .unwrap()
        .replace_nanosecond(ts.nanos as u32)
        .unwrap();
    assert_partitioned_query(&mut tx, user_id_1, &now, &ts).await;
    assert_partitioned_query(&mut tx, user_id_2, &now, &ts).await;
    assert_partitioned_query(&mut tx, user_id_3, &now, &ts).await;
    assert_partitioned_read(&mut tx, user_id_1, &now, &ts).await;
    assert_partitioned_read(&mut tx, user_id_2, &now, &ts).await;
    assert_partitioned_read(&mut tx, user_id_3, &now, &ts).await;

    let stmt = Statement::new("SELECT * FROM User p WHERE p.UserId LIKE 'user_partitionx_%'");
    let mut rows = execute_partitioned_query(&mut tx, stmt).await;
    assert_eq!(20000, rows.len());
    let mut map = HashMap::<String, Row>::new();
    while let Some(row) = rows.pop() {
        let user_id = row.column_by_name("UserId").unwrap();
        map.insert(user_id, row);
    }

    (0..20000).for_each(|x| {
        let user_id = format!("user_partitionx_{x}");
        let ts = chunk_timestamp(&cr2, x, USER_ROWS_PER_COMMIT);
        assert_user_row(map.get(&user_id).unwrap(), &user_id, &now, &ts)
    });
}

async fn test_query(count: usize, prefix: &str) {
    let now = OffsetDateTime::now_utc();
    let mutations = (0..count)
        .map(|x| create_user_mutation(&format!("user_{prefix}_{x}"), &now))
        .collect();
    let data_client = create_data_client().await;
    // Chunked because a large `count` would otherwise exceed Spanner's per-transaction
    // mutation limit; each chunk lands at its own commit timestamp.
    let cr = apply_in_chunks(&data_client, mutations, USER_ROWS_PER_COMMIT).await;

    let mut tx = data_client.read_only_transaction().await.unwrap();
    let stmt = Statement::new(format!("SELECT *, Array[UserId,UserId,UserId,UserId,UserId] as UserIds, Array[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20] as NumArray FROM User p WHERE p.UserId LIKE 'user_{prefix}_%' ORDER BY UserId "));
    let rows = execute_query(&mut tx, stmt).await;
    assert_eq!(count, rows.len());

    let mut user_ids: Vec<String> = (0..count).map(|x| format!("user_{prefix}_{x}")).collect();
    user_ids.sort();
    for (x, user_id) in user_ids.iter().enumerate() {
        let row = rows.get(x).unwrap();
        // Sorted lexicographically, so recover the mutation index from the name to find
        // which chunk -- and so which commit timestamp -- this row was written in.
        let index: usize = user_id.rsplit('_').next().unwrap().parse().unwrap();
        let ts = chunk_timestamp(&cr, index, USER_ROWS_PER_COMMIT);
        assert_user_row(row, user_id, &now, &ts);
        let user_ids = row.column_by_name::<Vec<String>>("UserIds").unwrap();
        user_ids.iter().for_each(|u| assert_eq!(u, user_id));
        let nums = row.column_by_name::<Vec<i64>>("NumArray").unwrap();
        let mut start = 0_i64;
        assert_eq!(20, nums.len());
        nums.iter().for_each(|u| {
            start += 1;
            assert_eq!(*u, start)
        });
    }
}

#[tokio::test]
#[serial]
async fn test_few_records_value() {
    test_query(10, "few").await;
}

#[tokio::test]
#[serial]
async fn test_many_records_value() {
    test_query(40000, "many").await;
}

#[tokio::test]
#[serial]
async fn test_many_records_struct() {
    //set up test data
    let now = OffsetDateTime::now_utc();
    let data_client = create_data_client().await;
    let user_id = "user_x_6";
    let mutations = vec![create_user_mutation(user_id, &now)];
    let _ = data_client.apply(mutations).await.unwrap();
    let item_mutations = (0..4500).map(|x| create_user_item_mutation(user_id, x)).collect();
    let _ = data_client.apply(item_mutations).await.unwrap();
    let characters_mutations = (0..4500).map(|x| create_user_character_mutation(user_id, x)).collect();
    let _ = data_client.apply(characters_mutations).await.unwrap();

    //test
    let mut tx = data_client.read_only_transaction().await.unwrap();
    let mut stmt = Statement::new(
        "SELECT *,
        ARRAY(SELECT AS STRUCT * FROM UserItem WHERE UserId = p.UserId) as UserItem,
        ARRAY(SELECT AS STRUCT * FROM UserCharacter WHERE UserId = p.UserId) as UserCharacter
        FROM User p WHERE UserId = @UserId;",
    );
    stmt.add_param("UserId", &user_id);

    let mut rows = execute_query(&mut tx, stmt).await;
    assert_eq!(1, rows.len());
    let row = rows.pop().unwrap();
    let items = row.column_by_name::<Vec<UserItem>>("UserItem").unwrap();
    assert_eq!(4500, items.len());
    let characters = row.column_by_name::<Vec<UserCharacter>>("UserCharacter").unwrap();
    assert_eq!(4500, characters.len());
}

#[tokio::test]
#[serial]
async fn test_read_row() {
    //set up test data
    let now = OffsetDateTime::now_utc();
    let user_id = "user_x_x";
    let mutations = vec![create_user_mutation(user_id, &now)];
    let data_client = create_data_client().await;
    let _ = data_client.apply(mutations).await.unwrap();

    //test
    let mut tx = data_client.read_only_transaction().await.unwrap();
    let row = tx.read_row("User", &["UserId"], Key::new(&user_id)).await.unwrap();
    assert!(row.is_some())
}

#[tokio::test]
#[serial]
async fn test_read_multi_row() {
    //set up test data
    let now = OffsetDateTime::now_utc();
    let user_id = format!("user_x_{}", now.second());
    let user_id2 = format!("user_x_{}", &now.second() + 1);
    let mutations = vec![
        create_user_mutation(&user_id, &now),
        create_user_mutation(&user_id2, &now),
    ];
    let data_client = create_data_client().await;
    let _ = data_client.apply(mutations).await.unwrap();

    // test
    let mut tx = data_client.read_only_transaction().await.unwrap();
    let row = tx
        .read("User", &["UserId"], vec![Key::new(&user_id), Key::new(&user_id2)])
        .await
        .unwrap();
    assert_eq!(2, all_rows(row).await.unwrap().len());
}

#[tokio::test]
#[serial]
async fn test_big_decimal() {
    let client = create_data_client().await;
    let mut tx = client.read_only_transaction().await.unwrap();
    let stmt = Statement::new(
        "SELECT
                cast(\"-99999999999999999999999999999.999999999\" as numeric),
                cast(\"-99999999999999999999999999999\" as numeric),
                cast(\"-0.999999999\" as numeric),
                 cast(\"0\" as numeric),
                 cast(\"0.999999999\" as numeric),
                 cast(\"99999999999999999999999999999\" as numeric),
                 cast(\"99999999999999999999999999999.999999999\" as numeric)",
    );
    let mut iter = tx.query(stmt).await.unwrap();
    let row = iter.next().await.unwrap().unwrap();
    assert_eq!(
        "-99999999999999999999999999999.999999999",
        row.column::<BigDecimal>(0).unwrap().to_string()
    );
    assert_eq!(
        "-99999999999999999999999999999",
        row.column::<BigDecimal>(1).unwrap().to_string()
    );
    assert_eq!("-0.999999999", row.column::<BigDecimal>(2).unwrap().to_string());
    assert_eq!("0", row.column::<BigDecimal>(3).unwrap().to_string());
    assert_eq!("0.999999999", row.column::<BigDecimal>(4).unwrap().to_string());
    assert_eq!(
        "99999999999999999999999999999",
        row.column::<BigDecimal>(5).unwrap().to_string()
    );
    assert_eq!(
        "99999999999999999999999999999.999999999",
        row.column::<BigDecimal>(6).unwrap().to_string()
    );
}

#[tokio::test]
#[serial]
async fn test_query_plan_mode_metadata() {
    let data_client = create_data_client().await;
    let mut tx = data_client.read_only_transaction().await.unwrap();

    let stmt = Statement::new("SELECT UserId, NotNullINT64 FROM User LIMIT 1");
    let options = QueryOptions {
        mode: QueryMode::Plan,
        ..Default::default()
    };
    let mut iter = tx.query_with_option(stmt, options).await.unwrap();

    // Drain iterator — Plan mode returns no data rows, so next() returns None immediately.
    // Metadata is populated via try_recv() during this call.
    assert!(iter.next().await.unwrap().is_none());

    // Verify metadata is populated after draining despite no data rows
    let metadata = iter.columns_metadata();
    assert!(!metadata.is_empty(), "Plan mode should populate column metadata");
    assert_eq!(metadata.len(), 2);
    assert_eq!(metadata[0].name, "UserId");
    assert_eq!(metadata[1].name, "NotNullINT64");

    // Note: On real Spanner, Plan mode also returns stats with query_plan
    // (plan_nodes). The emulator does not return stats for Plan mode, so we
    // cannot assert on query_plan here. Verified manually against a real instance
    // that stats().query_plan is Some with populated plan_nodes.
}

#[tokio::test]
#[serial]
async fn test_query_empty_result_metadata() {
    let data_client = create_data_client().await;
    let mut tx = data_client.read_only_transaction().await.unwrap();

    // Query that returns 0 rows but should still have metadata
    let stmt = Statement::new("SELECT UserId, NotNullINT64 FROM User WHERE FALSE");
    let mut iter = tx.query(stmt).await.unwrap();

    // No rows returned; metadata is populated via try_recv() during this call.
    assert!(iter.next().await.unwrap().is_none());

    // Metadata should still be populated after draining
    let metadata = iter.columns_metadata();
    assert!(!metadata.is_empty(), "Empty result should still populate column metadata");
    assert_eq!(metadata.len(), 2);
    assert_eq!(metadata[0].name, "UserId");
    assert_eq!(metadata[1].name, "NotNullINT64");
}

#[tokio::test]
#[serial]
async fn test_query_profile_mode_empty_result_stats() {
    let data_client = create_data_client().await;
    let mut tx = data_client.read_only_transaction().await.unwrap();

    // Profile mode with 0 rows: values are empty, but stats should still be captured
    let stmt = Statement::new("SELECT UserId, NotNullINT64 FROM User WHERE FALSE");
    let options = QueryOptions {
        mode: QueryMode::Profile,
        ..Default::default()
    };
    let mut iter = tx.query_with_option(stmt, options).await.unwrap();

    // Drain iterator — no data rows returned.
    assert!(iter.next().await.unwrap().is_none());

    // Metadata should be populated after draining
    let metadata = iter.columns_metadata();
    assert!(
        !metadata.is_empty(),
        "Profile mode empty result should populate column metadata"
    );
    assert_eq!(metadata.len(), 2);
    assert_eq!(metadata[0].name, "UserId");
    assert_eq!(metadata[1].name, "NotNullINT64");

    // Stats (query_stats) should be available even with 0 rows
    let stats = iter.stats();
    assert!(stats.is_some(), "Profile mode should return stats even with empty results");
    assert!(stats.unwrap().query_stats.is_some(), "Profile mode should return query_stats");
}
