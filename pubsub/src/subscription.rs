use std::future::Future;
use std::sync::Arc;
use google_cloud_googleapis::pubsub::v1::DeleteSubscriptionRequest;
use google_cloud_googleapis::Status;
use crate::apiv1::subscriber_client::SubscriberClient;
use crate::publisher::ReservedMessage;
use crate::subscriber::{Config, ReceivedMessage, Subscriber};

pub struct ReceiveConfig {
    pub ordering_worker_count: usize,
    pub worker_count: usize
}

impl Default for ReceiveConfig {
    fn default() -> Self {
        Self {
            ordering_worker_count: 0,
            worker_count: 10,
        }
    }
}

/// Subscription is a reference to a PubSub subscription.
pub struct Subscription {
    name: String,
    subc: SubscriberClient,
    subscriber: Option<Subscriber>
}

impl Subscription {
    pub(crate) fn new(name: String, subc: SubscriberClient) -> Self {
        Self {
            name,
            subc,
            subscriber: None
        }
    }

    pub async fn receive<F>(&mut self, f: impl Fn(ReceivedMessage) -> F + Send + 'static + Sync + Clone, config: Option<ReceiveConfig>)
    where F: Future<Output = ()> + Send + 'static {
        let op = config.unwrap_or_default();
        let mut receivers  = Vec::with_capacity(op.ordering_worker_count + op.worker_count);
        let mut senders = Vec::with_capacity(receivers.len());
        let (sender, receiver) = async_channel::unbounded::<ReceivedMessage>();
        (0..op.worker_count).map(|v| {
            receivers.push(receiver.clone());
            senders.push(sender.clone());
        });
        (0..op.ordering_worker_count).map(|v| {
            let (sender, receiver) = async_channel::unbounded::<ReceivedMessage>();
            receivers.push(receiver);
            senders.push(sender);
        });

        self.subscriber = Some(Subscriber::new(self.name.clone(), self.subc.clone(), senders, Config::default()));
        let mut join_handles = Vec::with_capacity(receivers.len());
        for receiver in receivers {
            let f_clone = f.clone();
            join_handles.push(tokio::spawn(async move {
                while let Ok(message) = receiver.recv().await {
                    f_clone(message).await;
                };
            }));
        }
        // wait
        for j in join_handles {
            j.await;
        }
    }

    pub async fn delete(&mut self) -> Result<(), Status>{
        self.subc.delete_subscription(DeleteSubscriptionRequest {
            subscription: self.name.to_string()
        }, None).await.map(|v| v.into_inner())
    }

    pub fn stop(&mut self) {
        if let Some(s) = &mut self.subscriber {
            s.stop();
        }
    }
}
