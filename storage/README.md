# google-cloud-storage

Google Cloud Platform Storage Client library.

[![crates.io](https://img.shields.io/crates/v/google-cloud-storage.svg)](https://crates.io/crates/google-cloud-storage)

* [About Cloud Storage](https://cloud.google.com/storage/)
* [JSON API Documentation](https://cloud.google.com/storage/docs/json_api/v1)

## Installation

```
[dependencies]
google-cloud-storage = <version>
```

## Quick Start

```rust
use google_cloud_storage::client::Client;
use google_cloud_storage::client::http::Error;
use google_cloud_storage::sign::SignedURLOptions;
use google_cloud_storage::sign::SignedURLMethod;
use google_cloud_storage::http::objects::download::Range;
use google_cloud_storage::http::objects::get::GetObjectRequest;
use google_cloud_storage::http::objects::upload::UploadObjectRequest;
use tokio::task::JoinHandle;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;

#[tokio::main]
async fn main() -> Result<(), Error> {

    // Create client.
    let mut client = Client::default().await.unwrap();

    // Upload the file
    let uploaded = client.upload_object(&UploadObjectRequest {
        bucket: "bucket".to_string(),
        name: "file.png".to_string(),
        ..Default::default()
    }, "hello world".as_bytes(), "application/octet-stream", None).await;

    // Download the file
    let data = client.download_object(&GetObjectRequest {
        bucket: "bucket".to_string(),
        object: "file.png".to_string(),
        ..Default::default()
   }, &Range::default(), None).await;

    // Create signed url.
    let url_for_download = client.signed_url("bucket", "foo.txt", SignedURLOptions::default());
    let url_for_upload = client.signed_url("bucket", "foo.txt", SignedURLOptions {
        method: SignedURLMethod::PUT,
        ..Default::default()
    });
    Ok(())
}
```
