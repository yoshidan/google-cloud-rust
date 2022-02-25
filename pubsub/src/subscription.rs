use std::collections::HashMap;
use std::future::Future;

use google_cloud_gax::retry::RetrySetting;
use google_cloud_gax::status::{Code, Status};
use prost_types::FieldMask;
use std::time::Duration;
use tokio::select;
use tokio_util::sync::CancellationToken;

use crate::apiv1::subscriber_client::SubscriberClient;
use google_cloud_googleapis::pubsub::v1::{
    DeadLetterPolicy, DeleteSubscriptionRequest, ExpirationPolicy, GetSubscriptionRequest, PushConfig, RetryPolicy,
    Subscription as InternalSubscription, UpdateSubscriptionRequest,
};

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
            topic_message_retention_duration: None,
        }
    }
}

impl Into<SubscriptionConfig> for InternalSubscription {
    fn into(self) -> SubscriptionConfig {
        SubscriptionConfig {
            push_config: self.push_config,
            ack_deadline_seconds: self.ack_deadline_seconds,
            retain_acked_messages: self.retain_acked_messages,
            message_retention_duration: self
                .message_retention_duration
                .map(|v| std::time::Duration::new(v.seconds as u64, v.nanos as u32)),
            labels: self.labels,
            enable_message_ordering: self.enable_message_ordering,
            expiration_policy: self.expiration_policy,
            filter: self.filter,
            dead_letter_policy: self.dead_letter_policy,
            retry_policy: self.retry_policy,
            detached: self.detached,
            topic_message_retention_duration: self
                .topic_message_retention_duration
                .map(|v| std::time::Duration::new(v.seconds as u64, v.nanos as u32)),
        }
    }
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

impl Default for SubscriptionConfigToUpdate {
    fn default() -> Self {
        Self {
            push_config: None,
            ack_deadline_seconds: None,
            retain_acked_messages: None,
            message_retention_duration: None,
            labels: None,
            expiration_policy: None,
            dead_letter_policy: None,
            retry_policy: None,
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
    fqsn: String,
    subc: SubscriberClient,
}

impl Subscription {
    pub(crate) fn new(fqsn: String, subc: SubscriberClient) -> Self {
        Self { fqsn, subc }
    }

    /// id returns the unique identifier of the subscription within its project.
    pub fn id(&self) -> String {
        self.fqsn
            .rfind('/')
            .map_or("".to_string(), |i| self.fqsn[(i + 1)..].to_string())
    }

    /// string returns the globally unique printable name of the subscription.
    pub fn fully_qualified_name(&self) -> &str {
        self.fqsn.as_str()
    }

    /// create creates the subscription.
    pub async fn create(
        &self,
        ctx: CancellationToken,
        fqtn: &str,
        cfg: SubscriptionConfig,
        retry_option: Option<RetrySetting>,
    ) -> Result<(), Status> {
        self.subc
            .create_subscription(
                ctx,
                InternalSubscription {
                    name: self.fully_qualified_name().to_string(),
                    topic: fqtn.to_string(),
                    push_config: cfg.push_config,
                    ack_deadline_seconds: cfg.ack_deadline_seconds,
                    labels: cfg.labels,
                    enable_message_ordering: cfg.enable_message_ordering,
                    expiration_policy: cfg.expiration_policy,
                    filter: cfg.filter,
                    dead_letter_policy: cfg.dead_letter_policy,
                    retry_policy: cfg.retry_policy,
                    detached: cfg.detached,
                    message_retention_duration: cfg.message_retention_duration.map(|v| v.into()),
                    retain_acked_messages: cfg.retain_acked_messages,
                    topic_message_retention_duration: cfg.topic_message_retention_duration.map(|v| v.into()),
                },
                retry_option,
            )
            .await
            .map(|_v| ())
    }

    /// delete deletes the subscription.
    pub async fn delete(&self, ctx: CancellationToken, retry_option: Option<RetrySetting>) -> Result<(), Status> {
        self.subc
            .delete_subscription(
                ctx,
                DeleteSubscriptionRequest {
                    subscription: self.fqsn.to_string(),
                },
                retry_option,
            )
            .await
            .map(|v| v.into_inner())
    }

    /// exists reports whether the subscription exists on the server.
    pub async fn exists(&self, ctx: CancellationToken, retry_option: Option<RetrySetting>) -> Result<bool, Status> {
        match self
            .subc
            .get_subscription(
                ctx,
                GetSubscriptionRequest {
                    subscription: self.fqsn.to_string(),
                },
                retry_option,
            )
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                if e.code() == Code::NotFound {
                    Ok(false)
                } else {
                    Err(e)
                }
            }
        }
    }

