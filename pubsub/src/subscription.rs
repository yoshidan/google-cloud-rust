use std::collections::HashMap;
use std::future::Future;

use google_cloud_gax::cancel::CancellationToken;
use google_cloud_gax::grpc::{Code, Status};
use google_cloud_gax::retry::RetrySetting;
use prost_types::FieldMask;
use std::time::Duration;

use crate::apiv1::subscriber_client::SubscriberClient;
use google_cloud_googleapis::pubsub::v1::{
    BigQueryConfig, DeadLetterPolicy, DeleteSubscriptionRequest, ExpirationPolicy, GetSubscriptionRequest, PullRequest,
    PushConfig, RetryPolicy, Subscription as InternalSubscription, UpdateSubscriptionRequest,
};

use crate::subscriber::{ReceivedMessage, Subscriber, SubscriberConfig};

#[derive(Default)]
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
    pub enable_exactly_once_delivery: bool,
    pub bigquery_config: Option<BigQueryConfig>,
    pub state: i32,
}

impl From<InternalSubscription> for SubscriptionConfig {
    fn from(f: InternalSubscription) -> Self {
        Self {
            push_config: f.push_config,
            bigquery_config: f.bigquery_config,
            ack_deadline_seconds: f.ack_deadline_seconds,
            retain_acked_messages: f.retain_acked_messages,
            message_retention_duration: f
                .message_retention_duration
                .map(|v| std::time::Duration::new(v.seconds as u64, v.nanos as u32)),
            labels: f.labels,
            enable_message_ordering: f.enable_message_ordering,
            expiration_policy: f.expiration_policy,
            filter: f.filter,
            dead_letter_policy: f.dead_letter_policy,
            retry_policy: f.retry_policy,
            detached: f.detached,
            topic_message_retention_duration: f
                .topic_message_retention_duration
                .map(|v| std::time::Duration::new(v.seconds as u64, v.nanos as u32)),
            enable_exactly_once_delivery: f.enable_exactly_once_delivery,
            state: f.state,
        }
    }
}

#[derive(Default)]
pub struct SubscriptionConfigToUpdate {
    pub push_config: Option<PushConfig>,
    pub bigquery_config: Option<BigQueryConfig>,
    pub ack_deadline_seconds: Option<i32>,
    pub retain_acked_messages: Option<bool>,
    pub message_retention_duration: Option<Duration>,
    pub labels: Option<HashMap<String, String>>,
    pub expiration_policy: Option<ExpirationPolicy>,
    pub dead_letter_policy: Option<DeadLetterPolicy>,
    pub retry_policy: Option<RetryPolicy>,
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
#[derive(Debug)]
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

    /// fully_qualified_name returns the globally unique printable name of the subscription.
    pub fn fully_qualified_name(&self) -> &str {
        self.fqsn.as_str()
    }

