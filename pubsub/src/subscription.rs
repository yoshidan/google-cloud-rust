use std::collections::HashMap;
use std::ops::DerefMut;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, SystemTime};

use prost_types::{DurationError, FieldMask};

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
use crate::subscriber::{ack, ReceivedMessage, Receiver, Subscriber, SubscriberConfig};

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
    buffer: Receiver,
    tasks: Vec<Subscriber>,
}

impl MessageStream {
    pub async fn dispose(self) -> usize {
        // dispose buffer
        let mut unprocessed = self.buffer.dispose().await;
        tracing::debug!("unprocessed messages in the buffer: {}", unprocessed);

        // stop all the subscribers
        for task in self.tasks {
            let nacked = task.dispose().await;
            tracing::debug!("unprocessed messages in the subscriber: {}", nacked);
            unprocessed += nacked;
        }
        unprocessed
    }
}

impl Stream for MessageStream {
    type Item = ReceivedMessage;

    // return None when all the subscribers are stopped and the queue is empty.
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(self.buffer.deref_mut()).poll_next(cx)
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

    /// pull pulls messages from the server.
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

    /// Subscribes to a Pub/Sub subscription and creates a `MessageStream` for consuming messages.
    ///
    /// This method initializes a message stream by setting up the necessary channel and spawning
    /// subscriber tasks based on the provided configuration. It supports multiple subscribers and
    /// configurable channel capacity.
    ///
    /// # Arguments
    /// - `opt`: An optional `SubscribeConfig` that specifies the subscription configuration, such as
    ///   enabling multiple subscribers, setting channel capacity, or providing a custom `SubscriberConfig`.
    ///
    /// # Returns
    /// - `Ok(MessageStream)`: A stream of `ReceivedMessage` objects for consuming messages.
    /// - `Err(Status)`: An error if the subscription configuration or setup fails.
    ///
    /// # Behavior
    /// - If `enable_multiple_subscriber` is set to `true` in the `SubscribeConfig`, multiple subscriber
    ///   tasks are spawned based on the streaming pool size.
    /// - If `channel_capacity` is specified, the channel is bounded; otherwise, it is unbounded.
    ///
    /// ```
    /// use google_cloud_gax::grpc::Status;
    /// use google_cloud_pubsub::subscription::{SubscribeConfig, Subscription};
    /// use futures_util::StreamExt;
    /// use tokio::select;
    /// use tokio_util::sync::CancellationToken;
    ///
    /// async fn run(ctx: CancellationToken, subscription: Subscription) -> Result<(), Status> {
    ///     // Start receiving messages from the subscription.
    ///     let mut iter = subscription.subscribe(None).await?;
    ///     // Get buffered messages.
    ///     // To close safely, use a CancellationToken or to signal shutdown.
    ///     while let Some(message) = tokio::select!{
    ///         v = iter.next() => v,
    ///         _ = ctx.cancelled() => None,
    ///     }.await {
    ///         let _ = message.ack().await;
    ///     }
    ///     // Wait for all the unprocessed messages to be Nack.
    ///     // If you don't call dispose, the unprocessed messages will be Nacke when the iterator is dropped.
    ///     iter.dispose().await;
    ///     Ok(())
    ///  }
    /// ```
    pub async fn subscribe(&self, opt: Option<SubscribeConfig>) -> Result<MessageStream, Status> {
        let opt = opt.unwrap_or_default();
        let (tx, rx) = match opt.channel_capacity {
            None => async_channel::unbounded(),
            Some(cap) => async_channel::bounded(cap),
        };
        let sub_opt = self.unwrap_subscribe_config(opt.subscriber_config).await?;

        // spawn a separate subscriber task for each connection in the pool
        let subscribers = if opt.enable_multiple_subscriber {
            self.streaming_pool_size()
        } else {
            1
        };
        let mut tasks = Vec::with_capacity(subscribers);
        for _ in 0..subscribers {
            tasks.push(Subscriber::spawn(
                self.fqsn.clone(),
                self.subc.clone(),
                tx.clone(),
                sub_opt.clone(),
            ));
        }

        Ok(MessageStream {
            buffer: Receiver::new(rx),
            tasks,
        })
    }

    /// Ack acknowledges the messages associated with the ack_ids in the AcknowledgeRequest.
    /// The Pub/Sub system can remove the relevant messages from the subscription.
    /// This method is for batch ack.
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

#[cfg(test)]
#[allow(deprecated)]
mod tests {

    use std::collections::HashMap;

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
    use crate::subscription::{SeekTo, SubscribeConfig, Subscription, SubscriptionConfig, SubscriptionConfigToUpdate};
    use crate::topic::Topic;

