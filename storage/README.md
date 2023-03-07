# google-cloud-storage

Google Cloud Platform Storage Client library.

[![crates.io](https://img.shields.io/crates/v/google-cloud-storage.svg)](https://crates.io/crates/google-cloud-storage)

* [About Cloud Storage](https://cloud.google.com/storage/)
* [JSON API Documentation](https://cloud.google.com/storage/docs/json_api/v1)

## Installation

```
[dependencies]
google-cloud-storage = <version>
google-cloud-default = { version = <version>, features = ["storage"] }
```

## Quick Start

```rust
use google_cloud_storage::client::Client;
use google_cloud_storage::sign::SignedURLOptions;
use google_cloud_storage::sign::SignedURLMethod;
use google_cloud_storage::http::Error;
use google_cloud_storage::http::objects::download::Range;
use google_cloud_storage::http::objects::get::GetObjectRequest;
use google_cloud_storage::http::objects::upload::UploadObjectRequest;
use tokio::task::JoinHandle;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use google_cloud_default::WithAuthExt;

#[tokio::main]
async fn main() -> Result<(), Error> {

    // Create client.
    let config = ClientConfig::default().with_auth().await.unwrap();
    let mut client = Client::new(config);

    // Upload the file
    let upload_type = UploadType::Simple(Media::new("file.png"));
    let uploaded = client.upload_object(&UploadObjectRequest {
        bucket: "bucket".to_string(),
        ..Default::default()
    }, "hello world".as_bytes(), upload_type).await;

    // Download the file
    let data = client.download_object(&GetObjectRequest {
        bucket: "bucket".to_string(),
        object: "file.png".to_string(),
        ..Default::default()
   }, &Range::default()).await;

    // Create signed url.
    let url_for_download = client.signed_url("bucket", "foo.txt", SignedURLOptions::default());
    let url_for_upload = client.signed_url("bucket", "foo.txt", SignedURLOptions {
        method: SignedURLMethod::PUT,
        ..Default::default()
    });
    Ok(())
}
```
