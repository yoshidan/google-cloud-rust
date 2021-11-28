# google-cloud-metadata

Google Cloud Platform GCE check library.

[![crates.io](https://img.shields.io/crates/v/google-cloud-metadata.svg)](https://crates.io/crates/google-cloud-metadata)

## Installation

```
[dependencies]
google-cloud-metadata = 0.1.1
```

## Usage 
```rust
use google_cloud_metadata::*;

#[tokio::test]
async fn test_on_gce() {
    // true: server is running on the GCP such as GCE and GKE.
    let result = on_gce().await;
    assert_eq!(true, result);
}
```
