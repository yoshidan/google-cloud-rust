use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use prost_types::FieldMask;
use google_cloud_googleapis::pubsub::v1::{DeadLetterPolicy, DeleteSubscriptionRequest, ExpirationPolicy, GetSubscriptionRequest, PushConfig, RetryPolicy, Subscription as InternalSubscription, UpdateSubscriptionRequest};
use google_cloud_googleapis::Status;
use crate::apiv1::subscriber_client::SubscriberClient;
use crate::publisher::ReservedMessage;
use crate::subscriber::{Config, ReceivedMessage, Subscriber};
use crate::topic::Topic;

pub struct SubscriptionConfig {
    pub push_config: Option<PushConfig>,
    pub ack_deadline_seconds: i32,
    pub retain_acked_messages: bool,
    pub message_retention_duration: Option<Duration>,
    pub labels: HashMap<String, String>,
    pub enable_message_ordering: bool,
    pub expiration_policy: Option<ExpirationPolicy>,
    pub filter: String,
    pub dead_letter_policy: Option<DeadLetterPolicy>,
    pub retry_policy: Option<RetryPolicy>,
    pub detached: bool,
    pub topic_message_retention_duration: Option<Duration>,
}

pub struct SubscriptionConfigToUpdate {
    pub push_config: Option<PushConfig>,
    pub ack_deadline_seconds: Option<i32>,
    pub retain_acked_messages: Option<bool>,
    pub message_retention_duration: Option<Duration>,
    pub labels: Option<HashMap<String, String>>,
    pub expiration_policy: Option<ExpirationPolicy>,
    pub dead_letter_policy: Option<DeadLetterPolicy>,
    pub retry_policy: Option<RetryPolicy>,
}


impl Default for SubscriptionConfig {
    fn default() -> Self {
        Self {
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
        }
    }
}

impl Into<SubscriptionConfig> for InternalSubscription {
    fn into(self) -> SubscriptionConfig {
        SubscriptionConfig {
            push_config: self.push_config,
            ack_deadline_seconds: self.ack_deadline_seconds,
            retain_acked_messages: self.retain_acked_messages,
            message_retention_duration: self.message_retention_duration.map(|v| std::time::Duration::new(v.seconds as u64, v.nanos as u32)),
            labels: self.labels,
            enable_message_ordering: self.enable_message_ordering,
            expiration_policy: self.expiration_policy,
            filter: self.filter,
            dead_letter_policy: self.dead_letter_policy,
            retry_policy: self.retry_policy,
            detached: self.detached,
            topic_message_retention_duration: self.topic_message_retention_duration.map(|v| std::time::Duration::new(v.seconds as u64, v.nanos as u32)),
        }
    }
}

pub struct ReceiveConfig {
    pub worker_count: usize
}

impl Default for ReceiveConfig {
    fn default() -> Self {
        Self {
            worker_count: 10,
        }
    }
}

/// Subscription is a reference to a PubSub subscription.
pub struct Subscription {
    name: String,
    subc: SubscriberClient,
    subscriber: Option<Subscriber>,
}

impl Subscription {
    pub(crate) fn new(name: String, subc: SubscriberClient) -> Self {
        Self {
            name,
            subc,
            subscriber: None,
        }
    }

    /// id returns the unique identifier of the subscription within its project.
    pub fn id(&self) -> Option<String> {
        self.name.rfind('/').map(|i| self.name[(i + 1)..].to_string())
    }

    /// string returns the globally unique printable name of the subscription.
    pub fn string(&self) -> &str {
        self.name.as_str()
    }

    pub async fn receive<F>(&mut self, cancel: tokio::sync::oneshot::Receiver<bool>,  f: impl Fn(ReceivedMessage) -> F + Send + 'static + Sync + Clone, config: Option<ReceiveConfig>) -> Result<(), Status>
    where F: Future<Output = ()> + Send + 'static {
        let op = config.unwrap_or_default();
        let mut receivers  = Vec::with_capacity(op.worker_count);
        let mut senders = Vec::with_capacity(receivers.len());

        if self.config().await?.1.enable_message_ordering {
            (0..op.worker_count).for_each(|v| {
                let (sender, receiver) = async_channel::unbounded::<ReceivedMessage>();
                receivers.push(receiver);
                senders.push(sender);
            });
        }else {
            let (sender, receiver) = async_channel::unbounded::<ReceivedMessage>();
            (0..op.worker_count).for_each(|v| {
                receivers.push(receiver.clone());
                senders.push(sender.clone());
            });
        }

        let mut subscriber = Subscriber::new(self.name.clone(), self.subc.clone(), senders.clone(), Config::default());
        let mut join_handles = Vec::with_capacity(receivers.len());
        for receiver in receivers {
            let f_clone = f.clone();
            join_handles.push(tokio::spawn(async move {
                while let Ok(message) = receiver.recv().await {
                    f_clone(message).await;
                };
                println!("closed subscription workers");
            }));
        }
        cancel.await;
        println!("cancelled");
        subscriber.stop();
        for s in senders {
            s.close();
        }

        // wait
        for j in join_handles {
            j.await;
        }
        Ok(())
    }

    pub async fn delete(&mut self) -> Result<(), Status>{
        self.subc.delete_subscription(DeleteSubscriptionRequest {
            subscription: self.name.to_string()
        }, None).await.map(|v| v.into_inner())
    }

    pub async fn config(&mut self) -> Result<(String, SubscriptionConfig), Status>{
        self.subc.get_subscription(GetSubscriptionRequest{
            subscription: self.name.to_string()
        }, None).await.map(|v| {
            let inner = v.into_inner();
            (inner.topic.to_string(),inner.into())
        })
    }

    pub async fn update(&mut self, updating: SubscriptionConfigToUpdate) -> Result<(String, SubscriptionConfig), Status>{
        let mut config = self.subc.get_subscription(GetSubscriptionRequest{
            subscription: self.name.to_string()
        }, None).await?.into_inner();

        let mut paths = vec![];
        if updating.push_config.is_some() {
            config.push_config = updating.push_config;
            paths.push("push_config".to_string());
        }
        if let Some(v) = updating.ack_deadline_seconds{
            config.ack_deadline_seconds = v;
            paths.push("ack_deadline_seconds".to_string());
        }
        if let Some(v) = updating.retain_acked_messages {
            config.retain_acked_messages = v;
            paths.push("retain_acked_messages".to_string());
        }
        if updating.message_retention_duration.is_some() {
            let v = updating.message_retention_duration.map(|v| prost_types::Duration::from(v));
            config.message_retention_duration = v;
            paths.push("message_retention_duration".to_string());
        }
        if updating.expiration_policy.is_some() {
            config.expiration_policy = updating.expiration_policy;
            paths.push("expiration_policy".to_string());
        }
        if let Some(v) = updating.labels {
            config.labels = v;
            paths.push("labels".to_string());
        }
        if updating.retry_policy.is_some() {
            config.retry_policy = updating.retry_policy;
            paths.push("retry_policy".to_string());
        }

        self.subc.update_subscription(UpdateSubscriptionRequest{
            subscription: Some(config.into()),
            update_mask: Some(FieldMask {
                paths
            })
        }, None).await.map(|v| {
            let inner = v.into_inner();
            (inner.topic.to_string(),inner.into())
        })
    }

    pub fn stop(&mut self) {
        if let Some(s) = &mut self.subscriber {
            s.stop();
        }
    }
}

impl Drop for Subscription {

    fn drop(&mut self) {
        self.stop();
    }

}