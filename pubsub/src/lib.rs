//! # google-cloud-pubsub
//!
//! Google Cloud Platform pub/sub library.
//!
//! * [About Cloud Pub/Sub](https://cloud.google.com/pubsub/)
//! * [Pub/Sub API Documentation](https://cloud.google.com/pubsub/docs)
//!
//! ## Quick Start
//!
//! ### Publish Message
//!
//! ```
//! use google_cloud_pubsub::client::Client;
//! use google_cloud_googleapis::Status;
//! use tokio_util::sync::CancellationToken;
//! use google_cloud_googleapis::pubsub::v1::PubsubMessage;
//! use google_cloud_pubsub::subscription::SubscriptionConfig;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Status> {
//!
//!     // Create pubsub client.
//!     let mut client = Client::new("local-project", None).await.unwrap();
//!
//!     // Token for cancel.
//!     let ctx = CancellationToken::new();
//!
//!     // Create topic.
//!     let mut topic = client.topic("test-topic");
//!     if !topic.exists(ctx.clone(), None).await? {
//!         topic.create(ctx.clone(), None).await?;
//!     }
//!
//!     // Start publisher.
//!     topic.run(None);
//!     let topic = topic;
//!
//!     // Publish message.
//!     let tasks = (0..10).into_iter().map(|_i| {
//!         let topic = topic.clone();
//!         let ctx = ctx.clone();
//!         tokio::spawn(async move {
//!             let mut awaiter = topic.publish(PubsubMessage {
//!                 data: "abc".as_bytes().to_vec(),
//!                 attributes: Default::default(),
//!                 message_id: "".to_string(),
//!                 publish_time: None,
//!                 ordering_key: "".to_string()
//!             }).await;
//!             // The get method blocks until a server-generated ID or an error is returned for the published message.
//!             let message_id = awaiter.get(ctx).await.unwrap();
//!         })
//!     });
//!
//!     // Wait for all publish task finish
//!     for task in tasks {
//!         task.await;
//!     }
//!
//!     // Wait for publishers in topic finish.
//!     topic.shutdown();
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Subscribe Message
//!
//! ```
//! use google_cloud_pubsub::client::Client;
//! use google_cloud_googleapis::Status;
//! use tokio_util::sync::CancellationToken;
//! use google_cloud_googleapis::pubsub::v1::PubsubMessage;
//! use google_cloud_pubsub::subscription::SubscriptionConfig;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Status> {
//!
//!     // Create pubsub client
//!     use std::time::Duration;
//!     let mut client = Client::new("local-project", None).await.unwrap();
//!
//!     // Token for cancel.
//!     let ctx = CancellationToken::new();
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
//!     if !subscription.exists(ctx.clone(), None).await? {
//!         subscription.create(ctx.clone(), topic.fully_qualified_name(), config, None).await?;
//!     }
//!     let ctx2 = ctx.clone();
//!     tokio::spawn(async move {
//!         // Cancel after 10 seconds.
//!         tokio::time::sleep(Duration::from_secs(10)).await;
//!         ctx2.cancel();
//!     });
//!
//!     // Receive blocks until the ctx is cancelled or an error occurs.
//!     subscription.receive(ctx.clone(), |mut message, ctx| async move {
//!         // Handle data.
//!         let data = message.message.data.as_slice();
//!         println!("{:?}", data);
//!
//!         // Ack or Nack message.
//!         message.ack().await;
//!     }, None).await;
//!
//!     // Delete subscription if needed.
//!     subscription.delete(ctx, None).await;
//!
//!     Ok(())
//! }
//! ```
pub mod apiv1;
pub mod topic;
pub mod subscription;
pub mod publisher;
pub mod subscriber;
pub mod client;
pub mod util;