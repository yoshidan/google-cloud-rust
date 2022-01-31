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
    let mut client = SubscriberClient::new(cons.conn());

    let pubc = PublisherClient::new(cons.conn());

    let mut publisher = Arc::new(Publisher::new("projects/local-project/topics/test-topic1".to_string(), SchedulerConfig {
        workers: 5,
        timeout: std::time::Duration::from_secs(1)
    }, pubc));

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

    let request = StreamingPullRequest {
        subscription: "projects/local-project/subscriptions/test-subscription1".to_string(),
        ack_ids: vec![],
        modify_deadline_seconds: vec![],
        modify_deadline_ack_ids: vec![],
        stream_ack_deadline_seconds: 30,
        client_id: "".to_string(),
        max_outstanding_messages: 1000,
        max_outstanding_bytes: 1000 * 1000
    };

    let (sender, receiver) = tokio::sync::watch::channel(0);
    let result = client.streaming_pull(request.clone(),receiver, None).await.unwrap();
    let mut response = result.into_inner();
    println!("start response");

    //pinger
    /*tokio::spawn(async move {
        let mut d = 0;
        loop {
            d += 1;
            let v = sender.send(d).is_ok();
            println!("ping {} {}", d, v );
            tokio::time::sleep(std::time::Duration::from_secs(4)).await;
        }
    });
     */
    let waiter = tokio::spawn(async move {
        loop {

            while let Some(message) = response.message().await.unwrap() {
                println!("message = {}", message.received_messages.len());
            }

        }
    });

    //let mut subc = Arc::new(Subscriber::new("projects/local-project/subscriptions/test-subscription2".to_string(), client));
    waiter.await;
    Ok(())
}