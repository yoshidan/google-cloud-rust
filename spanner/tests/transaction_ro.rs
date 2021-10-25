use chrono::{NaiveDateTime, Utc};
use google_cloud_spanner::key::{Key, KeySet};
use google_cloud_spanner::mutation::insert_or_update;
use google_cloud_spanner::row::Row;
use google_cloud_spanner::statement::Statement;
use google_cloud_spanner::transaction::{CallOptions, QueryOptions};
use google_cloud_spanner::transaction_ro::{BatchReadOnlyTransaction, ReadOnlyTransaction};
use google_cloud_spanner::value::TimestampBound;
use serial_test::serial;
use std::ops::DerefMut;

mod common;
use common::*;
use google_cloud_spanner::reader::{AsyncIterator, RowIterator};

async fn all_rows(mut itr: RowIterator<'_>) -> Vec<Row> {
    let mut rows = vec![];
    loop {
        match itr.next().await {
            Ok(row) => {
                if row.is_some() {
                    rows.push(row.unwrap());
                } else {
                    break;
                }
            }
            Err(status) => panic!("reader aborted {:?}", status),
        };
    }
    rows
}

async fn assert_read(
    tx: &mut ReadOnlyTransaction,
    user_id: &str,
    now: &NaiveDateTime,
    cts: &NaiveDateTime,
) {
    let reader = match tx
        .read(
            "User",
            user_columns(),
            KeySet::from(Key::one(user_id)),
            None,
        )
        .await
    {
        Ok(tx) => tx,
        Err(status) => panic!("read error {:?}", status),
    };
    let mut rows = all_rows(reader).await;
    assert_eq!(1, rows.len(), "row must exists");
    let row = rows.pop().unwrap();
    assert_user_row(&row, user_id, now, cts);
}

async fn assert_query(
    tx: &mut ReadOnlyTransaction,
    user_id: &str,
    now: &NaiveDateTime,
    cts: &NaiveDateTime,
) {
    let mut stmt = Statement::new("SELECT * FROM User WHERE UserId = @UserID");
    stmt.add_param("UserId", user_id);
    let mut rows = execute_query(tx, stmt).await;
    assert_eq!(1, rows.len(), "row must exists");
    let row = rows.pop().unwrap();
    assert_user_row(&row, user_id, now, cts);
}

async fn execute_query(tx: &mut ReadOnlyTransaction, stmt: Statement) -> Vec<Row> {
    let reader = match tx.query(stmt, Some(QueryOptions::default())).await {
        Ok(tx) => tx,
        Err(status) => panic!("query error {:?}", status),
    };
    all_rows(reader).await
}

async fn assert_partitioned_query(
    tx: &mut BatchReadOnlyTransaction,
    user_id: &str,
    now: &NaiveDateTime,
    cts: &NaiveDateTime,
) {
    let mut stmt = Statement::new("SELECT * FROM User WHERE UserId = @UserID");
    stmt.add_param("UserId", user_id);
    let row = execute_partitioned_query(tx, stmt).await;
    assert_eq!(row.len(), 1);
    assert_user_row(row.first().unwrap(), user_id, now, cts);
}

async fn execute_partitioned_query(tx: &mut BatchReadOnlyTransaction, stmt: Statement) -> Vec<Row> {
    let partitions = match tx.partition_query(stmt, None, None).await {
        Ok(tx) => tx,
        Err(status) => panic!("query error {:?}", status),
    };
    println!("partition count = {}", partitions.len());
    let mut rows = vec![];
    for p in partitions.into_iter() {
        let reader = match tx.execute(p).await {
            Ok(tx) => tx,
            Err(status) => panic!("query error {:?}", status),
        };
        let rows_per_partition = all_rows(reader).await;
        for x in rows_per_partition {
            rows.push(x);
        }
    }
    rows
}

async fn assert_partitioned_read(
    tx: &mut BatchReadOnlyTransaction,
    user_id: &str,
    now: &NaiveDateTime,
    cts: &NaiveDateTime,
) {
    let partitions = match tx
        .partition_read(
            "User",
            user_columns(),
            KeySet::from(Key::one(user_id)),
            None,
            None,
        )
        .await
    {
        Ok(tx) => tx,
        Err(status) => panic!("query error {:?}", status),
    };
    println!("partition count = {}", partitions.len());
    let mut rows = vec![];
    for p in partitions.into_iter() {
        let reader = match tx.execute(p).await {
            Ok(tx) => tx,
            Err(status) => panic!("query error {:?}", status),
        };
        let rows_per_partition = all_rows(reader).await;
        for x in rows_per_partition {
            rows.push(x);
        }
    }
    assert_eq!(rows.len(), 1);
    assert_user_row(rows.first().unwrap(), user_id, now, cts);
}

