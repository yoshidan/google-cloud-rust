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
    pub fn id(&self) -> String {
        self.name.rfind('/').map_or("".to_string(),|i| self.name[(i + 1)..].to_string())
    }

    /// string returns the globally unique printable name of the subscription.
    pub fn string(&self) -> &str {
        self.name.as_str()
    }

    /// delete deletes the subscription.
    pub async fn delete(&self, retry_option: Option<BackoffRetrySettings>) -> Result<(), Status>{
        self.subc.delete_subscription(DeleteSubscriptionRequest {
            subscription: self.name.to_string()
        }, retry_option).await.map(|v| v.into_inner())
    }

    /// exists reports whether the subscription exists on the server.
    pub async fn exists(&self, retry_option: Option<BackoffRetrySettings>) -> Result<bool, Status>{
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
    pub async fn config(&self, retry_option: Option<BackoffRetrySettings>) -> Result<(String, SubscriptionConfig), Status>{
        self.subc.get_subscription(GetSubscriptionRequest{
            subscription: self.name.to_string()
        }, retry_option).await.map(|v| {
            let inner = v.into_inner();
            (inner.topic.to_string(),inner.into())
        })
    }

    /// update changes an existing subscription according to the fields set in updating.
    /// It returns the new SubscriptionConfig.
    pub async fn update(&self, updating: SubscriptionConfigToUpdate, opt: Option<BackoffRetrySettings>) -> Result<(String, SubscriptionConfig), Status>{
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
    pub async fn receive<F>(&self, mut cancellation_token: CancellationToken,  f: impl Fn(ReceivedMessage) -> F + Send + 'static + Sync + Clone, config: Option<ReceiveConfig>) -> Result<(), Status>
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
            let name = self.name.clone();
            message_receivers.push(tokio::spawn(async move {
                while let Ok(message) = receiver.recv().await {
                    f_clone(message).await;
                };
                log::trace!("stop message receiver : {}", name);
            }));
        }
        cancellation_token.done().await;

        for mut subscriber in subscribers {
            subscriber.stop().await;
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::time::Duration;
    use uuid::Uuid;
    use google_cloud_googleapis::pubsub::v1::{ExpirationPolicy, Subscription as InternalSubscription};
    use serial_test::serial;
    use crate::apiv1::conn_pool::ConnectionManager;
    use crate::apiv1::subscriber_client::SubscriberClient;
    use crate::cancel::CancellationToken;
    use crate::subscription::{Subscription, SubscriptionConfigToUpdate};

    #[tokio::test]
    #[serial]
    async fn test_subscription() -> Result<(), anyhow::Error> {
        std::env::set_var("RUST_LOG","google_cloud_pubsub=trace".to_string());
        env_logger::init();
        let cm = ConnectionManager::new(4, Some("localhost:8681".to_string())).await?;
        let client = SubscriberClient::new(cm);

        let uuid = Uuid::new_v4().to_hyphenated().to_string();
        let subscription_name = format!("projects/loca-lproject/subscriptions/s{}", &uuid);
        let topic_name = "projects/local-project/topics/test-topic1".to_string();
        let subscription = client.create_subscription(InternalSubscription {
            name: subscription_name.to_string(),
            topic: topic_name.to_string(),
            push_config: None,
            ack_deadline_seconds: 0,
            retain_acked_messages: false,
            message_retention_duration: None,
            labels: Default::default(),
            enable_message_ordering: true,
            expiration_policy: None,
            filter: "".to_string(),
            dead_letter_policy: None,
            retry_policy: None,
            detached: false,
            topic_message_retention_duration: None
        }, None).await?.into_inner();

        let mut sub = Subscription::new(subscription.name, client);
        assert!(sub.exists(None).await?);

        let config = sub.config(None).await?;
        assert_eq!(config.0, topic_name);
        assert!(config.1.enable_message_ordering);

        let new_config = sub.update(SubscriptionConfigToUpdate {
            push_config: None,
            ack_deadline_seconds: Some(100),
            retain_acked_messages: None,
            message_retention_duration: None,
            labels: None,
            expiration_policy: None,
            dead_letter_policy: None,
            retry_policy: None
        }, None).await?;
        assert_eq!(new_config.0, topic_name);
        assert_eq!(new_config.1.ack_deadline_seconds, 100);

        let (ctx, cancel) = CancellationToken::new();
        let handle = tokio::spawn(async move {
            let _ = sub.receive(ctx, |mut message| async move {
                println!("{}", message.message.message_id);
                message.ack();
            }, None).await;

            sub.delete(None).await.unwrap();
            assert!(!sub.exists(None).await.unwrap())
        });
        tokio::time::sleep(Duration::from_secs(1)).await;
        drop(cancel);
        handle.await;

        Ok(())

    }
}