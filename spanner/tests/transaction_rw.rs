use anyhow::Result;
use chrono::{NaiveDate, NaiveDateTime, Utc};
use google_cloud_googleapis::spanner::v1::commit_request::Transaction::SingleUseTransaction;
use google_cloud_googleapis::spanner::v1::Mutation;
use google_cloud_spanner::key::{Key, KeySet};
use google_cloud_spanner::mutation::insert_or_update;
use google_cloud_spanner::row::Row;
use google_cloud_spanner::statement::{Statement, ToKind};
use google_cloud_spanner::transaction::{CallOptions, QueryOptions};
use google_cloud_spanner::transaction_ro::{BatchReadOnlyTransaction, ReadOnlyTransaction};
use google_cloud_spanner::value::{CommitTimestamp, TimestampBound};
use rust_decimal::Decimal;
use serial_test::serial;
use std::ops::DerefMut;
use std::str::FromStr;

mod common;
use common::*;
use google_cloud_spanner::transaction_rw::ReadWriteTransaction;

#[tokio::test]
#[serial]
async fn test_mutation_and_statement() {
    let now = Utc::now().naive_utc();
    let mut session = create_session().await;

    let past_user = format!("user_{}", now.timestamp());
    replace_test_data(&mut session, vec![create_user_mutation(&past_user, &now)]).await;

    let mut tx = match ReadWriteTransaction::begin(session, CallOptions::default()).await {
        Ok(tx) => tx,
        Err(e) => panic!("error {:?}", e.status),
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
    match result {
        Ok(s) => {
            assert!(s.0.is_some());
            let ts = s.0.unwrap();
            println!(
                "commit time stamp is {}",
                NaiveDateTime::from_timestamp(ts.seconds, ts.nanos as u32).to_string()
            );
        }
        Err(e) => panic!("error {:?}", e.0),
    }

    // get by another transaction
    //    let mut session = create_session().await;
    //   let mut tx = match ReadWriteTransaction::begin(session, CallOptions::default()).await {
    //      Ok(tx) => tx,
    //     Err(e) => panic!("error {:?}", e.status)
    //};
}