    const PROJECT_NAME: &str = "local-project";
    const EMULATOR: &str = "localhost:8681";

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_pull() {
        let (subscription, topic) = create_subscription(false, false).await;
        let base = PubsubMessage {
            data: "test_message".into(),
            ..Default::default()
        };
        publish(&topic, Some(vec![base.clone(), base.clone(), base])).await;
        let messages = subscription.pull(2, None).await.unwrap();
        assert_eq!(messages.len(), 2);
        for m in messages {
            m.ack().await.unwrap();
        }
        subscription.delete(None).await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_batch_ack() {
        let ctx = CancellationToken::new();
        let (subscription, topic) = create_subscription(false, false).await;
        let (sender, receiver) = async_channel::unbounded();
        let subscription_for_receive = subscription.clone();
        let ctx_for_subscribe = ctx.clone();

        let subscriber = tokio::spawn(async move {
            let mut stream = subscription_for_receive.subscribe(None).await.unwrap();
            while let Some(message) = tokio::select! {
                v = stream.next() => v,
                _ = ctx_for_subscribe.cancelled() => None,
            } {
                let _ = sender.send(message.ack_id().to_string()).await;
            }
            stream.dispose().await;
            tracing::info!("finish subscriber task");
        });

        let ack_manager = tokio::spawn(async move {
            let mut ack_ids = Vec::new();
            while let Ok(ack_id) = receiver.recv().await {
                tracing::info!("received ack_id: {}", ack_id);
                ack_ids.push(ack_id);
            }
            assert!(!ack_ids.is_empty());
            let _ = subscription.ack(ack_ids).await;
            tracing::info!("finish ack manager task");
        });

        let msg = PubsubMessage {
            data: "test".into(),
            ..Default::default()
        };
        let msg: Vec<PubsubMessage> = (0..10).map(|_v| msg.clone()).collect();
        publish(&topic, Some(msg)).await;
        tokio::time::sleep(Duration::from_secs(10)).await;
        ctx.cancel();

        assert!(subscriber.await.is_ok());
        assert!(ack_manager.await.is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn test_snapshots() {
        let (subscription, _topic) = create_subscription(false, false).await;

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

    #[tokio::test]
    #[serial]
    async fn test_seek_snapshot() {
        let (subscription, topic) = create_subscription(false, false).await;
        let snapshot_name = format!("snapshot-{}", rand::random::<u64>());

        // publish and receive a message
        publish(&topic, None).await;
        let messages = subscription.pull(100, None).await.unwrap();
        ack_all(&messages).await;
        assert_eq!(messages.len(), 1);

        // snapshot at received = 1
        let _snapshot = subscription
            .create_snapshot(snapshot_name.as_str(), HashMap::new(), None)
            .await
            .unwrap();

        // publish and receive another message
        publish(&topic, None).await;
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
        let (subscription, topic) = create_subscription(false, false).await;

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
        publish(&topic, None).await;
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
    async fn test_subscribe_pattern() {
        // default
        let opt = Some(SubscribeConfig::default());
        test_subscribe(opt.clone(), true, true, 10, 11).await;
        test_subscribe(opt.clone(), false, true, 10, 11).await;
        test_subscribe(opt.clone(), true, false, 10, 10).await;
        test_subscribe(opt.clone(), false, false, 10, 10).await;
        test_subscribe(opt.clone(), true, true, 10, 5).await;
        test_subscribe(opt.clone(), false, true, 10, 5).await;
        test_subscribe(opt.clone(), true, false, 10, 1).await;
        test_subscribe(opt.clone(), false, false, 10, 1).await;
        test_subscribe(opt.clone(), true, true, 0, 0).await;
        test_subscribe(opt.clone(), false, true, 0, 0).await;

        // with multiple subscribers
        let opt = Some(SubscribeConfig::default().with_enable_multiple_subscriber(true));
        test_subscribe(opt.clone(), true, false, 10, 11).await;
        test_subscribe(opt.clone(), false, false, 10, 11).await;
        test_subscribe(opt.clone(), true, true, 10, 10).await;
        test_subscribe(opt.clone(), false, true, 10, 10).await;
        test_subscribe(opt.clone(), true, false, 10, 5).await;
        test_subscribe(opt.clone(), false, false, 10, 5).await;
        test_subscribe(opt.clone(), true, true, 10, 1).await;
        test_subscribe(opt.clone(), false, true, 10, 1).await;
        test_subscribe(opt.clone(), true, false, 0, 0).await;
        test_subscribe(opt.clone(), false, false, 0, 0).await;

        // with multiple subscribers and channel capacity
        let opt = Some(
            SubscribeConfig::default()
                .with_enable_multiple_subscriber(true)
                .with_channel_capacity(1),
        );
        test_subscribe(opt.clone(), true, true, 10, 11).await;
        test_subscribe(opt.clone(), false, true, 10, 11).await;
        test_subscribe(opt.clone(), true, false, 10, 10).await;
        test_subscribe(opt.clone(), false, false, 10, 10).await;
        test_subscribe(opt.clone(), true, true, 10, 5).await;
        test_subscribe(opt.clone(), false, true, 10, 5).await;
        test_subscribe(opt.clone(), true, false, 10, 1).await;
        test_subscribe(opt.clone(), false, false, 10, 1).await;
        test_subscribe(opt.clone(), true, true, 0, 0).await;
        test_subscribe(opt.clone(), false, true, 0, 0).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_subscribe_forget() {
        let (subscription, topic) = create_subscription(false, false).await;

        // for all nack
        let iter = subscription.subscribe(None).await.unwrap();

        let msg = PubsubMessage {
            data: "test".into(),
            ordering_key: "order1".to_string(),
            ..Default::default()
        };
        let msg: Vec<PubsubMessage> = (0..10).map(|_v| msg.clone()).collect();
        publish(&topic, Some(msg)).await;
        tokio::time::sleep(Duration::from_secs(5)).await;

        // spawn nack task
        drop(iter);
        tokio::time::sleep(Duration::from_secs(3)).await;

        // ensure all the messages should be redelivered
        let ctx = CancellationToken::new();
        let ctx_for_sub = ctx.clone();
        let subscriber = tokio::spawn(async move {
            let mut acked = 0;
            let mut iter = subscription.subscribe(None).await.unwrap();
            let task = async {
                while let Some(message) = iter.next().await {
                    let _ = message.ack().await;
                    tracing::info!("acked {}", message.message.message_id);
                    acked += 1;
                }
            };
            tokio::select! {
                _ = task => {},
                _ = ctx_for_sub.cancelled() => {}
            }
            let nack_msgs = iter.dispose().await;
            assert_eq!(nack_msgs, 0);
            tracing::info!("disposed");
            acked
        });

        tokio::time::sleep(Duration::from_secs(10)).await;
        ctx.cancel();
        let acked = subscriber.await.unwrap();
        assert_eq!(acked, 10);
    }

    async fn test_subscribe(
        opt: Option<SubscribeConfig>,
        enable_exactly_once_delivery: bool,
        enable_message_ordering: bool,
        msg_count: usize,
        limit: usize,
    ) {
        tracing::info!(
            "test_subscribe: exactly_once_delivery={} msg_count={} limit={}",
            enable_exactly_once_delivery,
            msg_count,
            limit
        );
        let (subscription, topic) = create_subscription(enable_exactly_once_delivery, enable_message_ordering).await;

        let ctx = CancellationToken::new();
        let ctx_for_pub = ctx.clone();

        // publish messages
        let publisher = tokio::spawn(async move {
            let msg = PubsubMessage {
                data: "test".into(),
                ordering_key: "order1".to_string(),
                ..Default::default()
            };
            let msg: Vec<PubsubMessage> = (0..msg_count).map(|_v| msg.clone()).collect();
            publish(&topic, Some(msg)).await;
            tokio::time::sleep(Duration::from_secs(10)).await;
            ctx_for_pub.cancel();
        });

        // subscribe and ack messages
        let mut acked = 0;
        let mut iter = subscription.subscribe(opt).await.unwrap();
        while let Some(message) = {
            tokio::select! {
                v = iter.next() => v,
                _ = ctx.cancelled() => None
            }
        } {
            let _ = message.ack().await;
            tracing::info!("acked {}", message.message.message_id);
            acked += 1;
            if acked >= limit {
                // should nack rest of messages
                break;
            }
        }
        let nack_msgs = iter.dispose().await;
        assert_eq!(nack_msgs, msg_count - limit.min(msg_count));

        publisher.await.unwrap();
        tracing::info!("disposed");

        if limit > msg_count {
            assert_eq!(acked, msg_count);
        } else {
            assert_eq!(acked, limit);
        }
    }

    async fn ack_all(messages: &[ReceivedMessage]) {
        for message in messages.iter() {
            message.ack().await.unwrap();
        }
    }

    async fn create_subscription(
        enable_exactly_once_delivery: bool,
        enable_message_ordering: bool,
    ) -> (Subscription, Topic) {
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
        let cm3 = ConnectionManager::new(
            4,
            "",
            &Environment::Emulator(EMULATOR.to_string()),
            &ConnectionOptions::default(),
        )
        .await
        .unwrap();
        let sub_client = SubscriberClient::new(cm, cm2);
        let pub_client = PublisherClient::new(cm3);
        let uuid = Uuid::new_v4().hyphenated().to_string();

        let topic_name = format!("projects/{}/topics/t{}", PROJECT_NAME, &uuid);
        let topic = Topic::new(topic_name.clone(), pub_client, sub_client.clone());
        topic.create(None, None).await.unwrap();

        let subscription_name = format!("projects/{}/subscriptions/s{}", PROJECT_NAME, &uuid);
        let subscription = Subscription::new(subscription_name, sub_client);
        let config = SubscriptionConfig {
            enable_exactly_once_delivery,
            enable_message_ordering,
            ..Default::default()
        };
        subscription.create(topic_name.as_str(), config, None).await.unwrap();
        (subscription, topic)
    }

    async fn publish(topic: &Topic, messages: Option<Vec<PubsubMessage>>) {
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
            topic: topic.fully_qualified_name().to_string(),
            messages,
        };
        let _ = pubc.publish(req, None).await;
    }
}
