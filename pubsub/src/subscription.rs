use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, SystemTime};

use prost_types::{DurationError, FieldMask};
use tokio_util::sync::CancellationToken;

use google_cloud_gax::grpc::codegen::tokio_stream::Stream;
use google_cloud_gax::grpc::{Code, Status};
use google_cloud_gax::retry::RetrySetting;
use google_cloud_googleapis::pubsub::v1::seek_request::Target;
use google_cloud_googleapis::pubsub::v1::subscription::AnalyticsHubSubscriptionInfo;
use google_cloud_googleapis::pubsub::v1::{
    BigQueryConfig, CloudStorageConfig, CreateSnapshotRequest, DeadLetterPolicy, DeleteSnapshotRequest,
    DeleteSubscriptionRequest, ExpirationPolicy, GetSnapshotRequest, GetSubscriptionRequest, MessageTransform,
    PullRequest, PushConfig, RetryPolicy, SeekRequest, Snapshot, Subscription as InternalSubscription,
    UpdateSubscriptionRequest,
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
    pub cloud_storage_config: Option<CloudStorageConfig>,
    pub analytics_hub_subscription_info: Option<AnalyticsHubSubscriptionInfo>,
    pub message_transforms: Vec<MessageTransform>,
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
            cloud_storage_config: f.cloud_storage_config,
            analytics_hub_subscription_info: f.analytics_hub_subscription_info,
            message_transforms: f.message_transforms,
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

#[derive(Debug, Clone, Default)]
pub struct SubscribeConfig {
    enable_multiple_subscriber: bool,
    channel_capacity: Option<usize>,
    subscriber_config: Option<SubscriberConfig>,
}

impl SubscribeConfig {
    pub fn with_enable_multiple_subscriber(mut self, v: bool) -> Self {
        self.enable_multiple_subscriber = v;
        self
    }
    pub fn with_subscriber_config(mut self, v: SubscriberConfig) -> Self {
        self.subscriber_config = Some(v);
        self
    }
    pub fn with_channel_capacity(mut self, v: usize) -> Self {
        self.channel_capacity = Some(v);
        self
    }
}

#[derive(Debug, Clone)]
pub struct ReceiveConfig {
    pub worker_count: usize,
    pub channel_capacity: Option<usize>,
    pub subscriber_config: Option<SubscriberConfig>,
}

impl Default for ReceiveConfig {
    fn default() -> Self {
        Self {
            worker_count: 10,
            subscriber_config: None,
            channel_capacity: None,
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
    tasks: Vec<Subscriber>,
}

impl MessageStream {
    pub fn cancellable(&self) -> CancellationToken {
        self.cancel.clone()
    }

    pub async fn dispose(&mut self) {
        // Close streaming pull task
        if !self.cancel.is_cancelled() {
            self.cancel.cancel();
        }

        // Wait for all the streaming pull close.
        for task in &mut self.tasks {
            task.done().await;
        }

        // Nack for remaining messages.
        while let Ok(message) = self.queue.recv().await {
            if let Err(err) = message.nack().await {
                tracing::warn!("failed to nack message messageId={} {:?}", message.message.message_id, err);
            }
        }
    }

    /// Immediately Nack on cancel
    pub async fn read(&mut self) -> Option<ReceivedMessage> {
        let message = tokio::select! {
            msg = self.queue.recv() => msg.ok(),
            _ = self.cancel.cancelled() => None
        };
        if message.is_none() {
            self.dispose().await;
        }
        message
    }
}

impl Drop for MessageStream {
    fn drop(&mut self) {
        if !self.queue.is_empty() {
            tracing::warn!("Call 'dispose' before drop in order to call nack for remaining messages");
        }
        if !self.cancel.is_cancelled() {
            self.cancel.cancel();
        }
    }
}

impl Stream for MessageStream {
    type Item = ReceivedMessage;

    /// Return None unless the queue is open.
    /// Use CancellationToken for SubscribeConfig to get None
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

    pub(crate) fn streaming_pool_size(&self) -> usize {
        self.subc.streaming_pool_size()
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

    pub fn get_client(&self) -> SubscriberClient {
        self.subc.clone()
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
                    cloud_storage_config: cfg.cloud_storage_config,
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
                    analytics_hub_subscription_info: cfg.analytics_hub_subscription_info,
                    message_transforms: cfg.message_transforms,
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
            .map(|m| {
                ReceivedMessage::new(
                    self.fqsn.clone(),
                    self.subc.clone(),
                    m.message.unwrap(),
                    m.ack_id,
                    (m.delivery_attempt > 0).then_some(m.delivery_attempt as usize),
                )
            })
            .collect())
    }

    /// subscribe creates a `Stream` of `ReceivedMessage`
    /// ```
    /// use google_cloud_pubsub::subscription::{SubscribeConfig, Subscription};
    /// use tokio::select;
    /// use google_cloud_gax::grpc::Status;
    ///
    /// async fn run(subscription: Subscription) -> Result<(), Status> {
    ///     let mut iter = subscription.subscribe(None).await?;
    ///     let ctx = iter.cancellable();
    ///     let handler = tokio::spawn(async move {
    ///         while let Some(message) = iter.read().await {
    ///             let _ = message.ack().await;
    ///         }
    ///     });
    ///     // Cancel and wait for nack all the pulled messages.
    ///     ctx.cancel();
    ///     let _ = handler.await;
    ///     Ok(())
    ///  }
    /// ```
    ///
    /// ```
    /// use google_cloud_pubsub::subscription::{SubscribeConfig, Subscription};
    /// use futures_util::StreamExt;
    /// use tokio::select;
    /// use google_cloud_gax::grpc::Status;
    ///
    /// async fn run(subscription: Subscription) -> Result<(), Status> {
    ///     let mut iter = subscription.subscribe(None).await?;
    ///     let ctx = iter.cancellable();
    ///     let handler = tokio::spawn(async move {
    ///         while let Some(message) = iter.next().await {
    ///             let _ = message.ack().await;
    ///         }
    ///     });
    ///     // Cancel and wait for receive all the pulled messages.
    ///     ctx.cancel();
    ///     let _ = handler.await;
    ///     Ok(())
    ///  }
    /// ```
    pub async fn subscribe(&self, opt: Option<SubscribeConfig>) -> Result<MessageStream, Status> {
        let opt = opt.unwrap_or_default();
        let (tx, rx) = create_channel(opt.channel_capacity);
        let cancel = CancellationToken::new();
        let sub_opt = self.unwrap_subscribe_config(opt.subscriber_config).await?;

        // spawn a separate subscriber task for each connection in the pool
        let subscribers = if opt.enable_multiple_subscriber {
            self.streaming_pool_size()
        } else {
            1
        };
        let mut tasks = Vec::with_capacity(subscribers);
        for _ in 0..subscribers {
            tasks.push(Subscriber::start(
                cancel.clone(),
                self.fqsn.clone(),
                self.subc.clone(),
                tx.clone(),
                sub_opt.clone(),
            ));
        }

        Ok(MessageStream {
            queue: rx,
            cancel,
            tasks,
        })
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
        let sub_opt = self.unwrap_subscribe_config(op.subscriber_config).await?;

        if self
            .config(sub_opt.retry_setting.clone())
            .await?
            .1
            .enable_message_ordering
        {
            (0..op.worker_count).for_each(|_v| {
                let (sender, receiver) = create_channel(op.channel_capacity);
                receivers.push(receiver);
                senders.push(sender);
            });
        } else {
            let (sender, receiver) = create_channel(op.channel_capacity);
            (0..op.worker_count).for_each(|_v| {
                receivers.push(receiver.clone());
                senders.push(sender.clone());
            });
        }

        //same ordering key is in same stream.
        let subscribers: Vec<Subscriber> = senders
            .into_iter()
            .map(|queue| {
                Subscriber::start(cancel.clone(), self.fqsn.clone(), self.subc.clone(), queue, sub_opt.clone())
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

        // wait for all the threads finish.
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
    ///   Snapshots have a finite lifetime -- a maximum of 7 days from the time of creation, beyond which
    ///   they are discarded and any messages being retained solely due to the snapshot dropped.
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

    async fn unwrap_subscribe_config(&self, cfg: Option<SubscriberConfig>) -> Result<SubscriberConfig, Status> {
        if let Some(cfg) = cfg {
            return Ok(cfg);
        }
        let cfg = self.config(None).await?;
        let mut default_cfg = SubscriberConfig {
            stream_ack_deadline_seconds: cfg.1.ack_deadline_seconds.clamp(10, 600),
            ..Default::default()
        };
        if cfg.1.enable_exactly_once_delivery {
            default_cfg.max_outstanding_messages = 5;
        }
        Ok(default_cfg)
    }
}

fn create_channel(
    channel_capacity: Option<usize>,
) -> (async_channel::Sender<ReceivedMessage>, async_channel::Receiver<ReceivedMessage>) {
    match channel_capacity {
        None => async_channel::unbounded(),
        Some(cap) => async_channel::bounded(cap),
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

    use google_cloud_gax::conn::{ConnectionOptions, Environment};
    use google_cloud_googleapis::pubsub::v1::{PublishRequest, PubsubMessage};

    use crate::apiv1::conn_pool::ConnectionManager;
    use crate::apiv1::publisher_client::PublisherClient;
    use crate::apiv1::subscriber_client::SubscriberClient;
    use crate::subscriber::ReceivedMessage;
    use crate::subscription::{
        ReceiveConfig, SeekTo, SubscribeConfig, Subscription, SubscriptionConfig, SubscriptionConfigToUpdate,
    };

    const PROJECT_NAME: &str = "local-project";
    const EMULATOR: &str = "localhost:8681";

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt().try_init();
    }

    async fn create_subscription(enable_exactly_once_delivery: bool) -> Subscription {
        let cm = ConnectionManager::new(
            4,
            "",
            &Environment::Emulator(EMULATOR.to_string()),
            &ConnectionOptions::default(),
        )
        .await
        .unwrap();
        let cm2 = ConnectionManager::new(
            4,
            "",
            &Environment::Emulator(EMULATOR.to_string()),
            &ConnectionOptions::default(),
        )
        .await
        .unwrap();
        let client = SubscriberClient::new(cm, cm2);

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

    async fn publish(messages: Option<Vec<PubsubMessage>>) {
        let pubc = PublisherClient::new(
            ConnectionManager::new(
                4,
                "",
                &Environment::Emulator(EMULATOR.to_string()),
                &ConnectionOptions::default(),
            )
            .await
            .unwrap(),
        );
        let messages = messages.unwrap_or(vec![PubsubMessage {
            data: "test_message".into(),
            ..Default::default()
        }]);
        let req = PublishRequest {
            topic: format!("projects/{PROJECT_NAME}/topics/test-topic1"),
            messages,
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
        let base = PubsubMessage {
            data: "test_message".into(),
            ..Default::default()
        };
        publish(Some(vec![base.clone(), base.clone(), base])).await;
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
    async fn test_multi_subscriber_single_subscription_unbound() {
        test_multi_subscriber_single_subscription(None).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_multi_subscriber_single_subscription_bound() {
        let opt = Some(ReceiveConfig {
            channel_capacity: Some(1),
            ..Default::default()
        });
        test_multi_subscriber_single_subscription(opt).await;
    }

    async fn test_multi_subscriber_single_subscription(opt: Option<ReceiveConfig>) {
        let msg = PubsubMessage {
            data: "test".into(),
            ..Default::default()
        };
        let msg_size = 10;
        let msgs: Vec<PubsubMessage> = (0..msg_size).map(|_v| msg.clone()).collect();
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
                            tracing::info!("received {}", message.message.message_id);
                            v2.fetch_add(1, SeqCst);
                            let _ = message.ack().await;
                        }
                    },
                    cancel_receiver,
                    opt,
                )
                .await;
        });
        publish(Some(msgs)).await;
        tokio::time::sleep(Duration::from_secs(5)).await;
        cancellation_token.cancel();
        let _ = handle.await;
        assert_eq!(v.load(SeqCst), msg_size);
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

        publish(None).await;
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

        publish(None).await;
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
        publish(None).await;
        let messages = subscription.pull(100, None).await.unwrap();
        ack_all(&messages).await;
        assert_eq!(messages.len(), 1);

        // snapshot at received = 1
        let _snapshot = subscription
            .create_snapshot(snapshot_name.as_str(), HashMap::new(), None)
            .await
            .unwrap();

        // publish and receive another message
        publish(None).await;
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
        publish(None).await;
        let messages = subscription.pull(100, None).await.unwrap();
        ack_all(&messages).await;
        assert_eq!(messages.len(), 1);

        let message_publish_time = messages.first().unwrap().message.publish_time.to_owned().unwrap();

        // rewind to a timestamp where message was just published
        subscription
            .seek(SeekTo::Timestamp(message_publish_time.to_owned().try_into().unwrap()), None)
            .await
            .unwrap();

        // consume -- should receive the first message again
        let messages = subscription.pull(100, None).await.unwrap();
        ack_all(&messages).await;
        assert_eq!(messages.len(), 1);
        let seek_message_publish_time = messages.first().unwrap().message.publish_time.to_owned().unwrap();
        assert_eq!(seek_message_publish_time, message_publish_time);

        // cleanup
        subscription.delete(None).await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_subscribe_single_subscriber() {
        test_subscribe(None).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_subscribe_multiple_subscriber() {
        test_subscribe(Some(SubscribeConfig::default().with_enable_multiple_subscriber(true))).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_subscribe_multiple_subscriber_bound() {
        test_subscribe(Some(
            SubscribeConfig::default()
                .with_enable_multiple_subscriber(true)
                .with_channel_capacity(1),
        ))
        .await;
    }

    async fn test_subscribe(opt: Option<SubscribeConfig>) {
        let msg = PubsubMessage {
            data: "test".into(),
            ..Default::default()
        };
        let msg_count = 10;
        let msg: Vec<PubsubMessage> = (0..msg_count).map(|_v| msg.clone()).collect();
        let subscription = create_subscription(false).await;
        let received = Arc::new(Mutex::new(0));
        let checking = received.clone();
        let mut iter = subscription.subscribe(opt).await.unwrap();
        let cancellable = iter.cancellable();
        let handler = tokio::spawn(async move {
            while let Some(message) = iter.next().await {
                tracing::info!("received {}", message.message.message_id);
                *received.lock().unwrap() += 1;
                tokio::time::sleep(Duration::from_millis(500)).await;
                let _ = message.ack().await;
            }
        });
        publish(Some(msg)).await;
        tokio::time::sleep(Duration::from_secs(8)).await;
        cancellable.cancel();
        let _ = handler.await;
        assert_eq!(*checking.lock().unwrap(), msg_count);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_subscribe_nack_on_cancel_read() {
        subscribe_nack_on_cancel_read(10, true).await;
        subscribe_nack_on_cancel_read(0, true).await;
        subscribe_nack_on_cancel_read(10, false).await;
        subscribe_nack_on_cancel_read(0, false).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_subscribe_nack_on_cancel_next() {
        // cancel after subscribe all message
        subscribe_nack_on_cancel_next(10, Duration::from_secs(3)).await;
        // cancel after process all message
        subscribe_nack_on_cancel_next(10, Duration::from_millis(0)).await;
        // no message
        subscribe_nack_on_cancel_next(0, Duration::from_secs(3)).await;
    }

    async fn subscribe_nack_on_cancel_read(msg_count: usize, should_cancel: bool) {
        let opt = Some(SubscribeConfig::default().with_enable_multiple_subscriber(true));

        let msg = PubsubMessage {
            data: "test".into(),
            ..Default::default()
        };
        let msg: Vec<PubsubMessage> = (0..msg_count).map(|_v| msg.clone()).collect();
        let subscription = create_subscription(false).await;
        let received = Arc::new(Mutex::new(0));
        let checking = received.clone();

        let mut iter = subscription.subscribe(opt).await.unwrap();
        let ctx = iter.cancellable();
        let handler = tokio::spawn(async move {
            while let Some(message) = iter.read().await {
                tracing::info!("received {}", message.message.message_id);
                *received.lock().unwrap() += 1;
                if should_cancel {
                    // expect cancel
                    tokio::time::sleep(Duration::from_secs(10)).await;
                } else {
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
                let _ = message.ack().await;
            }
        });
        publish(Some(msg)).await;
        tokio::time::sleep(Duration::from_secs(10)).await;
        ctx.cancel();
        handler.await.unwrap();
        if should_cancel && msg_count > 0 {
            // expect nack
            assert!(*checking.lock().unwrap() < msg_count);
        } else {
            // all delivered
            assert_eq!(*checking.lock().unwrap(), msg_count);
        }
    }

    async fn subscribe_nack_on_cancel_next(msg_count: usize, recv_time: Duration) {
        let opt = Some(SubscribeConfig::default().with_enable_multiple_subscriber(true));

        let msg = PubsubMessage {
            data: "test".into(),
            ..Default::default()
        };
        let msg: Vec<PubsubMessage> = (0..msg_count).map(|_v| msg.clone()).collect();
        let subscription = create_subscription(false).await;
        let received = Arc::new(Mutex::new(0));
        let checking = received.clone();

        let mut iter = subscription.subscribe(opt).await.unwrap();
        let ctx = iter.cancellable();
        let handler = tokio::spawn(async move {
            while let Some(message) = iter.next().await {
                tracing::info!("received {}", message.message.message_id);
                *received.lock().unwrap() += 1;
                tokio::time::sleep(recv_time).await;
                let _ = message.ack().await;
            }
        });
        publish(Some(msg)).await;
        tokio::time::sleep(Duration::from_secs(10)).await;
        ctx.cancel();
        handler.await.unwrap();
        assert_eq!(*checking.lock().unwrap(), msg_count);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_message_stream_dispose() {
        let subscription = create_subscription(false).await;
        let mut iter = subscription.subscribe(None).await.unwrap();
        iter.dispose().await;
        // no effect
        iter.dispose().await;
        assert!(iter.next().await.is_none());
    }
}
