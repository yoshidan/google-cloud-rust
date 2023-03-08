//! # google-cloud-pubsub
//!
//! Google Cloud Platform pub/sub library.
//!
//! * [About Cloud Pub/Sub](https://cloud.google.com/pubsub/)
//! * [Pub/Sub API Documentation](https://cloud.google.com/pubsub/docs)
//!
//! ## Quick Start
//!
//! You can use [google-cloud-default](https://crates.io/crates/google-cloud-default) to create `ClientConfig`
//!
//! ### Publish Message
//!
//! ```
//! use google_cloud_pubsub::client::{Client, ClientConfig};
//! use google_cloud_gax::cancel::CancellationToken;
//! use google_cloud_googleapis::pubsub::v1::PubsubMessage;
//! use google_cloud_pubsub::topic::TopicConfig;
//! use google_cloud_pubsub::subscription::SubscriptionConfig;
//! use google_cloud_gax::grpc::Status;
//! use tokio::task::JoinHandle;
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
//!             let mut msg = PubsubMessage::default();
//!             msg.data = "abc".into();
//!             // Set ordering_key if needed (https://cloud.google.com/pubsub/docs/ordering)
//!             // msg.ordering_key = "order".into();
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
//! use google_cloud_gax::cancel::CancellationToken;
//! use google_cloud_googleapis::pubsub::v1::PubsubMessage;
//! use google_cloud_pubsub::subscription::SubscriptionConfig;
//! use google_cloud_gax::grpc::Status;
//! use std::time::Duration;
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
//!     // Configure subscription.
//!     let mut config = SubscriptionConfig::default();
//!     // Enable message ordering if needed (https://cloud.google.com/pubsub/docs/ordering)
//!     config.enable_message_ordering = true;
//!
//!     // Create subscription
//!     let subscription = client.subscription("test-subscription");
//!     if !subscription.exists(None).await? {
//!         subscription.create(topic.fully_qualified_name(), config, None).await?;
//!     }
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
