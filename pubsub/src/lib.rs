//! # google-cloud-pubsub
//!
//! Google Cloud Platform pub/sub library.
//!
//! * [About Cloud Pub/Sub](https://cloud.google.com/pubsub/)
//! * [Pub/Sub API Documentation](https://cloud.google.com/pubsub/docs)
//!
//! ## Quickstart
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
//! # use google_cloud_pubsub::client::ClientConfig;
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
//! # use google_cloud_pubsub::client::ClientConfig;
//! # use google_cloud_default::WithAuthExt;
//! # use google_cloud_gax::conn::Environment;
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
//! ### Emulator
//! For tests you can use the [Emulator-Option](https://docs.rs/google-cloud-gax/latest/google_cloud_gax/conn/enum.Environment.html#variant.GoogleCloud) like that:
//!
//! ```
//! # use google_cloud_auth::{credentials::CredentialsFile, project, token::DefaultTokenSourceProvider};
//! # use google_cloud_gax::conn::Environment;
//! # use google_cloud_pubsub::client::ClientConfig;
//! #
//! # async fn test() {
//! let config = ClientConfig {
//!     environment: Environment::Emulator("localhost:1234".into()),
//!     ..ClientConfig::default()
//! };
//! #
//! # let _ = config;
//! # }
//! ```
//!
//! ### Publish Message
//!
//! ```
//! use google_cloud_pubsub::client::{Client, ClientConfig};
//! use google_cloud_googleapis::pubsub::v1::PubsubMessage;
//! use google_cloud_pubsub::topic::TopicConfig;
//! use google_cloud_pubsub::subscription::SubscriptionConfig;
//! use google_cloud_gax::grpc::Status;
//! use tokio::task::JoinHandle;
//! use tokio_util::sync::CancellationToken;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Status> {
//!
//!     // Create pubsub client.
//!     // `use google_cloud_default::WithAuthExt;` is required to use default authentication.
//!     let config = ClientConfig::default();//.with_auth().await.unwrap();
//!     let client = Client::new(config).await.unwrap();
//!
//!     // Create topic.
//!     let topic = client.topic("test-topic");
//!     if !topic.exists(None).await? {
//!         topic.create(None, None).await?;
//!     }
//!
//!     // Start publisher.
//!     let publisher = topic.new_publisher(None);
//!
//!     // Publish message.
//!     let tasks : Vec<JoinHandle<Result<String,Status>>> = (0..10).into_iter().map(|_i| {
//!         let publisher = publisher.clone();
//!         tokio::spawn(async move {
//!             let msg = PubsubMessage {
//!                data: "abc".into(),
//!                // Set ordering_key if needed (https://cloud.google.com/pubsub/docs/ordering)
//!                ordering_key: "order".into(),
//!                ..Default::default()
//!             };
//!
//!             // Send a message. There are also `publish_bulk` and `publish_immediately` methods.
//!             let mut awaiter = publisher.publish(msg).await;
//!
//!             // The get method blocks until a server-generated ID or an error is returned for the published message.
//!             awaiter.get().await
//!         })
//!     }).collect();
//!
//!     // Wait for all publish task finish
//!     for task in tasks {
//!         let message_id = task.await.unwrap()?;
//!     }
//!
//!     // Wait for publishers in topic finish.
//!     let mut publisher = publisher;
//!     publisher.shutdown();
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Subscribe Message
//!
//! ```
//! use google_cloud_pubsub::client::{Client, ClientConfig};
//! use google_cloud_googleapis::pubsub::v1::PubsubMessage;
//! use google_cloud_pubsub::subscription::SubscriptionConfig;
//! use google_cloud_gax::grpc::Status;
//! use std::time::Duration;
//! use tokio_util::sync::CancellationToken;
//! use futures_util::StreamExt;
//! // use google_cloud_default::WithAuthExt;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Status> {
//!
//!     // Create pubsub client.
//!     // `with_auth` is the trait defined at google-cloud-default crate.
//!     let config = ClientConfig::default();//.with_auth().await.unwrap();
//!     let client = Client::new(config).await.unwrap();
//!
//!     // Get the topic to subscribe to.
//!     let topic = client.topic("test-topic");
//!
//!     // Create subscription
//!     // If subscription name does not contain a "/", then the project is taken from client above. Otherwise, the
//!     // name will be treated as a fully qualified resource name
//!     let config = SubscriptionConfig {
//!         // Enable message ordering if needed (https://cloud.google.com/pubsub/docs/ordering)
//!         enable_message_ordering: true,
//!         ..Default::default()
//!     };
//!
//!     // Create subscription
//!     let subscription = client.subscription("test-subscription");
//!     if !subscription.exists(None).await? {
//!         subscription.create(topic.fully_qualified_name(), config, None).await?;
//!     }
//!
//!     // Token for cancel.
//!     let cancel = CancellationToken::new();
//!     let cancel2 = cancel.clone();
//!     tokio::spawn(async move {
//!         // Cancel after 10 seconds.
//!         tokio::time::sleep(Duration::from_secs(10)).await;
//!         cancel2.cancel();
//!     });
//!
//!     // Receive blocks until the ctx is cancelled or an error occurs.
//!     // Or simply use the `subscription.subscribe` method.
//!     subscription.receive(|mut message, cancel| async move {
//!         // Handle data.
//!         println!("Got Message: {:?}", message.message.data);
//!
//!         // Ack or Nack message.
//!         message.ack().await;
//!     }, cancel.clone(), None).await;
//!
//!     // Delete subscription if needed.
//!     subscription.delete(None).await;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Subscribe Message (Alternative Way)
//!
//! ```no_run
//! use google_cloud_pubsub::client::{Client, ClientConfig};
//! use google_cloud_googleapis::pubsub::v1::PubsubMessage;
//! use google_cloud_pubsub::subscription::SubscriptionConfig;
//! use google_cloud_gax::grpc::Status;
//! use std::time::Duration;
//! use tokio_util::sync::CancellationToken;
//! use futures_util::StreamExt;
//! // use google_cloud_default::WithAuthExt;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Status> {
//!
//!     // Creating Client, Topic and Subscription...
//! #
//! #     let config = ClientConfig::default();//.with_auth().await.unwrap();
//! #     let client = Client::new(config).await.unwrap();
//! #
//! #     let topic = client.topic("test-topic");
//! #
//! #     let config = SubscriptionConfig {
//! #         // Enable message ordering if needed (https://cloud.google.com/pubsub/docs/ordering)
//! #         enable_message_ordering: true,
//! #         ..Default::default()
//! #     };
//! #
//! #     let subscription = client.subscription("test-subscription");
//! #     if !subscription.exists(None).await? {
//! #         subscription.create(topic.fully_qualified_name(), config, None).await?;
//! #     }
//!
//!     // Read the messages as a stream
//!     // (needs futures_util::StreamExt as import)
//!     // Note: This blocks the current thread but helps working with non clonable data
//!     let mut stream = subscription.subscribe(None).await?;
//!     while let Some(message) = stream.next().await {
//!         // Handle data.
//!         println!("Got Message: {:?}", message.message);
//!
//!         // Ack or Nack message.
//!         message.ack().await;
//!     }
//! #
//! #    // Delete subscription if needed.
//! #    subscription.delete(None).await;
//! #
//! #    Ok(())
//! }
//! ```
pub mod apiv1;
pub mod client;
pub mod publisher;
pub mod subscriber;
pub mod subscription;
pub mod topic;
pub mod util;
