# google-cloud-bigquery

Google Cloud Platform BigQuery Client library.

[![crates.io](https://img.shields.io/crates/v/google-cloud-bigquery.svg)](https://crates.io/crates/google-cloud-bigquery)

## Installation

```
[dependencies]
google-cloud-bigquery = <version>
google-cloud-default = { version = <version>, features = ["bigquery"] }
```

## Quick Start

### CreateClient

The function `create()` will try and read the credentials from a file specified in the environment variable `GOOGLE_APPLICATION_CREDENTIALS`, `GOOGLE_APPLICATION_CREDENTIALS_JSON` or
from a metadata server.

This is also described in [google-cloud-auth](https://github.com/yoshidan/google-cloud-rust/blob/main/foundation/auth/README.md)

```rust
use google_cloud_pubsub::client::{ClientConfig, Client};
use google_cloud_default::WithAuthExt;

async fn run() {
    let (config, project_id) = ClientConfig::create().await.unwrap();
    let client = Client::new(config).await.unwrap();
}
```

When you can't use the `gcloud` authentication but you have a different way to get your credentials (e.g a different environment variable)
you can parse your own version of the 'credentials-file' and use it like that:

```rust
use google_cloud_auth::credentials::CredentialsFile;
use google_cloud_bigquery::client::{ClientConfig, Client};
use google_cloud_default::biqquery::CreateAuthExt;

async fn run(cred: CredentialsFile) {
    let (config, project_id) = ClientConfig::create_with_credentials(cred).await.unwrap();
    let client = Client::new(config).await.unwrap();
}
```

### Read Data

#### Query
```rust
use google_cloud_bigquery::http::job::query::QueryRequest;
use google_cloud_bigquery::query::row::Row;
use google_cloud_bigquery::client::Client;

async fn run(client: &Client, project_id: &str) {
    let request = QueryRequest {
        query: "SELECT * FROM dataset.table".to_string(),
        ..Default::default()
    };
    let iter = client.query(project_id, request).await.unwrap();
    while let Some(row) = iter.next::<Row>().await.unwrap() {
        let col1 = row.column::<String>(0);
        let col2 = row.column::<Option<String>>(1);
    }
}
```

#### Read Table
```rust
use google_cloud_bigquery::storage::row::Row;
use google_cloud_bigquery::client::Client;

async fn run(client: &Client, project_id: &str) {
    let table = TableReference {
        project_id: project_id.to_string(),
        dataset_id: "dataset".to_string(),
        table_id: "table".to_string(),
    };
    let iter = client.read_table::<Row>(&table, None).await.unwrap();
    while let Some(row) = iter.next().await.unwrap() {
        let col1 = row.column::<String>(0);
        let col2 = row.column::<Option<String>>(1);
    }
}
```

#### Values 
Default supported types to decode by `row.column::<T>()` are
* String (for STRING)
* bool (for BOOL)
* i64 (for INT64)
* f64 (for FLOAT)
* bigdecimal::BigDecimal (for NUMERIC, BIGNUMERIC)
* Vec<u8> (for BINARY)
* time::OffsetDateTime (for TIMESTAMP)
* time::Date (for DATE)
* time::Time (for TIME)
* T: StructDecodable (for STRUCT)
  - [Example](https://github.com/yoshidan/google-cloud-rust/blob/ac291953e52987294d65ef5127cfc5ba8b582bb4/bigquery/src/http/bigquery_client.rs#L132)
* Option (for all NULLABLE)
* Vec (for ARRAY)

### Insert Data

```rust
use google_cloud_bigquery::http::tabledata::insert_all::{InsertAllRequest, Row};
use google_cloud_bigquery::client::Client;

#[derive(serde::Serialize)]
pub struct TestData {
    pub col1: String,
    #[serde(with = "time::serde::rfc3339::option")]
    pub col_timestamp: Option<OffsetDateTime>,
    // Must serialize as base64 string to insert binary data
    // #[serde(default, with = "Base64Standard")]
    pub col_binary: Vec<u8> 
}

async fn run(client: &Client, project_id: &str, data: TestData) {
    let data1 = Row {
        insert_id: None,
        json: data,
    };
    let request = InsertAllRequest {
        rows: vec![data1],
        ..Default::default()
    };
    let result = client.tabledata().insert(project_id, "dataset", "table", request).await.unwrap();
    let error = result.insert_errors; 
}
```

## Features
### HTTP API 
* [x] [job](https://cloud.google.com/bigquery/docs/reference/rest/v2/jobs)
* [x] [tabledata](https://cloud.google.com/bigquery/docs/reference/rest/v2/tabledata)
* [x] [dataset](https://cloud.google.com/bigquery/docs/reference/rest/v2/datasets)
* [x] [table](https://cloud.google.com/bigquery/docs/reference/rest/v2/tables)
* [x] [model](https://cloud.google.com/bigquery/docs/reference/rest/v2/models)
* [x] [routine](https://cloud.google.com/bigquery/docs/reference/rest/v2/routines)
* [x] [rowAccessPolicy](https://cloud.google.com/bigquery/docs/reference/rest/v2/rowAccessPolicies)
### Streaming
* [x] [Storage Read API](https://cloud.google.com/bigquery/docs/reference/storage)
* [ ] [Storage Write API](https://cloud.google.com/bigquery/docs/write-api)
