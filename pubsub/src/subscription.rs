use std::collections::HashMap;
use std::future::Future;

use std::time::Duration;
use prost_types::FieldMask;
use google_cloud_gax::call_option::BackoffRetrySettings;

use google_cloud_googleapis::pubsub::v1::{DeadLetterPolicy, DeleteSubscriptionRequest, ExpirationPolicy, GetSubscriptionRequest, PushConfig, RetryPolicy, Subscription as InternalSubscription, UpdateSubscriptionRequest};
use google_cloud_googleapis::{Code, Status};
use crate::apiv1::subscriber_client::SubscriberClient;
use crate::cancel::CancellationToken;

use crate::subscriber::{ReceivedMessage, Subscriber, SubscriberConfig};

/// SubscriptionConfigToUpdate describes how to update a subscription.
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
    pub worker_count: usize,
    pub subscriber_config: SubscriberConfig,
}

impl Default for ReceiveConfig {
    fn default() -> Self {
        Self {
            worker_count: 10,
            subscriber_config: SubscriberConfig::default(),
        }
    }
}


/// Subscription is a reference to a PubSub subscription.
pub struct Subscription {
    name: String,
    subc: SubscriberClient,
}

impl Subscription {
    pub(crate) fn new(name: String, subc: SubscriberClient) -> Self {
        Self {
            name,
            subc,
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

    /// delete deletes the subscription.
    pub async fn delete(&mut self, retry_option: Option<BackoffRetrySettings>) -> Result<(), Status>{
        self.subc.delete_subscription(DeleteSubscriptionRequest {
            subscription: self.name.to_string()
        }, retry_option).await.map(|v| v.into_inner())
    }

    /// exists reports whether the subscription exists on the server.
    pub async fn exists(&mut self, retry_option: Option<BackoffRetrySettings>) -> Result<bool, Status>{
        match self.subc.get_subscription(GetSubscriptionRequest{
            subscription: self.name.to_string()
        }, retry_option).await {
            Ok(_) => Ok(true),
            Err(e) => {
                if e.code() == Code::NotFound {
                    Ok(false)
                }else {
                    Err(e)
                }
            }
        }
    }

    /// config fetches the current configuration for the subscription.
    pub async fn config(&mut self, retry_option: Option<BackoffRetrySettings>) -> Result<(String, SubscriptionConfig), Status>{
        self.subc.get_subscription(GetSubscriptionRequest{
            subscription: self.name.to_string()
        }, retry_option).await.map(|v| {
            let inner = v.into_inner();
            (inner.topic.to_string(),inner.into())
        })
    }

    /// update changes an existing subscription according to the fields set in updating.
    /// It returns the new SubscriptionConfig.
    pub async fn update(&mut self, updating: SubscriptionConfigToUpdate, opt: Option<BackoffRetrySettings>) -> Result<(String, SubscriptionConfig), Status>{
        let mut config = self.subc.get_subscription(GetSubscriptionRequest{
            subscription: self.name.to_string()
        }, opt.clone()).await?.into_inner();

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
        }, opt).await.map(|v| {
            let inner = v.into_inner();
            (inner.topic.to_string(),inner.into())
        })
    }

    /// receive calls f with the outstanding messages from the subscription.
    /// It blocks until ctx is done, or the service returns a non-retryable error.
    /// The standard way to terminate a receive is to use CancellationToken.
    pub async fn receive<F>(&mut self, mut cancellation_token: CancellationToken,  f: impl Fn(ReceivedMessage) -> F + Send + 'static + Sync + Clone, config: Option<ReceiveConfig>) -> Result<(), Status>
        where F: Future<Output = ()> + Send + 'static {
        let op = config.unwrap_or_default();
        let mut receivers  = Vec::with_capacity(op.worker_count);
        let mut senders = Vec::with_capacity(receivers.len());

        if self.config(op.subscriber_config.retry_setting.clone()).await?.1.enable_message_ordering {
            (0..op.worker_count).for_each(|_v| {
                let (sender, receiver) = async_channel::unbounded::<ReceivedMessage>();
                receivers.push(receiver);
                senders.push(sender);
            });
        }else {
            let (sender, receiver) = async_channel::unbounded::<ReceivedMessage>();
            (0..op.worker_count).for_each(|_v| {
                receivers.push(receiver.clone());
                senders.push(sender.clone());
            });
        }

        //Orderingが有効な場合、順序付きメッセージは同じStreamに入ってくるためSubscriber毎にqueueが別れていれば問題はない。
        let subscribers : Vec<Subscriber> = senders.clone().into_iter().map(|queue| {
            Subscriber::new(self.name.clone(), self.subc.clone(), queue, Some(op.subscriber_config.clone()))
        }).collect();

        let mut message_receivers= Vec::with_capacity(receivers.len());
        for receiver in receivers {
            let f_clone = f.clone();
            message_receivers.push(tokio::spawn(async move {
                while let Ok(message) = receiver.recv().await {
                    f_clone(message).await;
                };
                println!("closed subscription workers");
            }));
        }
        cancellation_token.done().await;

        for subscriber in subscribers {
            subscriber.stop();
        }

        for sender in senders {
            sender.close();
        }

        // wait for finish
        for mr in message_receivers {
            mr.await;
        }
        Ok(())
    }
}
