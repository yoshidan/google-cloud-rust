# google-cloud-pubsub

Google Cloud Platform pub/sub library.

[![crates.io](https://img.shields.io/crates/v/gcloud-pubsub.svg)](https://crates.io/crates/gcloud-pubsub)


* [About Cloud Pub/Sub](https://cloud.google.com/pubsub/)
* [Pub/Sub API Documentation](https://cloud.google.com/pubsub/docs)

## Installation

```toml
[dependencies]
google-cloud-pubsub = { package="gcloud-pubsub", version="1.0.0" }
```

## Quickstart

### Authentication
There are two ways to create a client that is authenticated against the google cloud.

#### Automatically

The function `with_auth()` will try and read the credentials from a file specified in the environment variable `GOOGLE_APPLICATION_CREDENTIALS`, `GOOGLE_APPLICATION_CREDENTIALS_JSON` or
from a metadata server.

This is also described in [google-cloud-auth](https://github.com/yoshidan/google-cloud-rust/blob/main/foundation/auth/README.md)

```rust
use google_cloud_pubsub::client::{ClientConfig, Client};

async fn run() {
    let config = ClientConfig::default().with_auth().await.unwrap();
    let client = Client::new(config).await.unwrap();
}
```

### Manually

When you can't use the `gcloud` authentication but you have a different way to get your credentials (e.g a different environment variable)
you can parse your own version of the 'credentials-file' and use it like that:

```rust
use google_cloud_auth::credentials::CredentialsFile;
// or google_cloud_pubsub::client::google_cloud_auth::credentials::CredentialsFile
use google_cloud_pubsub::client::{ClientConfig, Client};

async fn run(cred: CredentialsFile) {
    let config = ClientConfig::default().with_credentials(cred).await.unwrap();
    let client = Client::new(config).await.unwrap();
}
```

### Emulator
For tests, you can use the [Emulator-Option](https://github.com/yoshidan/google-cloud-rust/blob/cbd5ed1315d7b828c89a50fe71fcbaf15ddc964b/pubsub/src/client.rs#L32) like that:
Before executing the program, specify the address of the emulator in the following environment variable.

```sh
export PUBSUB_EMULATOR_HOST=localhost:8681
```

### Publish Message

```rust
use google_cloud_pubsub::client::{Client, ClientConfig};
use google_cloud_googleapis::pubsub::v1::PubsubMessage;
use google_cloud_pubsub::topic::TopicConfig;
use google_cloud_pubsub::subscription::SubscriptionConfig;
use google_cloud_gax::grpc::Status;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

async fn run(config: ClientConfig) -> Result<(), Status> {

    // Create pubsub client.
    let client = Client::new(config).await.unwrap();

    // Create topic.
    let topic = client.topic("test-topic");
    if !topic.exists(None).await? {
        topic.create(None, None).await?;
    }

    // Start publisher.
    let publisher = topic.new_publisher(None);

    // Publish message.
    let tasks : Vec<JoinHandle<Result<String,Status>>> = (0..10).into_iter().map(|_i| {
        let publisher = publisher.clone();
        tokio::spawn(async move {
            let msg = PubsubMessage {
               data: "abc".into(),
               // Set ordering_key if needed (https://cloud.google.com/pubsub/docs/ordering)
               ordering_key: "order".into(),
               ..Default::default()
            };

            // Send a message. There are also `publish_bulk` and `publish_immediately` methods.
            let mut awaiter = publisher.publish(msg).await;

            // The get method blocks until a server-generated ID or an error is returned for the published message.
            awaiter.get().await
        })
    }).collect();

    // Wait for all publish task finish
    for task in tasks {
        let message_id = task.await.unwrap()?;
    }

    // Wait for publishers in topic finish.
    let mut publisher = publisher;
    publisher.shutdown();

    Ok(())
}
```

### Subscribe Message

```rust
use google_cloud_pubsub::client::{Client, ClientConfig};
use google_cloud_googleapis::pubsub::v1::PubsubMessage;
use google_cloud_pubsub::subscription::SubscriptionConfig;
use google_cloud_gax::grpc::Status;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use futures_util::StreamExt;

async fn run(config: ClientConfig) -> Result<(), Status> {

    // Create pubsub client.
    let client = Client::new(config).await.unwrap();

    // Get the topic to subscribe to.
    let topic = client.topic("test-topic");

    // Create subscription
    // If subscription name does not contain a "/", then the project is taken from client above. Otherwise, the
    // name will be treated as a fully qualified resource name
    let config = SubscriptionConfig {
        // Enable message ordering if needed (https://cloud.google.com/pubsub/docs/ordering)
        enable_message_ordering: true,
        ..Default::default()
    };

    // Create subscription
    let subscription = client.subscription("test-subscription");
    if !subscription.exists(None).await? {
        subscription.create(topic.fully_qualified_name(), config, None).await?;
    }

    // Token for cancel.
    let cancel = CancellationToken::new();
    let cancel2 = cancel.clone();
    tokio::spawn(async move {
        // Cancel after 10 seconds.
        tokio::time::sleep(Duration::from_secs(10)).await;
        cancel2.cancel();
    });

    // Receive blocks until the ctx is cancelled or an error occurs.
    // Or simply use the `subscription.subscribe` method.
    subscription.receive(|mut message, cancel| async move {
        // Handle data.
        println!("Got Message: {:?}", message.message.data);

        // Ack or Nack message.
        let _ = message.ack().await;
    }, cancel.clone(), None).await?;

    // Delete subscription if needed.
    subscription.delete(None).await?;

    Ok(())
}
```

### Subscribe Message (Alternative Way)

After canceling, wait until all pulled messages are processed.
```rust
use std::time::Duration;
use futures_util::StreamExt;
use google_cloud_pubsub::client::{Client, ClientConfig};
use google_cloud_googleapis::pubsub::v1::PubsubMessage;
use google_cloud_pubsub::subscription::{SubscribeConfig, SubscriptionConfig};
use google_cloud_gax::grpc::Status;

async fn run(config: ClientConfig) -> Result<(), Status> {
     // Creating Client, Topic and Subscription...
     let client = Client::new(config).await.unwrap();
     let subscription = client.subscription("test-subscription");

     // Read the messages as a stream
     let mut stream = subscription.subscribe(None).await.unwrap();
     let cancellable = stream.cancellable();
     let task = tokio::spawn(async move {
         // None if the stream is cancelled
         while let Some(message) = stream.next().await {
             message.ack().await.unwrap();
         }
     });
     tokio::time::sleep(Duration::from_secs(60)).await;
     cancellable.cancel();
     let _ = task.await;
     Ok(())
}
 ```

Unprocessed messages are nack after cancellation.
```rust
use std::time::Duration;
use google_cloud_pubsub::client::{Client, ClientConfig};
use google_cloud_googleapis::pubsub::v1::PubsubMessage;
use google_cloud_pubsub::subscription::{SubscribeConfig, SubscriptionConfig};
use google_cloud_gax::grpc::Status;

async fn run(config: ClientConfig) -> Result<(), Status> {
     // Creating Client, Topic and Subscription...
     let client = Client::new(config).await.unwrap();
     let subscription = client.subscription("test-subscription");

     // Read the messages as a stream
     let mut stream = subscription.subscribe(None).await.unwrap();
     let cancellable = stream.cancellable();
     let task = tokio::spawn(async move {
         // None if the stream is cancelled
         while let Some(message) = stream.read().await {
             message.ack().await.unwrap();
         }
     });
     tokio::time::sleep(Duration::from_secs(60)).await;
     cancellable.cancel();
     let _ = task.await;
     Ok(())
}
```