#[tokio::test]
#[serial]
async fn test_query_and_read() {
    let now = Utc::now().naive_utc();
    let mut session = create_session().await;
    let user_id_1 = "user_1";
    let user_id_2 = "user_2";
    let user_id_3 = "user_3";
    let cr = replace_test_data(
        session.deref_mut(),
        vec![
            create_user_mutation(&user_id_1, &now),
            create_user_mutation(&user_id_2, &now),
            create_user_mutation(&user_id_3, &now),
        ],
    )
    .await
    .unwrap();

    let mut tx = read_only_transaction(session).await;
    let ts = cr.commit_timestamp.as_ref().unwrap();
    let ts = NaiveDateTime::from_timestamp(ts.seconds, ts.nanos as u32);
    assert_query(&mut tx, user_id_1, &now, &ts).await;
    assert_query(&mut tx, user_id_2, &now, &ts).await;
    assert_query(&mut tx, user_id_3, &now, &ts).await;
    assert_read(&mut tx, user_id_1, &now, &ts).await;
    assert_read(&mut tx, user_id_2, &now, &ts).await;
    assert_read(&mut tx, user_id_3, &now, &ts).await;
}

#[tokio::test]
#[serial]
async fn test_complex_query() {
    let now = Utc::now().naive_utc();
    let mut session = create_session().await;
    let user_id_1 = "user_10";
    let cr = replace_test_data(
        session.deref_mut(),
        vec![
            create_user_mutation(&user_id_1, &now),
            create_user_item_mutation(&user_id_1, 1),
            create_user_item_mutation(&user_id_1, 2),
            create_user_character_mutation(&user_id_1, 10),
            create_user_character_mutation(&user_id_1, 20),
        ],
    )
    .await
    .unwrap();

    let mut tx = read_only_transaction(session).await;
    let mut stmt = Statement::new(
        "SELECT *,
        ARRAY(SELECT AS STRUCT * FROM UserItem WHERE UserId = p.UserId) as UserItem,
        ARRAY(SELECT AS STRUCT * FROM UserCharacter WHERE UserId = p.UserId) as UserCharacter,
        FROM User p WHERE UserId = @UserId;
    ",
    );
    stmt.add_param("UserId", user_id_1);
    let mut rows = execute_query(&mut tx, stmt).await;
    assert_eq!(1, rows.len());
    let row = rows.pop().unwrap();

    // check UserTable
    let ts = cr.commit_timestamp.as_ref().unwrap();
    let ts = NaiveDateTime::from_timestamp(ts.seconds, ts.nanos as u32);
    assert_user_row(&row, user_id_1, &now, &ts);

    let mut user_items = row.column_by_name::<Vec<UserItem>>("UserItem").unwrap();
    let first_item = user_items.pop().unwrap();
    assert_eq!(first_item.user_id, user_id_1);
    assert_eq!(first_item.item_id, 2);
    assert_eq!(first_item.quantity, 100);
    assert_ne!(first_item.updated_at.timestamp.to_string(), now.to_string());
    let second_item = user_items.pop().unwrap();
    assert_eq!(second_item.user_id, user_id_1);
    assert_eq!(second_item.item_id, 1);
    assert_eq!(second_item.quantity, 100);
    assert_ne!(
        second_item.updated_at.timestamp.to_string(),
        now.to_string()
    );
    assert!(user_items.is_empty());

    let mut user_characters = row
        .column_by_name::<Vec<UserCharacter>>("UserCharacter")
        .unwrap();
    let first_character = user_characters.pop().unwrap();
    assert_eq!(first_character.user_id, user_id_1);
    assert_eq!(first_character.character_id, 20);
    assert_eq!(first_character.level, 1);
    assert_ne!(
        first_character.updated_at.timestamp.to_string(),
        now.to_string()
    );
    let second_character = user_characters.pop().unwrap();
    assert_eq!(second_character.user_id, user_id_1);
    assert_eq!(second_character.character_id, 10);
    assert_eq!(second_character.level, 1);
    assert_ne!(
        second_character.updated_at.timestamp.to_string(),
        now.to_string()
    );
    assert!(user_characters.is_empty());
}

