use std::thread;
use std::time::Duration;
use google_cloud_googleapis::pubsub::v1::PubsubMessage;
use serial_test::serial;
use uuid::Uuid;
use google_cloud_pubsub::cancel::CancellationToken;
use google_cloud_pubsub::client::{Client};
use google_cloud_pubsub::subscriber::SubscriberConfig;

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
    let topic = client.create_topic(topic_name, None, None).await.unwrap();
    let mut config = SubscriptionConfig::default();
    config.enable_message_ordering = true;
    let mut subscription = client.create_subscription(subscription_name , &topic, config, None).await.unwrap();

    let (token,cancel) = CancellationToken::new();
    //subscribe
    let mut config = ReceiveConfig {
        worker_count: 2,
        subscriber_config: SubscriberConfig::default(),
    };
    config.subscriber_config.ping_interval = Duration::from_secs(1);
    let handle = tokio::spawn(async move {
        subscription.receive(token, |mut v| async move {
            let _ = v.ack().await;
            println!("tid={:?} id={} data={}", thread::current().id(), v.message.message_id, std::str::from_utf8(&v.message.data).unwrap());
        }, Some(config)).await;
    });

    //publish
    let mut awaiters = Vec::with_capacity(100);
    for v in 0..100 {
        let message = create_message(format!("abc_{}",v).as_bytes(), "orderkey");
        awaiters.push(topic.publish(message).await);
    }
    for mut v in awaiters {
        println!("sent {}", v.get().await.unwrap());
    }

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    drop(cancel);
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    let _ = handle.await;
    Ok(())
}