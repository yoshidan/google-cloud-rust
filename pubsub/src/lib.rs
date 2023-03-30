//! # google-cloud-pubsub
//!
//! Google Cloud Platform pub/sub library.
//!
//! * [About Cloud Pub/Sub](https://cloud.google.com/pubsub/)
//! * [Pub/Sub API Documentation](https://cloud.google.com/pubsub/docs)
//!
//! ## Quickstart
//!
//! ### Authentication
//!
//! When you are not using an emulator you'll need to be authenticated.
//! There are two ways to do that:
//!
//! #### Automatically
//! You can use [google-cloud-default](https://crates.io/crates/google-cloud-default) to create [ClientConfig][crate::client::ClientConfig]
//!
//! This will try and read the credentials from a file specified in the environment variable `GOOGLE_APPLICATION_CREDENTIALS_JSON` or
//! from a metadata server.
//!
//! This is also described in [google-cloud-auth](https://github.com/yoshidan/google-cloud-rust/blob/main/foundation/auth/README.md)
//!
//! See [implementation](https://docs.rs/google-cloud-auth/0.9.1/src/google_cloud_auth/token.rs.html#59-74)
//!
//! #### Manually
//!
//! When you cant use the `gcloud` authentication but you have a different way to get your credentials (e.g a different environment variable)
//! you can parse your own version of the 'credentials-file' and use it like that:
//!
//! ```
//! let creds = Box::new(CredentialsFile {
//!     // add your parsed creds here
//! });
//!
//! let project_conf = project::Config {
//!     audience: Some(AUDIENCE),
//!     scopes: Some(&SCOPES),
//! };
//!
//! // build your own TokenSourceProvider
//! let token_source = DefaultTokenSourceProvider::new_with_credentials(project_conf, creds)
//!     .await?;
//!
//! // use that provider to authenticate yourself against the google cloud
//! let config = ClientConfig {
//!     project_id: token_source.project_id.clone(),
//!     environment: Environment::GoogleCloud(Box::new(token_source)),
//!     ..ClientConfig::default()
//! };
//! ```
//!
//! ### Emulator
//! For tests you can use the [Emulator-Option](https://docs.rs/google-cloud-gax/latest/google_cloud_gax/conn/enum.Environment.html#variant.GoogleCloud) like that:
//!
//! ```
//! let config = ClientConfig {
//!     project_id: token_source.project_id.clone(),
//!     environment: Environment::Emulator("localhost:1234".into()),
//!     ..ClientConfig::default()
//! };
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
//! // Client config
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
//!         let data = message.message.data.as_ref();
//!         println!("{:?}", data);
//!
//!         // Ack or Nack message.
//!         message.ack().await;
//!     }, cancel.clone(), None).await;
//!
//!     // Alternativly you can use the messages as a stream
//!     // (needs futures_util::StreamExt as import)
//!     // Note: This blocks the current thread but helps working with non clonable data
//!     let mut stream = subscription.subscribe(None).await?();
//!     while let Some(message) = stream.next().await {
//!         // Handle data.
//!         let data = message.message.data.as_ref();
//!         println!("{:?}", data);
//!
//!         // Ack or Nack message.
//!         message.ack().await;
//!     }
//!
//!     // Delete subscription if needed.
//!     subscription.delete(None).await;
//!
//!     Ok(())
//! }
//! ```
pub mod apiv1;
pub mod client;
pub mod publisher;
pub mod subscriber;
pub mod subscription;
pub mod topic;
pub mod util;
