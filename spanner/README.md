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

### Read Operation
```rust
use google_cloud_metadata::*;

#[tokio::main]
async fn main() {

    const DATABASE: &str = "projects/your_projects/instances/your-instance/databases/your-database";
   
    // Create spanner client
    let client = Arc::new(Client::new(DATABASE, None).await.unwrap());
    let hello = warp::path!("read").and_then(move || read_handler(client.clone()));

    warp::serve(hello)
        .run(([127, 0, 0, 1], 3030))
        .await;
}


async fn read_handler(client: Arc<Client>) -> Result<impl Reply, Rejection> {
    // Create read only transaction
    let tx = match client.read_only_transaction(None).await {
        Ok(tx) => tx,
        Err(e) => {
            Ok(warp::reply::with_status(
                warp::reply::html(e.message().to_string()),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    };

    match read(tx).await {
        Ok(rows) => {
            Ok(warp::reply::with_status(
                warp::reply::html(format!("length={}", rows.len())),
                warp::http::StatusCode::OK,
            ))
        },
        Err(e) => {
            Ok(warp::reply::with_status(
                warp::reply::html(e.message().to_string()),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

async fn read(mut tx: ReadOnlyTransaction) -> Result<Vec<String>, tonic::Status> {
    
    // Execute query and get all rows
    let mut stmt = Statement::new("SELECT UserID, UpdatedAt AS UpdatedAt FROM User limit 10");
    let mut reader = tx.query(stmt,None).await?;

    loop {
        let mut data = vec![];
        let row = match reader.next().await? {
            Some(row) => row,
            None => return Ok(data),
        };
        let user_id = row.column_by_name::<String>("User").map_err(|e| Status::aborted(e.to_string()))?;
        data.push(user_id);
    };
}
```

## Guidline 
