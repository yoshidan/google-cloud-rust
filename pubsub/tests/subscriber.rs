use std::ops::Deref;
use std::ptr::hash;
use std::sync::Arc;
use parking_lot::Mutex;
use google_cloud_googleapis::pubsub::v1::{PubsubMessage, StreamingPullRequest};
use google_cloud_pubsub::apiv1::publisher_client::PublisherClient;
use google_cloud_pubsub::apiv1::conn_pool::ConnectionManager;
use google_cloud_pubsub::publisher::{Publisher, SchedulerConfig};
use serial_test::serial;
use tokio::time::timeout;
use tonic::IntoStreamingRequest;
use google_cloud_pubsub::apiv1::subscriber_client::SubscriberClient;
use google_cloud_pubsub::subscriber::{Config, Subscriber};

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_subscribe() -> Result<(), anyhow::Error> {

    let cons = ConnectionManager::new(4, Some("localhost:8681".to_string())).await?;
    let subc = SubscriberClient::new(cons.conn());
    let pubc = PublisherClient::new(cons.conn());

    let mut publisher = Publisher::new("projects/local-project/topics/test-topic1".to_string(), SchedulerConfig {
        workers: 5,
        timeout: std::time::Duration::from_secs(1)
    }, pubc);
    publisher.publish(PubsubMessage {
        data: "test_message".into(),
        attributes: Default::default(),
        message_id: "".to_string(),
        publish_time: None,
        ordering_key: "".to_string()
    }).await.get().await;

    let (sender, receiver) = async_channel::unbounded();
    let mut subscriber = Subscriber::new("projects/local-project/subscriptions/test-subscription1".to_string(), subc , sender, Config::default());

    let (tx, rx) = tokio::sync::oneshot::channel::<bool>();
    tokio::spawn(async move {
        match receiver.recv().await {
            Ok(mut message) => {
                println!("message = {}", message.message.message_id);
                let data = &message.message.data;
                let string = std::str::from_utf8(data).unwrap();
                if string == "test_message" {
                    tx.send(true);
                }
                message.ack().await;
            },
            Err(e) => {
                println!("closed {:?}", e);
            }
        };
    });
    let mut result = false;
    if let Ok(_) = timeout(std::time::Duration::from_secs(3),rx).await {
        println!("result ok");
        result = true;
    };
    assert_eq!(result, true) ;
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    subscriber.stop();
    publisher.stop();
    println!("stopped");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    Ok(())
}