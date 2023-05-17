use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, SystemTime};

use prost_types::{DurationError, FieldMask};
use tokio_util::sync::CancellationToken;

use google_cloud_gax::grpc::codegen::futures_core::Stream;
use google_cloud_gax::grpc::{Code, Status};
use google_cloud_gax::retry::RetrySetting;
use google_cloud_googleapis::pubsub::v1::seek_request::Target;
use google_cloud_googleapis::pubsub::v1::{
    BigQueryConfig, CreateSnapshotRequest, DeadLetterPolicy, DeleteSnapshotRequest, DeleteSubscriptionRequest,
    ExpirationPolicy, GetSnapshotRequest, GetSubscriptionRequest, PullRequest, PushConfig, RetryPolicy, SeekRequest,
    Snapshot, Subscription as InternalSubscription, UpdateSubscriptionRequest,
};

use crate::apiv1::subscriber_client::SubscriberClient;
use crate::subscriber::{ack, ReceivedMessage, Subscriber, SubscriberConfig};

#[derive(Debug, Clone, Default)]
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

#[derive(Debug, Clone, Default)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum SeekTo {
    Timestamp(SystemTime),
    Snapshot(String),
}

impl From<SeekTo> for Target {
    fn from(to: SeekTo) -> Target {
        use SeekTo::*;
        match to {
            Timestamp(t) => Target::Time(prost_types::Timestamp::from(t)),
            Snapshot(s) => Target::Snapshot(s),
        }
    }
}

pub struct MessageStream {
    queue: async_channel::Receiver<ReceivedMessage>,
    cancel: CancellationToken,
}

impl Drop for MessageStream {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}

impl Stream for MessageStream {
    type Item = ReceivedMessage;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().queue).poll_next(cx)
    }
}

