use std::time::Duration;
use async_channel::Sender;
use google_cloud_gax::call_option::BackoffRetrySettings;
use google_cloud_googleapis::pubsub::v1::{AcknowledgeRequest, ModifyAckDeadlineRequest, PubsubMessage};
use google_cloud_googleapis::{Status};

use crate::apiv1::subscriber_client::{create_default_streaming_pull_request, SubscriberClient};

pub struct ReceivedMessage {
   pub message: PubsubMessage ,
   ack_id: String,
   subscription: String,
   subscriber_client: SubscriberClient
}

impl ReceivedMessage {
    pub async fn ack(&mut self) -> Result<(), Status> {
       self.subscriber_client.acknowledge(AcknowledgeRequest {
           subscription: self.subscription.to_string(),
           ack_ids: vec![self.ack_id.to_string()]
       }, None).await.map(|e| e.into_inner())
    }

    pub async fn nack(&mut self) -> Result<(), Status> {
        self.subscriber_client.modify_ack_deadline(ModifyAckDeadlineRequest {
            subscription: self.subscription.to_string(),
            ack_deadline_seconds: 0,
            ack_ids: vec![self.ack_id.to_string()]
        }, None).await.map(|e| e.into_inner())
    }
}

#[derive(Clone)]
pub struct SubscriberConfig {
    pub ping_interval: Duration,
    pub retry_setting: Option<BackoffRetrySettings>
}

impl Default for SubscriberConfig {
    fn default() -> Self {
        Self {
            ping_interval: std::time::Duration::from_secs(10),
            retry_setting: None,
        }
    }
}

pub(crate) struct Subscriber {
    pub ping_sender: Sender<bool>,
}

impl Subscriber {

    pub fn new(subscription: String, mut client: SubscriberClient, queue: async_channel::Sender<ReceivedMessage>, opt: Option<SubscriberConfig>) -> Self {
        let config = opt.unwrap_or_default();

        let (ping_sender,ping_receiver) = async_channel::unbounded();

        // ping request
        let ping_sender_clone = ping_sender.clone();
        let subscription_clone =  subscription.to_string();
        tokio::spawn(async move {
            while !ping_sender_clone.is_closed() {
                ping_sender_clone.send(true).await;
                tokio::time::sleep(config.ping_interval).await;
            }
            log::trace!("stop pinger : {}", subscription_clone);
        });

        tokio::spawn(async move {
            log::trace!("start subscriber: {}", subscription);
            let request = create_default_streaming_pull_request(subscription.to_string());
            let response = client.streaming_pull(request, ping_receiver, config.retry_setting).await;

            match response {
                Ok(r) => {
                    let mut stream = r.into_inner();
                    while let Ok(Some(message)) = stream.message().await {
                        for m in message.received_messages {
                            if let Some(mes) = m.message {
                                log::debug!("message received: {}", mes.message_id);
                                let v = queue.send(ReceivedMessage {
                                    message: mes,
                                    ack_id: m.ack_id,
                                    subscription: subscription.to_string(),
                                    subscriber_client: client.clone()
                                }).await;
                                if v.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    // streaming request is closed when the ping_sender closed.
                    log::trace!("stop subscriber : {}", subscription);
                },
                Err(e)=> {
                    log::error!("subscriber error {:?} : {}", e, subscription);
                }
            };
            ()
        });
        return Self{
            ping_sender,
        }
    }

    pub fn stop(& self) {
        self.ping_sender.close();
    }
}

impl Drop for Subscriber {
    fn drop(&mut self) {
        self.stop();
    }
}


#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use google_cloud_googleapis::pubsub::v1::{PubsubMessage, Subscription};
    use crate::apiv1::publisher_client::PublisherClient;
    use crate::apiv1::conn_pool::ConnectionManager;
    use crate::publisher::{Publisher, PublisherConfig};
    use serial_test::serial;

    use crate::apiv1::subscriber_client::SubscriberClient;
    use crate::subscriber::{ReceivedMessage, Subscriber};
    use std::sync::atomic::AtomicU32;
    use std::sync::atomic::Ordering::SeqCst;

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

    async fn publish() -> Publisher {
        let pubc = PublisherClient::new(ConnectionManager::new(4, Some("localhost:8681".to_string())).await.unwrap());
        let publisher = Publisher::new("projects/local-project/topics/test-topic1".to_string(), pubc, None);
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
                log::info!("message = {} from={}", message.message.message_id, name.to_string());
                let data = &message.message.data;
                let string = std::str::from_utf8(data).unwrap();
                if string == "test_message" {
                    v.fetch_add(1, SeqCst);
                }
                match message.ack().await {
                    Ok(_) => {},
                    Err(e) => {
                        log::error!("error {}", e);
                    }
                }
            };
        });
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_multi_subscriber_single_subscription() -> Result<(), anyhow::Error> {
        std::env::set_var("RUST_LOG","google_cloud_pubsub=trace".to_string());
        env_logger::init();
        let subc = SubscriberClient::new(ConnectionManager::new(4, Some("localhost:8681".to_string())).await?);
        let v = Arc::new(AtomicU32::new(0));
        let subscription = subc.create_subscription(create_default_subscription_request( "projects/local-project/topics/test-topic1".to_string()), None).await.unwrap().into_inner().name;
        let mut subscribers = vec![];
        for _ in 0..3 {
            let (sender, receiver) = async_channel::unbounded();
            subscribers.push(Subscriber::new(subscription.clone(), subc.clone(), sender, None));
            subscribe(v.clone(), subscription.clone(), receiver);
        }

        let mut publisher = publish().await;

        for subscriber in subscribers {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            subscriber.stop();
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }

        assert_eq!(v.load(SeqCst),1);
        publisher.stop();
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_multi_subscriber_multi_subscription() -> Result<(), anyhow::Error> {

        std::env::set_var("RUST_LOG","google_cloud_pubsub=trace".to_string());
        env_logger::init();
        let cons = ConnectionManager::new(4, Some("localhost:8681".to_string())).await?;
        let subc = SubscriberClient::new(cons);

        let mut subscribers = vec![];
        for _ in 0..3 {
            let subscription = subc.clone().create_subscription(create_default_subscription_request("projects/local-project/topics/test-topic1".to_string()), None).await.unwrap().into_inner().name;
            let (sender, receiver) = async_channel::unbounded();
            let v = Arc::new(AtomicU32::new(0));
            subscribers.push((v.clone(), Subscriber::new(subscription.clone(), subc.clone(), sender, None)));
            subscribe(v.clone(), subscription, receiver);
        }

        let mut publisher = publish().await;

        for (v, subscriber) in subscribers {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            subscriber.stop();
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            assert_eq!(v.load(SeqCst),1);
        }
        publisher.stop();
        Ok(())
    }
}