#![allow(clippy::result_large_err)]
//! # google-cloud-storage
//!
//! Google Cloud Platform Storage Client library.
//!
//! * [About Cloud Storage](https://cloud.google.com/storage/)
//! * [JSON API Documentation](https://cloud.google.com/storage/docs/json_api/v1)
//!
//! ## Quick Start
//!
//! ### Authentication
//! There are two ways to create a client that is authenticated against the google cloud.
//!
//! #### Automatically
//!
//! The function `with_auth()` will try and read the credentials from a file specified in the environment variable `GOOGLE_APPLICATION_CREDENTIALS`, `GOOGLE_APPLICATION_CREDENTIALS_JSON` or
//! from a metadata server.
//!
//! This is also described in [google-cloud-auth](https://github.com/yoshidan/google-cloud-rust/blob/main/foundation/auth/README.md)
//!
//! ```
//! use google_cloud_storage::client::{ClientConfig, Client};
//!
//! async fn run() {
//!     let config = ClientConfig::default().with_auth().await.unwrap();
//!     let client = Client::new(config);
//! }
//! ```
//!
//! #### Manually
//!
//! When you cant use the `gcloud` authentication but you have a different way to get your credentials (e.g a different environment variable)
//! you can parse your own version of the 'credentials-file' and use it like that:
//!
//! ```
//! use google_cloud_auth::credentials::CredentialsFile;
//! // or google_cloud_storage::client::google_cloud_auth::credentials::CredentialsFile
//! use google_cloud_storage::client::{ClientConfig, Client};
//!
//! async fn run(cred: CredentialsFile) {
//!     let config = ClientConfig::default().with_credentials(cred).await.unwrap();
//!     let client = Client::new(config);
//! }
//! ```
//!
//! ### Anonymous Access
//!
//! To provide [anonymous access without authentication](https://cloud.google.com/storage/docs/authentication), do the following.
//!
//! ```rust
//! use google_cloud_storage::client::{ClientConfig, Client};
//!
//! async fn run() {
//!     let config = ClientConfig::default().anonymous();
//!     let client = Client::new(config);
//! }
//! ```
//!
//! ### Usage
//!
//! ```
//! use google_cloud_storage::client::Client;
//! use google_cloud_storage::client::ClientConfig;
//! use google_cloud_storage::sign::SignedURLOptions;
//! use google_cloud_storage::sign::SignedURLMethod;
//! use google_cloud_storage::http::Error;
//! use google_cloud_storage::http::objects::download::Range;
//! use google_cloud_storage::http::objects::get::GetObjectRequest;
//! use google_cloud_storage::http::objects::upload::{Media, UploadObjectRequest, UploadType};
//! use tokio::task::JoinHandle;
//! use std::fs::File;
//! use std::io::BufReader;
//! use std::io::Read;
//!
//! async fn run(config: ClientConfig) -> Result<(), Error> {
//!
//!     // Create client.
//!     let mut client = Client::new(config);
//!
//!     // Upload the file
//!     let upload_type = UploadType::Simple(Media::new("file.png"));
//!     let uploaded = client.upload_object(&UploadObjectRequest {
//!         bucket: "bucket".to_string(),
//!         ..Default::default()
//!     }, "hello world".as_bytes(), &upload_type).await;
//!
//!     // Download the file
//!     let data = client.download_object(&GetObjectRequest {
//!         bucket: "bucket".to_string(),
//!         object: "file.png".to_string(),
//!         ..Default::default()
//!    }, &Range::default()).await;
//!
//!     // Create signed url with the default key and google-access-id of the client
//!     let url_for_download = client.signed_url("bucket", "foo.txt", None, None, SignedURLOptions::default());
//!     let url_for_upload = client.signed_url("bucket", "foo.txt", None, None, SignedURLOptions {
//!         method: SignedURLMethod::PUT,
//!         ..Default::default()
//!     });
//!
//!     Ok(())
//! }
//! ```

extern crate core;

pub mod client;
pub mod http;
pub mod sign;
