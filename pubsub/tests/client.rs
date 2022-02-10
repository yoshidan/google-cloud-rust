use std::sync::Arc;
use google_cloud_googleapis::pubsub::v1::PubsubMessage;
use google_cloud_pubsub::apiv1::publisher_client::PublisherClient;
use google_cloud_pubsub::apiv1::conn_pool::ConnectionManager;
use google_cloud_pubsub::publisher::{Publisher, SchedulerConfig};
use serial_test::serial;
use uuid::Uuid;
use google_cloud_pubsub::client::Client;


fn create_message(data: &[u8], ordering_key: &str) -> PubsubMessage {
    PubsubMessage {
        data: data.to_vec(),
        attributes: Default::default(),
        message_id: "".to_string(),
        publish_time: None,
        ordering_key: ordering_key.to_string()
    }
}

#[tokio::test]
#[serial]
async fn test_scenario() -> Result<(), anyhow::Error> {
    std::env::set_var("PUBSUB_EMULATOR_HOST","localhost:8681".to_string());
    let client = Client::new("local-project", None).await.unwrap();

    // create
    let uuid = Uuid::new_v4().to_hyphenated().to_string();
    let mut topic = client.create_topic(&uuid, None).await.unwrap();
    let mut subscription = client.create_subscription(&uuid, topic.string()).await.unwrap();

    //subscribe
    tokio::spawn(|| async move {
        subscription.receive(|mut v| async move {
            v.ack().await;
            println!("id={} data={}", v.message.message_id, std::str::from_utf8(&v.message.data).unwrap());
        }).await;
    });

    //publish
    let message = create_message("abc".as_bytes(), "");
    let message_id = topic.publish(message).await.get().await.unwrap();
    println!("sent {}", message_id);
    Ok(())
}