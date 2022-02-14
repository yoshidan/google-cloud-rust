use std::sync::Arc;
use google_cloud_googleapis::pubsub::v1::PubsubMessage;
use google_cloud_pubsub::apiv1::publisher_client::PublisherClient;
use google_cloud_pubsub::apiv1::conn_pool::ConnectionManager;
use google_cloud_pubsub::publisher::{Publisher, PublisherConfig};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_publish() -> Result<(), anyhow::Error> {

    let cons = ConnectionManager::new(4, Some("localhost:8681".to_string())).await?;
    let client = PublisherClient::new(cons);

    let mut publisher = Arc::new(Publisher::new("projects/local-project/topics/test-topic1".to_string(), PublisherConfig {
        workers: 5,
        timeout: std::time::Duration::from_secs(1),
        buffer_size: 3
    }, client));

    for _ in 0..10 {
        let p = publisher.clone();
        tokio::spawn(async move {
            let mut result = p.publish(PubsubMessage {
                data: "abc".into(),
                attributes: Default::default(),
                message_id: "".to_string(),
                publish_time: None,
                ordering_key: "".to_string()
            }).await;
            let v = result.get().await;
            println!("{}", v.unwrap());
        });
    }
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    Ok(())
}