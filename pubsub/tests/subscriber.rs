use std::ops::Deref;
use std::ptr::hash;
use std::sync::Arc;
use parking_lot::Mutex;
use google_cloud_googleapis::pubsub::v1::{PubsubMessage, StreamingPullRequest, Subscription};
use google_cloud_pubsub::apiv1::publisher_client::PublisherClient;
use google_cloud_pubsub::apiv1::conn_pool::ConnectionManager;
use google_cloud_pubsub::publisher::{Publisher, SchedulerConfig};
use serial_test::serial;
use tokio::time::timeout;
use tonic::IntoStreamingRequest;
use google_cloud_pubsub::apiv1::subscriber_client::SubscriberClient;
use google_cloud_pubsub::subscriber::{Config, ReceivedMessage, Subscriber};
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::SeqCst;
use google_cloud_googleapis::longrunning::CancelOperationRequest;
use google_cloud_grpc::conn::Channel;
use uuid::Uuid;

fn create_default_subscription_request(topic: String) -> Subscription {
    let uuid = Uuid::new_v4().to_hyphenated().to_string();
    return Subscription {
        name: format!("projects/local-project/subscriptions/test-{}",uuid),
        topic: topic.to_string(),
        push_config: None,
        ack_deadline_seconds: 0,
        retain_acked_messages: false,
        message_retention_duration: None,
        labels: Default::default(),
        enable_message_ordering: false,
        expiration_policy: None,
        filter: "".to_string(),
        dead_letter_policy: None,
        retry_policy: None,
        detached: false,
        topic_message_retention_duration: None
    };
}

async fn publish(ch: Channel) -> Publisher {
    let pubc = PublisherClient::new(ch);
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
    return publisher;
}

fn subscribe(v: Arc<AtomicU32>, name: String, receiver: async_channel::Receiver<ReceivedMessage>){
    tokio::spawn(async move {
        while let Ok(mut message) = receiver.recv().await {
            println!("message = {} from={}", message.message.message_id, name.to_string());
            let data = &message.message.data;
            let string = std::str::from_utf8(data).unwrap();
            if string == "test_message" {
                v.fetch_add(1, SeqCst);
            }
            match message.ack().await {
                Ok(_) => {},
                Err(e) => {
                    println!("error {}", e);
                }
            }
        };
    });
}

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_multi_subscriber_single_subscription() -> Result<(), anyhow::Error> {

    let cons = ConnectionManager::new(4, Some("localhost:8681".to_string())).await?;
    let mut subc = SubscriberClient::new(cons.conn());
    let v = Arc::new(AtomicU32::new(0));
    let subscription = subc.create_subscription(create_default_subscription_request( "projects/local-project/topics/test-topic1".to_string()), None).await.unwrap().into_inner().name;
    let mut subscribers = vec![];
    for _ in 0..3 {
        let mut subc = SubscriberClient::new(cons.conn());
        let (sender, receiver) = async_channel::unbounded();
        subscribers.push(Subscriber::new(subscription.clone(), subc, sender, Config::default()));
        subscribe(v.clone(), subscription.clone(), receiver);
    }

    let mut publisher = publish(cons.conn()).await;

    for mut subscriber in subscribers {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        subscriber.stop();
        println!("stopped");
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }

    assert_eq!(v.load(SeqCst),1);
    publisher.stop();
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn test_multi_subscriber_multi_subscription() -> Result<(), anyhow::Error> {

    let cons = ConnectionManager::new(4, Some("localhost:8681".to_string())).await?;

    let mut subscribers = vec![];
    for _ in 0..3 {
        let mut subc = SubscriberClient::new(cons.conn());
        let subscription = subc.create_subscription(create_default_subscription_request("projects/local-project/topics/test-topic1".to_string()), None).await.unwrap().into_inner().name;
        let (sender, receiver) = async_channel::unbounded();
        let v = Arc::new(AtomicU32::new(0));
        subscribers.push((v.clone(), Subscriber::new(subscription.clone(), subc, sender, Config::default())));
        subscribe(v.clone(), subscription, receiver);
    }

    let mut publisher = publish(cons.conn()).await;

    for (v, mut subscriber) in subscribers {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        subscriber.stop();
        println!("stopped");
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        assert_eq!(v.load(SeqCst),1);
    }
    publisher.stop();
    Ok(())
}