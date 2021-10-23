use anyhow::Result;
use chrono::{NaiveDate, NaiveDateTime, Utc};
use google_cloud_googleapis::spanner::v1::commit_request::Transaction::SingleUseTransaction;
use google_cloud_googleapis::spanner::v1::Mutation;
use google_cloud_spanner::key::{Key, KeySet};
use google_cloud_spanner::mutation::insert_or_update;
use google_cloud_spanner::row::Row;
use google_cloud_spanner::statement::{Statement, ToKind};
use google_cloud_spanner::transaction::{CallOptions, QueryOptions};
use serial_test::serial;

mod common;
use common::*;
use google_cloud_spanner::reader::{AsyncIterator, RowIterator};
use google_cloud_spanner::transaction_rw::ReadWriteTransaction;
use google_cloud_spanner::value::TimestampBound;
use tonic::Status;

pub async fn all_rows(mut itr: RowIterator<'_>) -> Result<Vec<Row>, Status> {
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
            Err(status) => return Err(status),
        };
    }
    Ok(rows)
}

#[tokio::test]
#[serial]
async fn test_mutation_and_statement() {
    let now = Utc::now().naive_utc();
    let mut session = create_session().await;

    let past_user = format!("user_{}", now.timestamp());
    let cr = replace_test_data(&mut session, vec![create_user_mutation(&past_user, &now)])
        .await
        .unwrap();

    let mut tx = match ReadWriteTransaction::begin(session, CallOptions::default()).await {
        Ok(tx) => tx,
        Err(e) => panic!("begin first error {:?}", e.status),
    };
    let result = async {
        let user_id_1 = "user_rw_1";
        let user_id_2 = "user_rw_2";
        let user_id_3 = "user_rw_3";
        tx.buffer_write(vec![create_user_mutation(user_id_1, &now)]);
        tx.buffer_write(vec![create_user_mutation(user_id_2, &now)]);
        tx.buffer_write(vec![create_user_mutation(user_id_3, &now)]);

        let mut stmt1 = Statement::new("INSERT INTO UserCharacter (UserId,CharacterId,Level,UpdatedAt) VALUES(@UserId,1,1,PENDING_COMMIT_TIMESTAMP())");
        stmt1.add_param("UserId", past_user.clone());
        let mut stmt2 = Statement::new("INSERT INTO UserItem (UserId,ItemId,Quantity,UpdatedAt) VALUES(@UserId,10,1000,PENDING_COMMIT_TIMESTAMP())");
        stmt2.add_param("UserId", past_user.clone());
        tx.update(stmt1, None).await?;
        return tx.update(stmt2, None).await;
    }.await;

    let result = tx.finish(result, None).await;
    let commit_timestamp = match result {
        Ok(s) => {
            assert!(s.0.is_some());
            let ts = s.0.unwrap();
            let naive = NaiveDateTime::from_timestamp(ts.seconds, ts.nanos as u32);
            println!("commit time stamp is {}", naive.to_string());
            naive
        }
        Err(e) => panic!("error {:?}", e.0),
    };

    let ts = cr.commit_timestamp.as_ref().unwrap();
    let ts = NaiveDateTime::from_timestamp(ts.seconds, ts.nanos as u32);
    assert_data(&past_user, &now, &ts, &commit_timestamp).await;
}

#[tokio::test]
#[serial]
async fn test_partitioned_dml() {
    let now = Utc::now().naive_utc();
    let mut session = create_session().await;

    let user_id = format!("user_{}", now.timestamp());
    let cr = replace_test_data(&mut session, vec![create_user_mutation(&user_id, &now)])
        .await
        .unwrap();

    let mut tx =
        match ReadWriteTransaction::begin_partitioned_dml(session, CallOptions::default()).await {
            Ok(tx) => tx,
            Err(e) => panic!("begin first error {:?}", e.status),
        };
    let result = async {
        let stmt1 = Statement::new(
            "UPDATE User SET NullableString = 'aaa' WHERE NullableString IS NOT NULL",
        );
        tx.update(stmt1, None).await
    }
    .await;

    // partition dml doesn't support commit/rollback
    assert!(result.is_ok());

    let session = create_session().await;
    let mut tx = read_only_transaction(session).await;
    let reader = tx
        .read(
            "User",
            vec!["NullableString"],
            KeySet::from(Key::one(user_id.clone())),
            None,
        )
        .await
        .unwrap();
    let row = all_rows(reader).await.unwrap().pop().unwrap();
    let value = row.column_by_name::<String>("NullableString").unwrap();
    assert_eq!(value, "aaa");
}