/// Subscription is a reference to a PubSub subscription.
#[derive(Clone, Debug)]
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

    /// fully_qualified_snapshot_name returns the globally unique printable name of the snapshot.
    pub fn fully_qualified_snapshot_name(&self, id: &str) -> String {
        if id.contains('/') {
            id.to_string()
        } else {
            format!("{}/snapshots/{}", self.fully_qualified_project_name(), id)
        }
    }

    fn fully_qualified_project_name(&self) -> String {
        let parts: Vec<_> = self
            .fqsn
            .split('/')
            .enumerate()
            .filter(|&(i, _)| i < 2)
            .map(|e| e.1)
            .collect();
        parts.join("/")
    }

    /// create creates the subscription.
    pub async fn create(&self, fqtn: &str, cfg: SubscriptionConfig, retry: Option<RetrySetting>) -> Result<(), Status> {
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
                    message_retention_duration: cfg
                        .message_retention_duration
                        .map(Duration::try_into)
                        .transpose()
                        .map_err(|err: DurationError| Status::internal(err.to_string()))?,
                    retain_acked_messages: cfg.retain_acked_messages,
                    topic_message_retention_duration: cfg
                        .topic_message_retention_duration
                        .map(Duration::try_into)
                        .transpose()
                        .map_err(|err: DurationError| Status::internal(err.to_string()))?,
                    enable_exactly_once_delivery: cfg.enable_exactly_once_delivery,
                    state: cfg.state,
                },
                retry,
            )
            .await
            .map(|_v| ())
    }

    /// delete deletes the subscription.
    pub async fn delete(&self, retry: Option<RetrySetting>) -> Result<(), Status> {
        let req = DeleteSubscriptionRequest {
            subscription: self.fqsn.to_string(),
        };
        self.subc.delete_subscription(req, retry).await.map(|v| v.into_inner())
    }

    /// exists reports whether the subscription exists on the server.
    pub async fn exists(&self, retry: Option<RetrySetting>) -> Result<bool, Status> {
        let req = GetSubscriptionRequest {
            subscription: self.fqsn.to_string(),
        };
        match self.subc.get_subscription(req, retry).await {
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
    pub async fn config(&self, retry: Option<RetrySetting>) -> Result<(String, SubscriptionConfig), Status> {
        let req = GetSubscriptionRequest {
            subscription: self.fqsn.to_string(),
        };
        self.subc.get_subscription(req, retry).await.map(|v| {
            let inner = v.into_inner();
            (inner.topic.to_string(), inner.into())
        })
    }

    /// update changes an existing subscription according to the fields set in updating.
    /// It returns the new SubscriptionConfig.
    pub async fn update(
        &self,
        updating: SubscriptionConfigToUpdate,
        retry: Option<RetrySetting>,
    ) -> Result<(String, SubscriptionConfig), Status> {
        let req = GetSubscriptionRequest {
            subscription: self.fqsn.to_string(),
        };
        let mut config = self.subc.get_subscription(req, retry.clone()).await?.into_inner();

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
            config.message_retention_duration = updating
                .message_retention_duration
                .map(prost_types::Duration::try_from)
                .transpose()
                .map_err(|err| Status::internal(err.to_string()))?;
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
        self.subc.update_subscription(update_req, retry).await.map(|v| {
            let inner = v.into_inner();
            (inner.topic.to_string(), inner.into())
        })
    }

    /// pull get message synchronously.
    /// It blocks until at least one message is available.
    pub async fn pull(&self, max_messages: i32, retry: Option<RetrySetting>) -> Result<Vec<ReceivedMessage>, Status> {
        #[allow(deprecated)]
        let req = PullRequest {
            subscription: self.fqsn.clone(),
            return_immediately: false,
            max_messages,
        };
        let messages = self.subc.pull(req, retry).await?.into_inner().received_messages;
        Ok(messages
            .into_iter()
            .filter(|m| m.message.is_some())
            .map(|m| ReceivedMessage::new(self.fqsn.clone(), self.subc.clone(), m.message.unwrap(), m.ack_id))
            .collect())
    }

    /// subscribe creates a `Stream` of `ReceivedMessage`
    /// Terminates the underlying `Subscriber` when dropped.
    /// ```no_test
    /// use google_cloud_pubsub::client::Client;
    /// use google_cloud_pubsub::subscription::Subscription;
    /// use google_cloud_gax::grpc::Status;
    /// use futures_util::StreamExt;
    ///
    /// async fn run(client: Client) -> Result<(), Status> {
    ///     let subscription = client.subscription("test-subscription");
    ///     let mut iter = subscription.subscribe(None).await?;
    ///     while let Some(message) = iter.next().await {
    ///         let _ = message.ack().await;
    ///     }
    ///     Ok(())
    ///  }
    /// ```
    pub async fn subscribe(&self, opt: Option<SubscriberConfig>) -> Result<MessageStream, Status> {
        let (tx, rx) = async_channel::unbounded::<ReceivedMessage>();

        let cancel = CancellationToken::new();
        Subscriber::start(cancel.clone(), self.fqsn.clone(), self.subc.clone(), tx, opt);

        Ok(MessageStream { queue: rx, cancel })
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
            .config(op.subscriber_config.retry_setting.clone())
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

    /// Ack acknowledges the messages associated with the ack_ids in the AcknowledgeRequest.
    /// The Pub/Sub system can remove the relevant messages from the subscription.
    /// This method is for batch acking.
    ///
    /// ```
    /// use google_cloud_pubsub::client::Client;
    /// use google_cloud_pubsub::subscription::Subscription;
    /// use google_cloud_gax::grpc::Status;
    /// use std::time::Duration;
    /// use tokio_util::sync::CancellationToken;;
    ///
    /// #[tokio::main]
    /// async fn run(client: Client) -> Result<(), Status> {
    ///     let subscription = client.subscription("test-subscription");
    ///     let ctx = CancellationToken::new();
    ///     let (sender, mut receiver)  = tokio::sync::mpsc::unbounded_channel();
    ///     let subscription_for_receive = subscription.clone();
    ///     let ctx_for_receive = ctx.clone();
    ///     let ctx_for_ack_manager = ctx.clone();
    ///
    ///     // receive
    ///     let handle = tokio::spawn(async move {
    ///         let _ = subscription_for_receive.receive(move |message, _ctx| {
    ///             let sender = sender.clone();
    ///             async move {
    ///                 let _ = sender.send(message.ack_id().to_string());
    ///             }
    ///         }, ctx_for_receive.clone(), None).await;
    ///     });
    ///
    ///     // batch ack manager
    ///     let ack_manager = tokio::spawn( async move {
    ///         let mut ack_ids = Vec::new();
    ///         loop {
    ///             tokio::select! {
    ///                 _ = ctx_for_ack_manager.cancelled() => {
    ///                     return subscription.ack(ack_ids).await;
    ///                 },
    ///                 r = tokio::time::timeout(Duration::from_secs(10), receiver.recv()) => match r {
    ///                     Ok(ack_id) => {
    ///                         if let Some(ack_id) = ack_id {
    ///                             ack_ids.push(ack_id);
    ///                             if ack_ids.len() > 10 {
    ///                                 let _ = subscription.ack(ack_ids).await;
    ///                                 ack_ids = Vec::new();
    ///                             }
    ///                         }
    ///                     },
    ///                     Err(_e) => {
    ///                         // timeout
    ///                         let _ = subscription.ack(ack_ids).await;
    ///                         ack_ids = Vec::new();
    ///                     }
    ///                 }
    ///             }
    ///         }
    ///     });
    ///
    ///     ctx.cancel();
    ///     Ok(())
    ///  }
    /// ```
    pub async fn ack(&self, ack_ids: Vec<String>) -> Result<(), Status> {
        ack(&self.subc, self.fqsn.to_string(), ack_ids).await
    }

    /// seek seeks the subscription a past timestamp or a saved snapshot.
    pub async fn seek(&self, to: SeekTo, retry: Option<RetrySetting>) -> Result<(), Status> {
        let to = match to {
            SeekTo::Timestamp(t) => SeekTo::Timestamp(t),
            SeekTo::Snapshot(name) => SeekTo::Snapshot(self.fully_qualified_snapshot_name(name.as_str())),
        };

        let req = SeekRequest {
            subscription: self.fqsn.to_owned(),
            target: Some(to.into()),
        };

        let _ = self.subc.seek(req, retry).await?;
        Ok(())
    }

    /// get_snapshot fetches an existing pubsub snapshot.
    pub async fn get_snapshot(&self, name: &str, retry: Option<RetrySetting>) -> Result<Snapshot, Status> {
        let req = GetSnapshotRequest {
            snapshot: self.fully_qualified_snapshot_name(name),
        };
        Ok(self.subc.get_snapshot(req, retry).await?.into_inner())
    }

    /// create_snapshot creates a new pubsub snapshot from the subscription's state at the time of calling.
    /// The snapshot retains the messages for the topic the subscription is subscribed to, with the acknowledgment
    /// states consistent with the subscriptions.
    /// The created snapshot is guaranteed to retain:
    /// - The message backlog on the subscription -- or to be specific, messages that are unacknowledged
    ///   at the time of the subscription's creation.
    /// - All messages published to the subscription's topic after the snapshot's creation.
    /// Snapshots have a finite lifetime -- a maximum of 7 days from the time of creation, beyond which
    /// they are discarded and any messages being retained solely due to the snapshot dropped.
    pub async fn create_snapshot(
        &self,
        name: &str,
        labels: HashMap<String, String>,
        retry: Option<RetrySetting>,
    ) -> Result<Snapshot, Status> {
        let req = CreateSnapshotRequest {
            name: self.fully_qualified_snapshot_name(name),
            labels,
            subscription: self.fqsn.to_owned(),
        };
        Ok(self.subc.create_snapshot(req, retry).await?.into_inner())
    }

    /// delete_snapshot deletes an existing pubsub snapshot.
    pub async fn delete_snapshot(&self, name: &str, retry: Option<RetrySetting>) -> Result<(), Status> {
        let req = DeleteSnapshotRequest {
            snapshot: self.fully_qualified_snapshot_name(name),
        };
        let _ = self.subc.delete_snapshot(req, retry).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::atomic::AtomicU32;
    use std::sync::atomic::Ordering::SeqCst;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use futures_util::StreamExt;
    use serial_test::serial;
    use tokio_util::sync::CancellationToken;
    use uuid::Uuid;

    use google_cloud_gax::conn::Environment;
    use google_cloud_googleapis::pubsub::v1::{PublishRequest, PubsubMessage};

    use crate::apiv1::conn_pool::ConnectionManager;
    use crate::apiv1::publisher_client::PublisherClient;
    use crate::apiv1::subscriber_client::SubscriberClient;
    use crate::subscriber::ReceivedMessage;
    use crate::subscription::{SeekTo, Subscription, SubscriptionConfig, SubscriptionConfigToUpdate};

    const PROJECT_NAME: &str = "local-project";
    const EMULATOR: &str = "localhost:8681";

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt().try_init();
    }

    async fn create_subscription(enable_exactly_once_delivery: bool) -> Subscription {
        let cm = ConnectionManager::new(4, "", &Environment::Emulator(EMULATOR.to_string()))
            .await
            .unwrap();
        let client = SubscriberClient::new(cm);

        let uuid = Uuid::new_v4().hyphenated().to_string();
        let subscription_name = format!("projects/{}/subscriptions/s{}", PROJECT_NAME, &uuid);
        let topic_name = format!("projects/{PROJECT_NAME}/topics/test-topic1");
        let subscription = Subscription::new(subscription_name, client);
        let config = SubscriptionConfig {
            enable_exactly_once_delivery,
            ..Default::default()
        };
        if !subscription.exists(None).await.unwrap() {
            subscription.create(topic_name.as_str(), config, None).await.unwrap();
        }
        subscription
    }

    async fn publish() {
        let pubc = PublisherClient::new(
            ConnectionManager::new(4, "", &Environment::Emulator(EMULATOR.to_string()))
                .await
                .unwrap(),
        );
        let msg = PubsubMessage {
            data: "test_message".into(),
            ..Default::default()
        };
        let req = PublishRequest {
            topic: format!("projects/{PROJECT_NAME}/topics/test-topic1"),
            messages: vec![msg],
        };
        let _ = pubc.publish(req, None).await;
    }

    async fn test_subscription(enable_exactly_once_delivery: bool) {
        let subscription = create_subscription(enable_exactly_once_delivery).await;

        let topic_name = format!("projects/{PROJECT_NAME}/topics/test-topic1");
        let config = subscription.config(None).await.unwrap();
        assert_eq!(config.0, topic_name);

        let updating = SubscriptionConfigToUpdate {
            ack_deadline_seconds: Some(100),
            ..Default::default()
        };
        let new_config = subscription.update(updating, None).await.unwrap();
        assert_eq!(new_config.0, topic_name);
        assert_eq!(new_config.1.ack_deadline_seconds, 100);

        let receiver_ctx = CancellationToken::new();
        let cancel_receiver = receiver_ctx.clone();
        let handle = tokio::spawn(async move {
            let _ = subscription
                .receive(
                    |message, _ctx| async move {
                        println!("{}", message.message.message_id);
                        let _ = message.ack().await;
                    },
                    cancel_receiver,
                    None,
                )
                .await;
            subscription.delete(None).await.unwrap();
            assert!(!subscription.exists(None).await.unwrap())
        });
        tokio::time::sleep(Duration::from_secs(3)).await;
        receiver_ctx.cancel();
        let _ = handle.await;
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_pull() {
        let subscription = create_subscription(false).await;
        publish().await;
        publish().await;
        publish().await;
        let messages = subscription.pull(2, None).await.unwrap();
        assert_eq!(messages.len(), 2);
        for m in messages {
            m.ack().await.unwrap();
        }
        subscription.delete(None).await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_subscription_exactly_once() {
        test_subscription(true).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_subscription_at_least_once() {
        test_subscription(false).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_multi_subscriber_single_subscription() {
        let subscription = create_subscription(false).await;
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
                            let _ = message.ack().await;
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
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_multi_subscriber_multi_subscription() {
        let mut subscriptions = vec![];

        let ctx = CancellationToken::new();
        for _ in 0..3 {
            let subscription = create_subscription(false).await;
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
                                let _ = message.ack().await;
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
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_batch_acking() {
        let ctx = CancellationToken::new();
        let subscription = create_subscription(false).await;
        let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();
        let subscription_for_receive = subscription.clone();
        let ctx_for_receive = ctx.clone();
        let handle = tokio::spawn(async move {
            let _ = subscription_for_receive
                .receive(
                    move |message, _ctx| {
                        let sender = sender.clone();
                        async move {
                            let _ = sender.send(message.ack_id().to_string());
                        }
                    },
                    ctx_for_receive.clone(),
                    None,
                )
                .await;
        });

        let ctx_for_ack_manager = ctx.clone();
        let ack_manager = tokio::spawn(async move {
            let mut ack_ids = Vec::new();
            while !ctx_for_ack_manager.is_cancelled() {
                match tokio::time::timeout(Duration::from_secs(10), receiver.recv()).await {
                    Ok(ack_id) => {
                        if let Some(ack_id) = ack_id {
                            ack_ids.push(ack_id);
                            if ack_ids.len() > 10 {
                                subscription.ack(ack_ids).await.unwrap();
                                ack_ids = Vec::new();
                            }
                        }
                    }
                    Err(_e) => {
                        // timeout
                        subscription.ack(ack_ids).await.unwrap();
                        ack_ids = Vec::new();
                    }
                }
            }
            // flush
            subscription.ack(ack_ids).await
        });

        publish().await;
        tokio::time::sleep(Duration::from_secs(5)).await;

        ctx.cancel();
        let _ = handle.await;
        assert!(ack_manager.await.is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn test_snapshots() {
        let subscription = create_subscription(false).await;

        let snapshot_name = format!("snapshot-{}", rand::random::<u64>());
        let labels: HashMap<String, String> =
            HashMap::from_iter([("label-1".into(), "v1".into()), ("label-2".into(), "v2".into())]);
        let expected_fq_snap_name = format!("projects/{PROJECT_NAME}/snapshots/{snapshot_name}");

        // cleanup; TODO: remove?
        let _response = subscription.delete_snapshot(snapshot_name.as_str(), None).await;

        // create
        let created_snapshot = subscription
            .create_snapshot(snapshot_name.as_str(), labels.clone(), None)
            .await
            .unwrap();

        assert_eq!(created_snapshot.name, expected_fq_snap_name);
        // NOTE: we don't assert the labels due to lack of label support in the pubsub emulator.

        // get
        let retrieved_snapshot = subscription.get_snapshot(snapshot_name.as_str(), None).await.unwrap();
        assert_eq!(created_snapshot, retrieved_snapshot);

        // delete
        subscription
            .delete_snapshot(snapshot_name.as_str(), None)
            .await
            .unwrap();

        let _deleted_snapshot_status = subscription
            .get_snapshot(snapshot_name.as_str(), None)
            .await
            .expect_err("snapshot should have been deleted");

        let _delete_again = subscription
            .delete_snapshot(snapshot_name.as_str(), None)
            .await
            .expect_err("snapshot should already be deleted");
    }

    async fn ack_all(messages: &[ReceivedMessage]) {
        for message in messages.iter() {
            message.ack().await.unwrap();
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_seek_snapshot() {
        let subscription = create_subscription(false).await;
        let snapshot_name = format!("snapshot-{}", rand::random::<u64>());

        // publish and receive a message
        publish().await;
        let messages = subscription.pull(100, None).await.unwrap();
        ack_all(&messages).await;
        assert_eq!(messages.len(), 1);

        // snapshot at received = 1
        let _snapshot = subscription
            .create_snapshot(snapshot_name.as_str(), HashMap::new(), None)
            .await
            .unwrap();

        // publish and receive another message
        publish().await;
        let messages = subscription.pull(100, None).await.unwrap();
        assert_eq!(messages.len(), 1);
        ack_all(&messages).await;

        // rewind to snapshot at received = 1
        subscription
            .seek(SeekTo::Snapshot(snapshot_name.clone()), None)
            .await
            .unwrap();

        // assert we receive the 1 message we should receive again
        let messages = subscription.pull(100, None).await.unwrap();
        assert_eq!(messages.len(), 1);
        ack_all(&messages).await;

        // cleanup
        subscription
            .delete_snapshot(snapshot_name.as_str(), None)
            .await
            .unwrap();
        subscription.delete(None).await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_seek_timestamp() {
        let subscription = create_subscription(false).await;

        // enable acked message retention on subscription -- required for timestamp-based seeks
        subscription
            .update(
                SubscriptionConfigToUpdate {
                    retain_acked_messages: Some(true),
                    message_retention_duration: Some(Duration::new(60 * 60 * 2, 0)),
                    ..Default::default()
                },
                None,
            )
            .await
            .unwrap();

        // publish and receive a message
        publish().await;
        let messages = subscription.pull(100, None).await.unwrap();
        ack_all(&messages).await;
        assert_eq!(messages.len(), 1);

        let message_publish_time = messages.get(0).unwrap().message.publish_time.to_owned().unwrap();

        // rewind to a timestamp where message was just published
        subscription
            .seek(SeekTo::Timestamp(message_publish_time.to_owned().try_into().unwrap()), None)
            .await
            .unwrap();

        // consume -- should receive the first message again
        let messages = subscription.pull(100, None).await.unwrap();
        ack_all(&messages).await;
        assert_eq!(messages.len(), 1);
        let seek_message_publish_time = messages.get(0).unwrap().message.publish_time.to_owned().unwrap();
        assert_eq!(seek_message_publish_time, message_publish_time);

        // cleanup
        subscription.delete(None).await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_subscribe() {
        let subscription = create_subscription(false).await;
        let received = Arc::new(Mutex::new(false));
        let checking = received.clone();
        let _handler = tokio::spawn(async move {
            let mut iter = subscription.subscribe(None).await.unwrap();
            while let Some(message) = iter.next().await {
                *received.lock().unwrap() = true;
                let _ = message.ack().await;
            }
        });
        publish().await;
        tokio::time::sleep(Duration::from_secs(5)).await;
        assert!(*checking.lock().unwrap());
    }
}
