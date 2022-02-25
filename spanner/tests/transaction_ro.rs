use chrono::{DateTime, TimeZone, Timelike, Utc};
use google_cloud_spanner::key::Key;

use google_cloud_spanner::row::Row;
use google_cloud_spanner::statement::Statement;
use google_cloud_spanner::transaction::CallOptions;
use google_cloud_spanner::transaction_ro::{BatchReadOnlyTransaction, ReadOnlyTransaction};
use google_cloud_spanner::value::TimestampBound;
use serial_test::serial;
use std::ops::DerefMut;

mod common;
use common::*;

use std::collections::HashMap;
use tokio_util::sync::CancellationToken;

async fn assert_read(
    tx: &mut ReadOnlyTransaction,
    user_id: &str,
    now: &DateTime<Utc>,
    cts: &DateTime<Utc>,
) {
    let reader = match tx
        .read(
            CancellationToken::new(),
            "User",
            &user_columns(),
            Key::key(&user_id),
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
    now: &DateTime<Utc>,
    cts: &DateTime<Utc>,
) {
    let mut stmt = Statement::new("SELECT * FROM User WHERE UserId = @UserID");
    stmt.add_param("UserId", &user_id);
    let mut rows = execute_query(tx, stmt).await;
    assert_eq!(1, rows.len(), "row must exists");
    let row = rows.pop().unwrap();
    assert_user_row(&row, user_id, now, cts);
}

async fn execute_query(tx: &mut ReadOnlyTransaction, stmt: Statement) -> Vec<Row> {
    let reader = match tx.query(CancellationToken::new(), stmt).await {
        Ok(tx) => tx,
        Err(status) => panic!("query error {:?}", status),
    };
    all_rows(reader).await
}

#[tokio::test]
#[serial]
async fn test_query_and_read() {
    let now = Utc::now();
    let mut session = create_session().await;
    let user_id_1 = "user_1";
    let user_id_2 = "user_2";
    let user_id_3 = "user_3";
    let cr = replace_test_data(
        session.deref_mut(),
        vec![
            create_user_mutation(user_id_1, &now),
            create_user_mutation(user_id_2, &now),
            create_user_mutation(user_id_3, &now),
        ],
    )
    .await
    .unwrap();

    let mut tx = read_only_transaction(session).await;
    let ts = cr.commit_timestamp.as_ref().unwrap();
    let ts = Utc.timestamp(ts.seconds, ts.nanos as u32);
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
    let now = Utc::now();
    let mut session = create_session().await;
    let user_id_1 = "user_10";
    let cr = replace_test_data(
        session.deref_mut(),
        vec![
            create_user_mutation(user_id_1, &now),
            create_user_item_mutation(user_id_1, 1),
            create_user_item_mutation(user_id_1, 2),
            create_user_character_mutation(user_id_1, 10),
            create_user_character_mutation(user_id_1, 20),
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
    stmt.add_param("UserId", &user_id_1);
    let mut rows = execute_query(&mut tx, stmt).await;
    assert_eq!(1, rows.len());
    let row = rows.pop().unwrap();

    // check UserTable
    let ts = cr.commit_timestamp.as_ref().unwrap();
    let ts = Utc.timestamp(ts.seconds, ts.nanos as u32);
    assert_user_row(&row, user_id_1, &now, &ts);

    let mut user_items = row.column_by_name::<Vec<UserItem>>("UserItem").unwrap();
    let first_item = user_items.pop().unwrap();
    assert_eq!(first_item.user_id, user_id_1);
    assert_eq!(first_item.item_id, 2);
    assert_eq!(first_item.quantity, 100);
    assert_ne!(
        DateTime::<Utc>::from(first_item.updated_at).to_string(),
        now.to_string()
    );
    let second_item = user_items.pop().unwrap();
    assert_eq!(second_item.user_id, user_id_1);
    assert_eq!(second_item.item_id, 1);
    assert_eq!(second_item.quantity, 100);
    assert_ne!(
        DateTime::<Utc>::from(second_item.updated_at).to_string(),
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
        DateTime::<Utc>::from(first_character.updated_at).to_string(),
        now.to_string()
    );
    let second_character = user_characters.pop().unwrap();
    assert_eq!(second_character.user_id, user_id_1);
    assert_eq!(second_character.character_id, 10);
    assert_eq!(second_character.level, 1);
    assert_ne!(
        DateTime::<Utc>::from(second_character.updated_at).to_string(),
        now.to_string()
    );
    assert!(user_characters.is_empty());
}

#[tokio::test]
#[serial]
async fn test_batch_partition_query_and_read() {
    let now = Utc::now();
    let mut session = create_session().await;
    let user_id_1 = "user_1";
    let user_id_2 = "user_2";
    let user_id_3 = "user_3";
    let cr = replace_test_data(
        session.deref_mut(),
        vec![
            create_user_mutation(user_id_1, &now),
            create_user_mutation(user_id_2, &now),
            create_user_mutation(user_id_3, &now),
        ],
    )
    .await
    .unwrap();

    let many = (0..20000)
        .map(|x| create_user_mutation(&format!("user_partitionx_{}", x), &now))
        .collect();
    let cr2 = replace_test_data(session.deref_mut(), many).await.unwrap();

    let mut tx = match BatchReadOnlyTransaction::begin(
        CancellationToken::new(),
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
    let ts = Utc.timestamp(ts.seconds, ts.nanos as u32);
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

    let ts = cr2.commit_timestamp.as_ref().unwrap();
    let ts = Utc.timestamp(ts.seconds, ts.nanos as u32);
    (0..20000).for_each(|x| {
        let user_id = format!("user_partitionx_{}", x);
        assert_user_row(map.get(&user_id).unwrap(), &user_id, &now, &ts)
    });
}

#[tokio::test]
#[serial]
async fn test_many_records() {
    let now = Utc::now();
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
    let ts = Utc.timestamp(ts.seconds, ts.nanos as u32);
    let mut user_ids: Vec<String> = (0..40000).map(|x| format!("user_many_{}", x)).collect();
    user_ids.sort();
    for (x, user_id) in user_ids.iter().enumerate() {
        let row = rows.get(x).unwrap();
        assert_user_row(row, user_id, &now, &ts);
        let user_ids = row.column_by_name::<Vec<String>>("UserIds").unwrap();
        user_ids.iter().for_each(|u| assert_eq!(u, user_id));
    }
}

#[tokio::test]
#[serial]
async fn test_many_records_struct() {
    let now = Utc::now();
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
    stmt.add_param("UserId", &user_id);

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

#[tokio::test]
#[serial]
async fn test_read_row() {
    let now = Utc::now();
    let mut session = create_session().await;
    let user_id = "user_x_x";
    let mutations = vec![create_user_mutation(user_id, &now)];
    let _ = replace_test_data(&mut session, mutations).await.unwrap();

    let mut tx = read_only_transaction(session).await;
    let row = tx
        .read_row(
            CancellationToken::new(),
            "User",
            &["UserId"],
            Key::key(&user_id),
        )
        .await
        .unwrap();
    assert!(row.is_some())
}

#[tokio::test]
#[serial]
async fn test_read_multi_row() {
    let now = Utc::now();
    let mut session = create_session().await;
    let user_id = format!("user_x_{}", &now.second());
    let user_id2 = format!("user_x_{}", &now.second() + 1);
    let mutations = vec![
        create_user_mutation(&user_id, &now),
        create_user_mutation(&user_id2, &now),
    ];
    let _ = replace_test_data(&mut session, mutations).await.unwrap();

    let mut tx = read_only_transaction(session).await;
    let row = tx
        .read(
            CancellationToken::new(),
            "User",
            &["UserId"],
            vec![Key::key(&user_id), Key::key(&user_id2)],
        )
        .await
        .unwrap();
    assert_eq!(2, all_rows(row).await.len());
}