#[tokio::test]
#[serial]
async fn test_rollback() {
    let now = Utc::now().naive_utc();
    let mut session = create_session().await;

    let past_user = format!("user_{}", now.timestamp());
    let cr = replace_test_data(&mut session, vec![create_user_mutation(&past_user, &now)])
        .await
        .unwrap();

    let mut tx = match ReadWriteTransaction::begin(session, CallOptions::default()).await {
        Ok(tx) => tx,
        Err(e) => panic!("begin first error {:?}", e.status),
    };
    let result = async {
        let mut stmt1 =
            Statement::new("UPDATE User SET NullableString = 'aaaaaaa' WHERE UserId = @UserId");
        stmt1.add_param("UserId", past_user.clone());
        tx.update(stmt1, None).await?;

        let stmt2 = Statement::new("UPDATE UserNoteFound SET Quantity = 10000");
        tx.update(stmt2, None).await
    }
    .await;

    let _ = tx.finish(result, None).await;
    let session = create_session().await;
    let mut tx = read_only_transaction(session).await;
    let reader = tx
        .read(
            "User",
            user_columns(),
            KeySet::from(Key::one(past_user.clone())),
            None,
        )
        .await
        .unwrap();
    let row = all_rows(reader).await.unwrap().pop().unwrap();
    let ts = cr.commit_timestamp.as_ref().unwrap();
    let ts = NaiveDateTime::from_timestamp(ts.seconds, ts.nanos as u32);
    assert_user_row(&row, &past_user, &now, &ts);
}

async fn assert_data(
    user_id: &String,
    now: &NaiveDateTime,
    user_commit_timestamp: &NaiveDateTime,
    commit_timestamp: &NaiveDateTime,
) {
    // get by another transaction
    let session = create_session().await;
    let mut tx = match ReadWriteTransaction::begin(session, CallOptions::default()).await {
        Ok(tx) => tx,
        Err(e) => panic!("begin second error {:?}", e.status),
    };
    let result = async {
        let mut stmt = Statement::new(
            "SELECT *,
        ARRAY(SELECT AS STRUCT * FROM UserItem WHERE UserId = p.UserId) as UserItem,
        ARRAY(SELECT AS STRUCT * FROM UserCharacter WHERE UserId = p.UserId) as UserCharacter,
        FROM User p WHERE UserId = @UserId;
    ",
        );
        stmt.add_param("UserId", user_id.clone());
        let result = tx.query(stmt, None).await?;
        all_rows(result).await
    }
    .await;

    // commit or rollback is required for rw transaction
    let rows = match tx.finish(result, None).await {
        Ok(s) => s.1,
        Err(e) => panic!("tx error {:?}", e.0),
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
        first_item.updated_at.timestamp.to_string(),
        commit_timestamp.to_string()
    );
    assert!(user_items.is_empty());

    let mut user_characters = row
        .column_by_name::<Vec<UserCharacter>>("UserCharacter")
        .unwrap();
    let first_character = user_characters.pop().unwrap();
    assert_eq!(first_character.user_id, *user_id);
    assert_eq!(first_character.character_id, 1);
    assert_eq!(first_character.level, 1);
    assert_eq!(
        first_character.updated_at.timestamp.to_string(),
        commit_timestamp.to_string()
    );
    assert!(user_characters.is_empty());
}
