use std::future::Future;
use std::sync::Arc;
use google_cloud_googleapis::pubsub::v1::DeleteSubscriptionRequest;
use google_cloud_googleapis::Status;
use crate::apiv1::subscriber_client::SubscriberClient;
use crate::publisher::ReservedMessage;
use crate::subscriber::{Config, ReceivedMessage, Subscriber};

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

    pub async fn receive<F>(&mut self, f: impl Fn(ReceivedMessage) -> F)
    where F: Future<Output = ()> {
        let (sender, receiver) = async_channel::unbounded();
        self.subscriber = Some(Subscriber::new(self.name.clone(), self.subc.clone(), sender, Config::default()));
        while let Ok(message) = receiver.recv().await {
            f(message).await;
        };
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
