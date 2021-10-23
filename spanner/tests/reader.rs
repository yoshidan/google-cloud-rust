mod common;
use common::*;
use chrono::Utc;
use google_cloud_spanner::transaction_ro::ReadOnlyTransaction;
use google_cloud_spanner::value::TimestampBound;
use google_cloud_spanner::transaction::CallOptions;
use google_cloud_spanner::statement::Statement;
use google_cloud_spanner::reader::AsyncIterator;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_many_records() {
    let now = Utc::now().naive_utc();
    let mut session = create_session().await;
    let mutations = (0..20000).map(|x| create_user_mutation(&format!("user_many_{}", x), &now)).collect();
    println!("create 20000 user mutation");
    let cr1= replace_test_data(&mut session, mutations).await.unwrap();
  //  let item_mutations = (0..20000).map(|x| create_user_item_mutation(&format!("user_many_{}", x), x)).collect();
   // println!("create 20000 user item");
   // let cr2 =  replace_test_data(&mut session, item_mutations).await.unwrap();
    //let characters_mutations = (0..20000).map(|x| create_user_character_mutation(&format!("user_many_{}", x), x)).collect();
    //println!("create 20000 user character");
    //let cr3=  replace_test_data(&mut session, characters_mutations).await.unwrap();

    let mut tx = match ReadOnlyTransaction::begin(
        session,
        TimestampBound::strong_read(),
        CallOptions::default(),
    )
        .await
    {
        Ok(tx) => tx,
        Err(status) => panic!("begin error {:?}", status),
    };

    let mut stmt = Statement::new("SELECT * FROM User p WHERE p.UserId LIKE 'user_many_%'");
    println!("execute query 20000 records");
    let mut reader = match tx.query(stmt, None).await {
        Ok(reader) => reader,
        Err(status) => panic!("query error {:?}", status),
    };
    let mut rows = vec![];
    println!("query executed start read");
    loop {
        let row = match reader.next().await {
            Ok(row) => row,
            Err(status) => panic!("query error {:?}", status),
        };
        let row = match row{
            Some(row) => row,
            None => break
        };
        rows.push(row);
    }
    assert_eq!(20000, rows.len());
}