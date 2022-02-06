use std::sync::Arc;
use google_cloud_googleapis::pubsub::v1::{PubsubMessage, StreamingPullRequest};
use google_cloud_pubsub::apiv1::publisher_client::PublisherClient;
use google_cloud_pubsub::apiv1::conn_pool::ConnectionManager;
use google_cloud_pubsub::publisher::{Publisher, SchedulerConfig};
use serial_test::serial;
use tonic::IntoStreamingRequest;
use google_cloud_pubsub::apiv1::subscriber_client::SubscriberClient;
use google_cloud_pubsub::subscriber::Subscriber;

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_subscribe() -> Result<(), anyhow::Error> {

    let cons = ConnectionManager::new(4, Some("localhost:8681".to_string())).await?;
    let mut subc = SubscriberClient::new(cons.conn());

    let cons = ConnectionManager::new(4, Some("localhost:8681".to_string())).await?;
    let client = PublisherClient::new(cons.conn());

    let mut publisher = Arc::new(Publisher::new("projects/local-project/topics/test-topic1".to_string(), SchedulerConfig {
        workers: 5,
        timeout: std::time::Duration::from_secs(1)
    }, client));
    publisher.publish(PubsubMessage {
        data: "abc".into(),
        attributes: Default::default(),
        message_id: "".to_string(),
        publish_time: None,
        ordering_key: "".to_string()
    }).await.get().await;

    let (sender, receiver) = async_channel::unbounded();
    let mut subscriber = Arc::new(Subscriber::new("projects/local-project/subscriptions/test-subscription1".to_string(), subc , sender));
    let message = receiver.recv().await;
    println!("result = {}", message.is_ok());
    println!("message = {}", std::str::from_utf8(&message.unwrap().data).unwrap());


    //let mut subc = Arc::new(Subscriber::new("projects/local-project/subscriptions/test-subscription2".to_string(), client));
    //waiter.await;
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    Ok(())
}