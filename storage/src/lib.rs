//! # google-cloud-storage
//!
//! Google Cloud Platform Storage Client library.
//!
//! * [About Cloud Storage](https://cloud.google.com/storage/)
//! * [JSON API Documentation](https://cloud.google.com/storage/docs/json_api/v1)
//!
//! ## Quick Start
//!
//! There are two ways to create a client that is authenticated against the google cloud.
//!
//! The crate [google-cloud-default](https://crates.io/crates/google-cloud-default) provides two
//! methods that help implementing those.
//!
//! #### Automatically
//!
//! The function `with_auth()` will try and read the credentials from a file specified in the environment variable `GOOGLE_APPLICATION_CREDENTIALS`, `GOOGLE_APPLICATION_CREDENTIALS_JSON` or
//! from a metadata server.
//!
//! This is also described in [google-cloud-auth](https://github.com/yoshidan/google-cloud-rust/blob/main/foundation/auth/README.md)
//!
//! See [implementation](https://docs.rs/google-cloud-auth/0.9.1/src/google_cloud_auth/token.rs.html#59-74)
//!
//! ```
//! # use google_cloud_storage::client::ClientConfig;
//! # use google_cloud_default::WithAuthExt;
//! #
//! # async fn test() {
//! let config = ClientConfig::default().with_auth().await.unwrap();
//! # let _ = config;
//! # }
//! ```
//!
//! ### Manually
//!
//! When you cant use the `gcloud` authentication but you have a different way to get your credentials (e.g a different environment variable)
//! you can parse your own version of the 'credentials-file' and use it like that:
//!
//! ```
//! # use google_cloud_auth::{credentials::CredentialsFile, project, token::DefaultTokenSourceProvider};
//! # use google_cloud_storage::client::ClientConfig;
//! # use google_cloud_default::WithAuthExt;
//! #
//! # async fn test() {
//! let creds = CredentialsFile {
//!     // Add your credentials here
//! #    tp: "".to_owned(),
//! #    project_id: None,
//! #    private_key_id: None,
//! #    private_key: None,
//! #    client_email: None,
//! #    client_id: None,
//! #    auth_uri: None,
//! #    token_uri: None,
//! #    client_secret: None,
//! #    audience: None,
//! #    subject_token_type: None,
//! #    token_url_external: None,
//! #    token_info_url: None,
//! #    service_account_impersonation_url: None,
//! #    credential_source: None,
//! #    quota_project_id: None,
//! #    refresh_token: None,
//! };
//!
//! let config = ClientConfig::default().with_credentials(creds).await.unwrap();
//! #
//! # let _ = config;
//! # }
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
//! // use google_cloud_default::WithAuthExt;
//! // let config = ClientConfig::default().with_auth().await?;
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
//!     // Create signed url.
//!     let url_for_download = client.signed_url("bucket", "foo.txt", SignedURLOptions::default());
//!     let url_for_upload = client.signed_url("bucket", "foo.txt", SignedURLOptions {
//!         method: SignedURLMethod::PUT,
//!         ..Default::default()
//!     });
//!     Ok(())
//! }
//! ```

extern crate core;

pub mod client;
pub mod http;
pub mod sign;
