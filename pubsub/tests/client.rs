use std::sync::Arc;
use std::thread;
use google_cloud_googleapis::pubsub::v1::PubsubMessage;
use google_cloud_pubsub::apiv1::publisher_client::PublisherClient;
use google_cloud_pubsub::apiv1::conn_pool::ConnectionManager;
use google_cloud_pubsub::publisher::{Publisher, PublisherConfig};
use serial_test::serial;
use uuid::Uuid;
use google_cloud_pubsub::client::{Client};
use google_cloud_pubsub::subscriber::ReceivedMessage;
use google_cloud_pubsub::subscription::{ReceiveConfig, SubscriptionConfig};


fn create_message(data: &[u8], ordering_key: &str) -> PubsubMessage {
    PubsubMessage {
        data: data.to_vec(),
        attributes: Default::default(),
        message_id: "".to_string(),
        publish_time: None,
        ordering_key: ordering_key.to_string()
    }
}

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_scenario() -> Result<(), anyhow::Error> {
    std::env::set_var("PUBSUB_EMULATOR_HOST","localhost:8681".to_string());
    let client = Client::new("local-project", None).await.unwrap();

    // create
    let uuid = Uuid::new_v4().to_hyphenated().to_string();
    let topic_name = &format!("t{}", &uuid);
    let subscription_name = &format!("s{}", &uuid);
    let mut topic = client.create_topic(topic_name, None).await.unwrap();
    let mut config = SubscriptionConfig::default();
    config.enable_message_ordering = true;
    let mut subscription = client.create_subscription(subscription_name , &topic, config).await.unwrap();

    //subscribe
    let handle = tokio::spawn(async move {
        subscription.receive(|mut v| async move {
            v.ack().await;
            println!("tid={:?} id={} data={}", thread::current().id(), v.message.message_id, std::str::from_utf8(&v.message.data).unwrap());
        }, Some(ReceiveConfig {
            ordering_worker_count: 2,
            worker_count: 2
        })).await
    });

    //publish
    for v in 0..100 {
        let message = create_message(format!("abc_{}",v).as_bytes(), "orderkey");
        let message_id = topic.publish(message).await.get().await.unwrap();
        println!("sent {}", message_id);
    }
    handle.await;

    Ok(())
}