#[tokio::test]
#[serial]
async fn test_batch_partition_query_and_read() {
    let now = Utc::now().naive_utc();
    let mut session = create_session().await;
    let user_id_1 = "user_1";
    let user_id_2 = "user_2";
    let user_id_3 = "user_3";
    let cr = replace_test_data(
        session.deref_mut(),
        vec![
            create_user_mutation(&user_id_1, &now),
            create_user_mutation(&user_id_2, &now),
            create_user_mutation(&user_id_3, &now),
        ],
    )
    .await
    .unwrap();

    let many = (0..20000)
        .map(|x| create_user_mutation(&format!("user_partition_{}", x), &now))
        .collect();
    let _cr2 = replace_test_data(session.deref_mut(), many).await.unwrap();

    let mut tx = match BatchReadOnlyTransaction::begin(
        session,
        TimestampBound::strong_read(),
        CallOptions::default(),
    )
    .await
    {
        Ok(tx) => tx,
        Err(status) => panic!("begin error {:?}", status),
    };

    let ts = cr.commit_timestamp.as_ref().unwrap();
    let ts = NaiveDateTime::from_timestamp(ts.seconds, ts.nanos as u32);
    assert_partitioned_query(&mut tx, user_id_1, &now, &ts).await;
    assert_partitioned_query(&mut tx, user_id_2, &now, &ts).await;
    assert_partitioned_query(&mut tx, user_id_3, &now, &ts).await;
    assert_partitioned_read(&mut tx, user_id_1, &now, &ts).await;
    assert_partitioned_read(&mut tx, user_id_2, &now, &ts).await;
    assert_partitioned_read(&mut tx, user_id_3, &now, &ts).await;

    let stmt = Statement::new("SELECT * FROM User p WHERE p.UserId LIKE 'user_partition_%'");
    let rows = execute_partitioned_query(&mut tx, stmt).await;
    assert_eq!(20000, rows.len());
    (0..20000).for_each(|x| {
        assert_user_row(
            rows.get(x).unwrap(),
            &format!("user_partition_{}", x),
            &now,
            &ts,
        )
    });
}

#[tokio::test]
#[serial]
async fn test_many_records() {
    let now = Utc::now().naive_utc();
    let mut session = create_session().await;
    let mutations = (0..40000)
        .map(|x| create_user_mutation(&format!("user_many_{}", x), &now))
        .collect();
    let cr = replace_test_data(&mut session, mutations).await.unwrap();

    let mut tx = read_only_transaction(session).await;
    let stmt = Statement::new("SELECT *, Array[UserId,UserId,UserId,UserId,UserId] as UserIds FROM User p WHERE p.UserId LIKE 'user_many_%' ORDER BY UserId ");
    let rows = execute_query(&mut tx, stmt).await;
    assert_eq!(40000, rows.len());

    let ts = cr.commit_timestamp.as_ref().unwrap();
    let ts = NaiveDateTime::from_timestamp(ts.seconds, ts.nanos as u32);
    let mut user_ids: Vec<String> = (0..40000).map(|x| format!("user_many_{}", x)).collect();
    user_ids.sort();
    for (x, user_id) in user_ids.iter().enumerate() {
        let row = rows.get(x).unwrap();
        assert_user_row(&row, user_id, &now, &ts);
        let user_ids = row.column_by_name::<Vec<String>>("UserIds").unwrap();
        user_ids.iter().for_each(|u| assert_eq!(u, user_id));
    }
}

#[tokio::test]
#[serial]
async fn test_many_records_struct() {
    let now = Utc::now().naive_utc();
    let mut session = create_session().await;
    let user_id = "user_x_6";
    let mutations = vec![create_user_mutation(user_id, &now)];
    let _ = replace_test_data(&mut session, mutations).await.unwrap();
    let item_mutations = (0..5000)
        .map(|x| create_user_item_mutation(user_id, x))
        .collect();
    let _ = replace_test_data(&mut session, item_mutations)
        .await
        .unwrap();
    let characters_mutations = (0..5000)
        .map(|x| create_user_character_mutation(user_id, x))
        .collect();
    let _ = replace_test_data(&mut session, characters_mutations)
        .await
        .unwrap();

    let mut tx = read_only_transaction(session).await;
    let mut stmt = Statement::new(
        "SELECT *,
        ARRAY(SELECT AS STRUCT * FROM UserItem WHERE UserId = p.UserId) as UserItem,
        ARRAY(SELECT AS STRUCT * FROM UserCharacter WHERE UserId = p.UserId) as UserCharacter
        FROM User p WHERE UserId = @UserId;",
    );
    stmt.add_param("UserId", user_id.clone());

    let mut rows = execute_query(&mut tx, stmt).await;
    assert_eq!(1, rows.len());
    let row = rows.pop().unwrap();
    let items = row.column_by_name::<Vec<UserItem>>("UserItem").unwrap();
    assert_eq!(5000, items.len());
    let characters = row
        .column_by_name::<Vec<UserCharacter>>("UserCharacter")
        .unwrap();
    assert_eq!(5000, characters.len());
}
