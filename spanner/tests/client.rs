use anyhow::Context;
use google_cloud_spanner::client::{Client, TxError};
use google_cloud_spanner::mutation::insert;
use google_cloud_spanner::statement::{Statement, ToKind};

const DATABASE: &str = "projects/local-project/instances/test-instance/databases/local-database";

#[tokio::test]
async fn test_new() -> Result<(), anyhow::Error> {
    let client = Client::new(DATABASE, None).await.context("error")?;
    let value = client
        .read_write_transaction(
            |mut tx| async move {
                let result = async {
                    let tx2 = &mut tx;
                    let stmt = Statement::new("UPDATE");
                    tx2.buffer_write(vec![insert("Table", vec!["Attr1"], vec!["a".to_kind()])]);
                    tx2.update(stmt, None).await.map_err(TxError::TonicStatus)
                }
                .await;
                return (tx, result);
            },
            None,
        )
        .await;
    println!("{:?}", value.err());
    Ok(())
}
