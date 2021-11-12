# google-cloud-spanner

Google Cloud Platform GCE spanner library.

[![crates.io](https://img.shields.io/crates/v/google-cloud-spanner.svg)](https://crates.io/crates/google-cloud-spanner)

* [About Cloud Spanner](https://cloud.google.com/spanner/)
* [API Documentation](https://cloud.google.com/spanner/docs)

## Installation

```
[dependencies]
google-cloud-spanner = 0.1.0
```

## Quick Start

Create `Client` and call transaction API same as [Google Cloud Go](https://github.com/googleapis/google-cloud-go/tree/main/spanner).

```rust
use google_cloud_spanner::client::Client;

#[tokio::main]
async fn main() {

    const DATABASE: &str = "projects/your_projects/instances/your-instance/databases/your-database";
   
    // Create spanner client
    let mut client = match Client::new(DATABASE, None).await {
        Ok(client) => client,
        Err(e) => { /* handle error */ }
    };
    
    //Reading transactions.
    client.single(); 
    client.read_only_transaction(); 
    client.batch_read_only_transaction();

    //Reading and writing transactions.
    client.apply();
    client.read_write_transaction();
    client.apply_at_least_once();
    client.partitioned_update();
}
```

## API

### Client 
