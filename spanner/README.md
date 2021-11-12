# google-cloud-spanner

Google Cloud Platform GCE spanner library.

[![crates.io](https://img.shields.io/crates/v/google-cloud-spanner.svg)](https://crates.io/crates/google-cloud-spanner)

## Installation

```
[dependencies]
google-cloud-spanner = 0.1.0
```

## Quick Start

Here is the quick start with using [Warp](https://github.com/seanmonstar/warp)

```rust
use google_cloud_metadata::*;

#[tokio::main]
async fn main() {

    const DATABASE: &str = "projects/your_projects/instances/your-instance/databases/your-database";

    log::info!("start server");

    let mut config = ClientConfig::default();
    let client = Arc::new(Client::new(DATABASE, Some(config)).await.unwrap());
    let hello = warp::path!("hello" / String).and_then(move |name| sample_handler(name, client.clone()));

    warp::serve(hello)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

pub async fn sample_handler(name: String, client: Arc<Client>) -> Result<impl Reply, Rejection> {

    //
    let mut tx = client.single().await.unwrap();
    let mut stmt = Statement::new("SELECT ARRAY(SELECT AS STRUCT UserID AS UserID, UpdatedAt AS UpdatedAt) As User FROM User limit 1");
    let mut reader = tx.query(stmt,None).await.unwrap();
    loop {
        match reader.next().await {
            Ok(record) => {
                if record.is_none() {
                    break;
                }
                let mut r = record.unwrap();
                let users = r.column_by_name::<Vec<Vec<String>>>("User").await
            },
            Err(e ) => println!("read error {:?}", e)
        }
    }
    Ok(warp::reply::with_status(warp::reply::html("ok"),warp::http::StatusCode::OK))
}
```

## Guidline 