    /// config fetches the current configuration for the subscription.
    pub async fn config(
        &self,
        ctx: CancellationToken,
        retry_option: Option<RetrySetting>,
    ) -> Result<(String, SubscriptionConfig), Status> {
        self.subc
            .get_subscription(
                ctx,
                GetSubscriptionRequest {
                    subscription: self.fqsn.to_string(),
                },
                retry_option,
            )
            .await
            .map(|v| {
                let inner = v.into_inner();
                (inner.topic.to_string(), inner.into())
            })
    }

    /// update changes an existing subscription according to the fields set in updating.
    /// It returns the new SubscriptionConfig.
    pub async fn update(
        &self,
        ctx: CancellationToken,
        updating: SubscriptionConfigToUpdate,
        opt: Option<RetrySetting>,
    ) -> Result<(String, SubscriptionConfig), Status> {
        let mut config = self
            .subc
            .get_subscription(
                ctx.clone(),
                GetSubscriptionRequest {
                    subscription: self.fqsn.to_string(),
                },
                opt.clone(),
            )
            .await?
            .into_inner();

        let mut paths = vec![];
        if updating.push_config.is_some() {
            config.push_config = updating.push_config;
            paths.push("push_config".to_string());
        }
        if let Some(v) = updating.ack_deadline_seconds {
            config.ack_deadline_seconds = v;
            paths.push("ack_deadline_seconds".to_string());
        }
        if let Some(v) = updating.retain_acked_messages {
            config.retain_acked_messages = v;
            paths.push("retain_acked_messages".to_string());
        }
        if updating.message_retention_duration.is_some() {
            let v = updating
                .message_retention_duration
                .map(|v| prost_types::Duration::from(v));
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

        self.subc
            .update_subscription(
                ctx,
                UpdateSubscriptionRequest {
                    subscription: Some(config.into()),
                    update_mask: Some(FieldMask { paths }),
                },
                opt,
            )
            .await
            .map(|v| {
                let inner = v.into_inner();
                (inner.topic.to_string(), inner.into())
            })
    }

    /// receive calls f with the outstanding messages from the subscription.
    /// It blocks until ctx is done, or the service returns a non-retryable error.
    /// The standard way to terminate a receive is to use CancellationToken.
    pub async fn receive<F>(
        &self,
        ctx: CancellationToken,
        f: impl Fn(ReceivedMessage, CancellationToken) -> F + Send + 'static + Sync + Clone,
        config: Option<ReceiveConfig>,
    ) -> Result<(), Status>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let op = config.unwrap_or_default();
        let mut receivers = Vec::with_capacity(op.worker_count);
        let mut senders = Vec::with_capacity(receivers.len());

        if self
            .config(ctx.clone(), op.subscriber_config.retry_setting.clone())
            .await?
            .1
            .enable_message_ordering
        {
            (0..op.worker_count).for_each(|_v| {
                let (sender, receiver) = async_channel::unbounded::<ReceivedMessage>();
                receivers.push(receiver);
                senders.push(sender);
            });
        } else {
            let (sender, receiver) = async_channel::unbounded::<ReceivedMessage>();
            (0..op.worker_count).for_each(|_v| {
                receivers.push(receiver.clone());
                senders.push(sender.clone());
            });
        }

        //same ordering key is in same stream.
        let subscribers: Vec<Subscriber> = senders
            .into_iter()
            .map(|queue| {
                Subscriber::start(
                    ctx.clone(),
                    self.fqsn.clone(),
                    self.subc.clone(),
                    queue,
                    Some(op.subscriber_config.clone()),
                )
            })
            .collect();

        let mut message_receivers = Vec::with_capacity(receivers.len());
        for receiver in receivers {
            let f_clone = f.clone();
            let ctx_clone = ctx.clone();
            let name = self.fqsn.clone();
            message_receivers.push(tokio::spawn(async move {
                loop {
                    select! {
                        _ = ctx_clone.cancelled() => break,
                        msg = receiver.recv() => match msg {
                            Ok(message) => f_clone(message, ctx_clone.clone()).await,
                            Err(_) => break
                        }

                    }
                }
                log::trace!("stop message receiver : {}", name);
            }));
        }
        ctx.cancelled().await;

        // wait for all the treads finish.
        for mut subscriber in subscribers {
            subscriber.done().await;
        }
        for mr in message_receivers {
            mr.await;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::apiv1::conn_pool::ConnectionManager;
    use crate::apiv1::publisher_client::PublisherClient;
    use crate::apiv1::subscriber_client::SubscriberClient;
    use crate::subscription::{Subscription, SubscriptionConfig, SubscriptionConfigToUpdate};
    use google_cloud_googleapis::pubsub::v1::{PublishRequest, PubsubMessage};
    use serial_test::serial;
    use std::sync::atomic::AtomicU32;
    use std::sync::atomic::Ordering::SeqCst;
    use std::sync::Arc;
    use std::time::Duration;
    use tokio_util::sync::CancellationToken;
    use uuid::Uuid;

    #[ctor::ctor]
    fn init() {
        std::env::set_var("RUST_LOG", "google_cloud_pubsub=trace".to_string());
        env_logger::try_init();
    }

    async fn create_subscription() -> Result<Subscription, anyhow::Error> {
        let cm = ConnectionManager::new(4, Some("localhost:8681".to_string())).await?;
        let client = SubscriberClient::new(cm);

        let uuid = Uuid::new_v4().to_hyphenated().to_string();
        let subscription_name = format!("projects/loca-lproject/subscriptions/s{}", &uuid);
        let topic_name = "projects/local-project/topics/test-topic1";
        let ctx = CancellationToken::new();
        let subscription = Subscription::new(subscription_name, client);
        if !subscription.exists(ctx.clone(), None).await? {
            subscription
                .create(ctx.clone(), topic_name, SubscriptionConfig::default(), None)
                .await?;
        }
        return Ok(subscription);
    }

    async fn publish() {
        let pubc = PublisherClient::new(
            ConnectionManager::new(4, Some("localhost:8681".to_string()))
                .await
                .unwrap(),
        );
        let mut msg = PubsubMessage::default();
        msg.data = "test_message".into();
        pubc.publish(
            CancellationToken::new(),
            PublishRequest {
                topic: "projects/local-project/topics/test-topic1".to_string(),
                messages: vec![msg],
            },
            None,
        )
        .await;
    }

    #[tokio::test]
    #[serial]
    async fn test_subscription() -> Result<(), anyhow::Error> {
        let subscription = create_subscription().await.unwrap();

        let topic_name = "projects/local-project/topics/test-topic1";
        let ctx = CancellationToken::new();
        let config = subscription.config(ctx.clone(), None).await?;
        assert_eq!(config.0, topic_name);

        let mut updating = SubscriptionConfigToUpdate::default();
        updating.ack_deadline_seconds = Some(100);
        let new_config = subscription.update(ctx.clone(), updating, None).await?;
        assert_eq!(new_config.0, topic_name);
        assert_eq!(new_config.1.ack_deadline_seconds, 100);

        let cancellation_token = CancellationToken::new();
        let cancel_receiver = cancellation_token.clone();
        let handle = tokio::spawn(async move {
            let _ = subscription
                .receive(
                    cancel_receiver,
                    |message, _ctx| async move {
                        println!("{}", message.message.message_id);
                        message.ack();
                    },
                    None,
                )
                .await;
            subscription.delete(ctx.clone(), None).await.unwrap();
            assert!(!subscription.exists(ctx.clone(), None).await.unwrap())
        });
        tokio::time::sleep(Duration::from_secs(3)).await;
        cancellation_token.cancel();
        handle.await;

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_multi_subscriber_single_subscription() -> Result<(), anyhow::Error> {
        let subscription = create_subscription().await.unwrap();
        let _ctx = CancellationToken::new();

        let cancellation_token = CancellationToken::new();
        let cancel_receiver = cancellation_token.clone();
        let v = Arc::new(AtomicU32::new(0));
        let v2 = v.clone();
        let handle = tokio::spawn(async move {
            let _ = subscription
                .receive(
                    cancel_receiver,
                    move |message, _ctx| {
                        let v2 = v2.clone();
                        async move {
                            v2.fetch_add(1, SeqCst);
                            message.ack();
                        }
                    },
                    None,
                )
                .await;
        });
        publish().await;
        tokio::time::sleep(Duration::from_secs(3)).await;
        cancellation_token.cancel();
        handle.await;
        assert_eq!(v.load(SeqCst), 1);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_multi_subscriber_multi_subscription() -> Result<(), anyhow::Error> {
        let mut subscriptions = vec![];

        let ctx = CancellationToken::new();
        for _ in 0..3 {
            let subscription = create_subscription().await?;
            let v = Arc::new(AtomicU32::new(0));
            let ctx = ctx.clone();
            let v2 = v.clone();
            let handle = tokio::spawn(async move {
                let _ = subscription
                    .receive(
                        ctx,
                        move |message, _ctx| {
                            let v2 = v2.clone();
                            async move {
                                v2.fetch_add(1, SeqCst);
                                message.ack();
                            }
                        },
                        None,
                    )
                    .await;
            });
            subscriptions.push((handle, v))
        }

        publish().await;
        tokio::time::sleep(Duration::from_secs(5)).await;

        ctx.cancel();
        for (task, v) in subscriptions {
            task.await;
            assert_eq!(v.load(SeqCst), 1);
        }
        Ok(())
    }
}
