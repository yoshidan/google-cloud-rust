use std::time::Duration;

use tokio::select;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use google_cloud_googleapis::pubsub::v1::{AcknowledgeRequest, ModifyAckDeadlineRequest, PubsubMessage};
use google_cloud_googleapis::{Code, Status};
use crate::apiv1::RetrySetting;

use crate::apiv1::subscriber_client::{create_default_streaming_pull_request, SubscriberClient};

pub struct ReceivedMessage {
   pub message: PubsubMessage ,
   ack_id: String,
   subscription: String,
   subscriber_client: SubscriberClient
}

impl ReceivedMessage {
    pub async fn ack(&self) -> Result<(), Status> {
       self.subscriber_client.acknowledge(CancellationToken::new(), AcknowledgeRequest {
           subscription: self.subscription.to_string(),
           ack_ids: vec![self.ack_id.to_string()]
       }, None).await.map(|e| e.into_inner())
    }

    pub async fn nack(&self) -> Result<(), Status> {
        self.subscriber_client.modify_ack_deadline(CancellationToken::new(), ModifyAckDeadlineRequest {
            subscription: self.subscription.to_string(),
            ack_deadline_seconds: 0,
            ack_ids: vec![self.ack_id.to_string()]
        }, None).await.map(|e| e.into_inner())
    }
}

#[derive(Clone)]
pub struct SubscriberConfig {
    pub ping_interval: Duration,
    pub retry_setting: Option<RetrySetting>
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
    pinger: Option<JoinHandle<()>>,
    inner: Option<JoinHandle<()>>,
}

impl Subscriber {

    pub fn start(ctx: CancellationToken, subscription: String, client: SubscriberClient, queue: async_channel::Sender<ReceivedMessage>, opt: Option<SubscriberConfig>) -> Self {
        let config = opt.unwrap_or_default();

        let cancel_receiver= ctx.clone();
        let (ping_sender,ping_receiver) = async_channel::unbounded();

        // ping request
        let subscription_clone =  subscription.to_string();

        let pinger = tokio::spawn(async move {
            loop {
                select! {
                    _ = cancel_receiver.cancelled() => {
                        ping_sender.close();
                        break;
                    }
                    _ = sleep(config.ping_interval) => {
                        ping_sender.send(true).await;
                    }
                }
            }
            log::trace!("stop pinger : {}", subscription_clone);
        });

        let cancel_receiver= ctx.clone();
        let inner= tokio::spawn(async move {
            log::trace!("start subscriber: {}", subscription);
            let request = create_default_streaming_pull_request(subscription.to_string());
            let response = client.streaming_pull(cancel_receiver.clone(), request, ping_receiver, config.retry_setting).await;

            let mut stream = match response {
                Ok(r) => r.into_inner(),
                Err(e) => {
                    if e.code() == Code::Cancelled {
                        log::trace!("stop subscriber : {}", subscription);
                    }else {
                        log::error!("subscriber error {:?} : {}", e, subscription);
                    }
                    return;
                }
            };
            log::trace!("start streaming: {}", subscription);
            loop {
                select! {
                    _ = cancel_receiver.cancelled() => {
                        queue.close();
                        break;
                    }
                    maybe = stream.message() => {
                        let message = match maybe{
                           Err(_e) => break,
                           Ok(message) => message
                        };
                        let message = match message {
                            Some(m) => m,
                            None => break
                        };
                        for m in message.received_messages {
                            if let Some(mes) = m.message {
                                log::debug!("message received: {}", mes.message_id);
                                queue.send(ReceivedMessage {
                                    message: mes,
                                    ack_id: m.ack_id,
                                    subscription: subscription.to_string(),
                                    subscriber_client: client.clone()
                                }).await;
                            }
                        }
                    }
                }
            }
            // streaming request is closed when the ping_sender closed.
            log::trace!("stop subscriber in streaming: {}", subscription);
        });
        return Self{
            pinger: Some(pinger),
            inner: Some(inner)
        }
    }

    pub async fn done(&mut self) {
        if let Some(v) = self.pinger.take() {
            v.await;
        }
        if let Some(v) = self.inner.take() {
            v.await;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use google_cloud_googleapis::pubsub::v1::{PublishRequest, PubsubMessage, Subscription};
    use crate::apiv1::publisher_client::PublisherClient;
    use crate::apiv1::conn_pool::ConnectionManager;
    
    use serial_test::serial;

    use crate::apiv1::subscriber_client::SubscriberClient;
    use crate::subscriber::{ReceivedMessage, Subscriber};
    use std::sync::atomic::AtomicU32;
    use std::sync::atomic::Ordering::SeqCst;
    use tokio_util::sync::CancellationToken;

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

    async fn publish() {
        let pubc = PublisherClient::new(ConnectionManager::new(4, Some("localhost:8681".to_string())).await.unwrap());
        let msg =  PubsubMessage {
            data: "test_message".into(),
            attributes: Default::default(),
            message_id: "".to_string(),
            publish_time: None,
            ordering_key: "".to_string()
        };
        pubc.publish(CancellationToken::new(), PublishRequest {
            topic: "projects/local-project/topics/test-topic1".to_string(),
            messages: vec![msg]
        }, None).await;
    }

    fn subscribe(v: Arc<AtomicU32>, name: String, receiver: async_channel::Receiver<ReceivedMessage>){
        tokio::spawn(async move {
            while let Ok(message) = receiver.recv().await {
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
        let ctx = CancellationToken::new();
        let subscription = subc.create_subscription(ctx.clone(), create_default_subscription_request( "projects/local-project/topics/test-topic1".to_string()), None).await.unwrap().into_inner().name;
        let mut subscribers = vec![];
        for _ in 0..3 {
            let (sender, receiver) = async_channel::unbounded();
            subscribers.push(Subscriber::start(ctx.clone(), subscription.clone(), subc.clone(), sender, None));
            subscribe(v.clone(), subscription.clone(), receiver);
        }

        publish().await;

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        ctx.cancel();
        for mut s in subscribers {
            s.done().await;
        }
        assert_eq!(v.load(SeqCst),1);
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
            let ctx = CancellationToken::new();
            let subscription = subc.clone().create_subscription(ctx.clone(), create_default_subscription_request("projects/local-project/topics/test-topic1".to_string()), None).await.unwrap().into_inner().name;
            let (sender, receiver) = async_channel::unbounded();
            let v = Arc::new(AtomicU32::new(0));
            subscribers.push((ctx.clone(), v.clone(), Subscriber::start(ctx, subscription.clone(), subc.clone(), sender, None)));
            subscribe(v.clone(), subscription, receiver);
        }

        publish().await;

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        for (ctx, v, mut s) in subscribers {
            ctx.cancel();
            s.done().await;
            assert_eq!(v.load(SeqCst),1);
        }
        Ok(())
    }
}