    /// create creates the subscription.
    pub async fn create(
        &self,
        fqtn: &str,
        cfg: SubscriptionConfig,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<(), Status> {
        self.subc
            .create_subscription(
                InternalSubscription {
                    name: self.fully_qualified_name().to_string(),
                    topic: fqtn.to_string(),
                    push_config: cfg.push_config,
                    bigquery_config: cfg.bigquery_config,
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
                    enable_exactly_once_delivery: cfg.enable_exactly_once_delivery,
                    state: cfg.state,
                },
                cancel,
                retry,
            )
            .await
            .map(|_v| ())
    }

    /// delete deletes the subscription.
    pub async fn delete(&self, cancel: Option<CancellationToken>, retry: Option<RetrySetting>) -> Result<(), Status> {
        let req = DeleteSubscriptionRequest {
            subscription: self.fqsn.to_string(),
        };
        self.subc
            .delete_subscription(req, cancel, retry)
            .await
            .map(|v| v.into_inner())
    }

    /// exists reports whether the subscription exists on the server.
    pub async fn exists(&self, cancel: Option<CancellationToken>, retry: Option<RetrySetting>) -> Result<bool, Status> {
        let req = GetSubscriptionRequest {
            subscription: self.fqsn.to_string(),
        };
        match self.subc.get_subscription(req, cancel, retry).await {
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
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<(String, SubscriptionConfig), Status> {
        let req = GetSubscriptionRequest {
            subscription: self.fqsn.to_string(),
        };
        self.subc.get_subscription(req, cancel, retry).await.map(|v| {
            let inner = v.into_inner();
            (inner.topic.to_string(), inner.into())
        })
    }

    /// update changes an existing subscription according to the fields set in updating.
    /// It returns the new SubscriptionConfig.
    pub async fn update(
        &self,
        updating: SubscriptionConfigToUpdate,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<(String, SubscriptionConfig), Status> {
        let req = GetSubscriptionRequest {
            subscription: self.fqsn.to_string(),
        };
        let mut config = self
            .subc
            .get_subscription(req, cancel.clone(), retry.clone())
            .await?
            .into_inner();

        let mut paths = vec![];
        if updating.push_config.is_some() {
            config.push_config = updating.push_config;
            paths.push("push_config".to_string());
        }
        if updating.bigquery_config.is_some() {
            config.bigquery_config = updating.bigquery_config;
            paths.push("bigquery_config".to_string());
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
            let v = updating.message_retention_duration.map(prost_types::Duration::from);
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

        let update_req = UpdateSubscriptionRequest {
            subscription: Some(config),
            update_mask: Some(FieldMask { paths }),
        };
        self.subc.update_subscription(update_req, cancel, retry).await.map(|v| {
            let inner = v.into_inner();
            (inner.topic.to_string(), inner.into())
        })
    }

    /// pull get message synchronously.
    /// It blocks until at least one message is available.
    pub async fn pull(
        &self,
        max_messages: i32,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Vec<ReceivedMessage>, Status> {
        #[allow(deprecated)]
        let req = PullRequest {
            subscription: self.fqsn.clone(),
            return_immediately: false,
            max_messages,
        };
        let messages = self.subc.pull(req, cancel, retry).await?.into_inner().received_messages;
        Ok(messages
            .into_iter()
            .filter(|m| m.message.is_some())
            .map(|m| ReceivedMessage::new(self.fqsn.clone(), self.subc.clone(), m.message.unwrap(), m.ack_id))
            .collect())
    }

    /// receive calls f with the outstanding messages from the subscription.
    /// It blocks until cancellation token is cancelled, or the service returns a non-retryable error.
    /// The standard way to terminate a receive is to use CancellationToken.
    pub async fn receive<F>(
        &self,
        f: impl Fn(ReceivedMessage, CancellationToken) -> F + Send + 'static + Sync + Clone,
        cancel: CancellationToken,
        config: Option<ReceiveConfig>,
    ) -> Result<(), Status>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let op = config.unwrap_or_default();
        let mut receivers = Vec::with_capacity(op.worker_count);
        let mut senders = Vec::with_capacity(receivers.len());

        if self
            .config(Some(cancel.clone()), op.subscriber_config.retry_setting.clone())
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
                    cancel.clone(),
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
            let cancel_clone = cancel.clone();
            let name = self.fqsn.clone();
            message_receivers.push(tokio::spawn(async move {
                while let Ok(message) = receiver.recv().await {
                    f_clone(message, cancel_clone.clone()).await;
                }
                // queue is closed by subscriber when the cancellation token is cancelled
                tracing::trace!("stop message receiver : {}", name);
            }));
        }
        cancel.cancelled().await;

        // wait for all the treads finish.
        for mut subscriber in subscribers {
            subscriber.done().await;
        }

        // wait for all the receivers process received messages
        for mr in message_receivers {
            let _ = mr.await;
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
    use google_cloud_gax::cancel::CancellationToken;
    use google_cloud_gax::grpc::Code;
    use google_cloud_googleapis::pubsub::v1::{PublishRequest, PubsubMessage};
    use serial_test::serial;
    use std::sync::atomic::AtomicU32;
    use std::sync::atomic::Ordering::SeqCst;
    use std::sync::Arc;

    use google_cloud_gax::conn::Environment;
    use std::time::Duration;
    use uuid::Uuid;

    const PROJECT_NAME: &str = "local-project";
    const EMULATOR: &str = "localhost:8681";

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt().try_init();
    }

    async fn create_subscription(enable_exactly_once_delivery: bool) -> Result<Subscription, anyhow::Error> {
        let cm = ConnectionManager::new(4, &Environment::Emulator(EMULATOR.to_string()), "").await?;
        let client = SubscriberClient::new(cm);

        let uuid = Uuid::new_v4().to_hyphenated().to_string();
        let subscription_name = format!("projects/{}/subscriptions/s{}", PROJECT_NAME, &uuid);
        let topic_name = format!("projects/{}/topics/test-topic1", PROJECT_NAME);
        let cancel = CancellationToken::new();
        let subscription = Subscription::new(subscription_name, client);
        let config = SubscriptionConfig {
            enable_exactly_once_delivery,
            ..Default::default()
        };
        if !subscription.exists(Some(cancel.clone()), None).await? {
            subscription
                .create(topic_name.as_str(), config, Some(cancel), None)
                .await?;
        }
        Ok(subscription)
    }

    async fn publish() {
        let pubc = PublisherClient::new(
            ConnectionManager::new(4, &Environment::Emulator(EMULATOR.to_string()), "")
                .await
                .unwrap(),
        );
        let msg = PubsubMessage {
            data: "test_message".into(),
            ..Default::default()
        };
        let req = PublishRequest {
            topic: format!("projects/{}/topics/test-topic1", PROJECT_NAME),
            messages: vec![msg],
        };
        let _ = pubc.publish(req, Some(CancellationToken::new()), None).await;
    }

    async fn test_subscription(enable_exactly_once_delivery: bool) -> Result<(), anyhow::Error> {
        let subscription = create_subscription(enable_exactly_once_delivery).await.unwrap();

        let topic_name = format!("projects/{}/topics/test-topic1", PROJECT_NAME);
        let cancel = CancellationToken::new();
        let config = subscription.config(Some(cancel.clone()), None).await?;
        assert_eq!(config.0, topic_name);

        let updating = SubscriptionConfigToUpdate {
            ack_deadline_seconds: Some(100),
            ..Default::default()
        };
        let new_config = subscription.update(updating, Some(cancel.clone()), None).await?;
        assert_eq!(new_config.0, topic_name);
        assert_eq!(new_config.1.ack_deadline_seconds, 100);

        let receiver_ctx = CancellationToken::new();
        let cancel_receiver = receiver_ctx.clone();
        let handle = tokio::spawn(async move {
            let _ = subscription
                .receive(
                    |message, _ctx| async move {
                        println!("{}", message.message.message_id);
                        let _ = message.ack();
                    },
                    cancel_receiver,
                    None,
                )
                .await;
            subscription.delete(Some(cancel.clone()), None).await.unwrap();
            assert!(!subscription.exists(Some(cancel.clone()), None).await.unwrap())
        });
        tokio::time::sleep(Duration::from_secs(3)).await;
        receiver_ctx.cancel();
        let _ = handle.await;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_pull() -> Result<(), anyhow::Error> {
        let subscription = create_subscription(false).await.unwrap();
        publish().await;
        publish().await;
        publish().await;
        let messages = subscription.pull(2, None, None).await?;
        assert_eq!(messages.len(), 2);
        for m in messages {
            m.ack().await.unwrap();
        }
        subscription.delete(None, None).await?;
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_pull_cancel() -> Result<(), anyhow::Error> {
        let subscription = create_subscription(false).await.unwrap();
        let cancel = CancellationToken::new();
        let cancel2 = cancel.clone();
        let j = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(5)).await;
            tracing::info!("cancelled");
            cancel2.clone().cancel();
        });
        let messages = subscription.pull(2, Some(cancel), None).await;
        match messages {
            Ok(_v) => panic!("must error"),
            Err(e) => {
                assert_eq!(e.code(), Code::Cancelled);
            }
        }
        let _ = j.await;
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_subscription_exactly_once() -> Result<(), anyhow::Error> {
        test_subscription(true).await
    }

    #[tokio::test]
    #[serial]
    async fn test_subscription_at_least_once() -> Result<(), anyhow::Error> {
        test_subscription(false).await
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_multi_subscriber_single_subscription() -> Result<(), anyhow::Error> {
        let subscription = create_subscription(false).await.unwrap();
        let cancellation_token = CancellationToken::new();
        let cancel_receiver = cancellation_token.clone();
        let v = Arc::new(AtomicU32::new(0));
        let v2 = v.clone();
        let handle = tokio::spawn(async move {
            let _ = subscription
                .receive(
                    move |message, _ctx| {
                        let v2 = v2.clone();
                        async move {
                            v2.fetch_add(1, SeqCst);
                            let _ = message.ack();
                        }
                    },
                    cancel_receiver,
                    None,
                )
                .await;
        });
        publish().await;
        tokio::time::sleep(Duration::from_secs(3)).await;
        cancellation_token.cancel();
        let _ = handle.await;
        assert_eq!(v.load(SeqCst), 1);
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_multi_subscriber_multi_subscription() -> Result<(), anyhow::Error> {
        let mut subscriptions = vec![];

        let ctx = CancellationToken::new();
        for _ in 0..3 {
            let subscription = create_subscription(false).await?;
            let v = Arc::new(AtomicU32::new(0));
            let ctx = ctx.clone();
            let v2 = v.clone();
            let handle = tokio::spawn(async move {
                let _ = subscription
                    .receive(
                        move |message, _ctx| {
                            let v2 = v2.clone();
                            async move {
                                v2.fetch_add(1, SeqCst);
                                let _ = message.ack();
                            }
                        },
                        ctx,
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
            let _ = task.await;
            assert_eq!(v.load(SeqCst), 1);
        }
        Ok(())
    }

    /*
    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn long_polling() -> Result<(), anyhow::Error> {
        let subscription = create_subscription().await.unwrap();
        let cancel = CancellationToken::new();
        subscription.receive(|message, _| async move{
            tracing::info!("received {}", message.message.message_id);
            message.ack().await;
        }, cancel, None).await;
        Ok(())
    }
     */
}
