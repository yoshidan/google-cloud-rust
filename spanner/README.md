# google-cloud-spanner

Google Cloud Platform spanner library.

[![crates.io](https://img.shields.io/crates/v/google-cloud-spanner.svg)](https://crates.io/crates/google-cloud-spanner)

* [About Cloud Spanner](https://cloud.google.com/spanner/)
* [Spanner API Documentation](https://cloud.google.com/spanner/docs)
* [Rust client Documentation](https://docs.rs/google-cloud-spanner/0.3.0/google_cloud_spanner/)

## Installation

```
[dependencies]
google-cloud-spanner = <version>
```

## Quick Start

Create `Client` and call transaction API same as [Google Cloud Go](https://github.com/googleapis/google-cloud-go/tree/main/spanner).

```rust
use google_cloud_spanner::client::Client;
use google_cloud_spanner::client::Client;
use google_cloud_spanner::mutation::insert;
use google_cloud_spanner::statement::Statement;
use google_cloud_spanner::reader::AsyncIterator;
use google_cloud_spanner::value::CommitTimestamp;

#[tokio::main]
async fn main() -> Result<(), Error> {

    const DATABASE: &str = "projects/your_project/instances/your-instance/databases/your-database";
   
    // Create spanner client
    let mut client = match Client::new(DATABASE).await?;

    // Insert 
    let mutation = insert("Table", &["col1", "col2", "col3"], &[&1, &"strvalue", &CommitTimestamp::new()]);
    let commit_timestamp = client.apply(vec![mutation]).await?;
    
    // Read with query
    let mut stmt = Statement::new("SELECT col2 FROM Table WHERE col1 = @col1");
    stmt.add_param("col1",&1);
    let iter = client.single().await?.query(stmt).await?;
    while let some(row) = iter.next().await? {
        let col2 = row.column_by_name::<String>("col2");
    }
}
```

## Example
Here is the example with using Warp.
* https://github.com/yoshidan/google-cloud-rust-example/tree/main/spanner/rust

## Performance 

Result of the 24 hours Load Test.

| Metrics | This library | [Google Cloud Go](https://github.com/googleapis/google-cloud-go/tree/main/spanner) | 
| -------- | ----------------| ----------------- |
| RPS | 438.4 | 443.4 |
| Used vCPU | 0.37 ~ 0.42 | 0.65 ~ 0.70 |

* This Library : [Performance report](https://storage.googleapis.com/0432808zbaeatxa/report_1637760853.008414.html) / [CPU Usage](https://storage.googleapis.com/0432808zbaeatxa/CPU%20(6).png)
* Google Cloud Go : [Performance report](https://storage.googleapis.com/0432808zbaeatxa/report_1637673736.2540932.html) / [CPU Usage](https://storage.googleapis.com/0432808zbaeatxa/CPU%20(5).png)

Test condition 
* 2.0 vCPU GKE Autopilot Pod
* 1 Node spanner database server
* 100 Users
* [Here](https://github.com/yoshidan/google-cloud-rust-example/tree/main/spanner) is the application for Load Test.