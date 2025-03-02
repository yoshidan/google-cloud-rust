use std::env::var;

use google_cloud_gax::conn::{ConnectionOptions, Environment};
use google_cloud_gax::grpc::Status;
use google_cloud_gax::retry::RetrySetting;
use google_cloud_googleapis::pubsub::v1::{
    DetachSubscriptionRequest, ListSnapshotsRequest, ListSubscriptionsRequest, ListTopicsRequest, Snapshot,
};
use token_source::NoopTokenSourceProvider;

use crate::apiv1::conn_pool::{ConnectionManager, PUBSUB};
use crate::apiv1::publisher_client::PublisherClient;
use crate::apiv1::subscriber_client::SubscriberClient;
use crate::subscription::{Subscription, SubscriptionConfig};
use crate::topic::{Topic, TopicConfig};

#[derive(Debug)]
pub struct ClientConfig {
    /// gRPC channel pool size
    pub pool_size: Option<usize>,
    /// Pub/Sub project_id
    pub project_id: Option<String>,
    /// Runtime project info
    pub environment: Environment,
    /// Overriding service endpoint
    pub endpoint: String,
    /// gRPC connection option
    pub connection_option: ConnectionOptions,
}

/// ClientConfigs created by default will prefer to use `PUBSUB_EMULATOR_HOST`
impl Default for ClientConfig {
    fn default() -> Self {
        let emulator = var("PUBSUB_EMULATOR_HOST").ok();
        let default_project_id = emulator.as_ref().map(|_| "local-project".to_string());
        Self {
            pool_size: Some(4),
            environment: match emulator {
                Some(v) => Environment::Emulator(v),
                None => Environment::GoogleCloud(Box::new(NoopTokenSourceProvider {})),
            },
            project_id: default_project_id,
            endpoint: PUBSUB.to_string(),
            connection_option: ConnectionOptions::default(),
        }
    }
}

#[cfg(feature = "auth")]
pub use google_cloud_auth;

#[cfg(feature = "auth")]
impl ClientConfig {
    pub async fn with_auth(mut self) -> Result<Self, google_cloud_auth::error::Error> {
        if let Environment::GoogleCloud(_) = self.environment {
            let ts = google_cloud_auth::token::DefaultTokenSourceProvider::new(Self::auth_config()).await?;
            self.project_id = self.project_id.or(ts.project_id.clone());
            self.environment = Environment::GoogleCloud(Box::new(ts))
        }
        Ok(self)
    }

    pub async fn with_credentials(
        mut self,
        credentials: google_cloud_auth::credentials::CredentialsFile,
    ) -> Result<Self, google_cloud_auth::error::Error> {
        if let Environment::GoogleCloud(_) = self.environment {
            let ts = google_cloud_auth::token::DefaultTokenSourceProvider::new_with_credentials(
                Self::auth_config(),
                Box::new(credentials),
            )
            .await?;
            self.project_id = self.project_id.or(ts.project_id.clone());
            self.environment = Environment::GoogleCloud(Box::new(ts))
        }
        Ok(self)
    }

    fn auth_config() -> google_cloud_auth::project::Config<'static> {
        google_cloud_auth::project::Config::default()
            .with_audience(crate::apiv1::conn_pool::AUDIENCE)
            .with_scopes(&crate::apiv1::conn_pool::SCOPES)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    GAX(#[from] google_cloud_gax::conn::Error),
    #[error("Project ID was not found")]
    ProjectIdNotFound,
}

/// Client is a Google Pub/Sub client scoped to a single project.
///
/// Clients should be reused rather than being created as needed.
/// A Client may be shared by multiple tasks.
#[derive(Clone, Debug)]
pub struct Client {
    project_id: String,
    pubc: PublisherClient,
    subc: SubscriberClient,
}

impl Client {
    /// new creates a Pub/Sub client. See [`ClientConfig`] for more information.
    pub async fn new(config: ClientConfig) -> Result<Self, Error> {
        let pool_size = config.pool_size.unwrap_or_default();

        let pubc = PublisherClient::new(
            ConnectionManager::new(
                pool_size,
                config.endpoint.as_str(),
                &config.environment,
                &config.connection_option,
            )
            .await?,
        );
        let subc = SubscriberClient::new(
            ConnectionManager::new(
                pool_size,
                config.endpoint.as_str(),
                &config.environment,
                &config.connection_option,
            )
            .await?,
            ConnectionManager::new(
                pool_size,
                config.endpoint.as_str(),
                &config.environment,
                &config.connection_option,
            )
            .await?,
        );
        Ok(Self {
            project_id: config.project_id.ok_or(Error::ProjectIdNotFound)?,
            pubc,
            subc,
        })
    }

    /// create_subscription creates a new subscription on a topic.
    ///
    /// id is the name of the subscription to create. It must start with a letter,
    /// and contain only letters ([A-Za-z]), numbers ([0-9]), dashes (-),
    /// underscores (_), periods (.), tildes (~), plus (+) or percent signs (%). It
    /// must be between 3 and 255 characters in length, and must not start with
    /// "goog".
    ///
    /// cfg.ack_deadline is the maximum time after a subscriber receives a message before
    /// the subscriber should acknowledge the message. It must be between 10 and 600
    /// seconds (inclusive), and is rounded down to the nearest second. If the
    /// provided ackDeadline is 0, then the default value of 10 seconds is used.
    /// Note: messages which are obtained via Subscription.Receive need not be
    /// acknowledged within this deadline, as the deadline will be automatically
    /// extended.
    ///
    /// cfg.push_config may be set to configure this subscription for push delivery.
    ///
    /// If the subscription already exists an error will be returned.
    pub async fn create_subscription(
        &self,
        id: &str,
        topic_id: &str,
        cfg: SubscriptionConfig,
        retry: Option<RetrySetting>,
    ) -> Result<Subscription, Status> {
        let subscription = self.subscription(id);
        subscription
            .create(self.fully_qualified_topic_name(topic_id).as_str(), cfg, retry)
            .await
            .map(|_v| subscription)
    }

    /// subscriptions returns an iterator which returns all of the subscriptions for the client's project.
    pub async fn get_subscriptions(&self, retry: Option<RetrySetting>) -> Result<Vec<Subscription>, Status> {
        let req = ListSubscriptionsRequest {
            project: self.fully_qualified_project_name(),
            page_size: 0,
            page_token: "".to_string(),
        };
        self.subc.list_subscriptions(req, retry).await.map(|v| {
            v.into_iter()
                .map(|x| Subscription::new(x.name, self.subc.clone()))
                .collect()
        })
    }

    /// subscription creates a reference to a subscription.
    pub fn subscription(&self, id: &str) -> Subscription {
        Subscription::new(self.fully_qualified_subscription_name(id), self.subc.clone())
    }

    /// detach_subscription detaches a subscription from its topic. All messages
    /// retained in the subscription are dropped. Subsequent `Pull` and `StreamingPull`
    /// requests will return FAILED_PRECONDITION. If the subscription is a push
    /// subscription, pushes to the endpoint will stop.
    pub async fn detach_subscription(&self, fqsn: &str, retry: Option<RetrySetting>) -> Result<(), Status> {
        let req = DetachSubscriptionRequest {
            subscription: fqsn.to_string(),
        };
        self.pubc.detach_subscription(req, retry).await.map(|_v| ())
    }

    /// create_topic creates a new topic.
    ///
    /// The specified topic ID must start with a letter, and contain only letters
    /// ([A-Za-z]), numbers ([0-9]), dashes (-), underscores (_), periods (.),
    /// tildes (~), plus (+) or percent signs (%). It must be between 3 and 255
    /// characters in length, and must not start with "goog". For more information,
    /// see: https://cloud.google.com/pubsub/docs/admin#resource_names
    ///
    /// If the topic already exists an error will be returned.
    pub async fn create_topic(
        &self,
        id: &str,
        cfg: Option<TopicConfig>,
        retry: Option<RetrySetting>,
    ) -> Result<Topic, Status> {
        let topic = self.topic(id);
        topic.create(cfg, retry).await.map(|_v| topic)
    }

    /// topics returns an iterator which returns all of the topics for the client's project.
    pub async fn get_topics(&self, retry: Option<RetrySetting>) -> Result<Vec<String>, Status> {
        let req = ListTopicsRequest {
            project: self.fully_qualified_project_name(),
            page_size: 0,
            page_token: "".to_string(),
        };
        self.pubc
            .list_topics(req, retry)
            .await
            .map(|v| v.into_iter().map(|x| x.name).collect())
    }

    /// topic creates a reference to a topic in the client's project.
    ///
    /// If a Topic's Publish method is called, it has background tasks
    /// associated with it. Clean them up by calling topic.stop.
    ///
    /// Avoid creating many Topic instances if you use them to publish.
    pub fn topic(&self, id: &str) -> Topic {
        Topic::new(self.fully_qualified_topic_name(id), self.pubc.clone(), self.subc.clone())
    }

    /// get_snapshots lists the existing snapshots. Snapshots are used in Seek (at https://cloud.google.com/pubsub/docs/replay-overview) operations, which
    /// allow you to manage message acknowledgments in bulk. That is, you can set
    /// the acknowledgment state of messages in an existing subscription to the
    /// state captured by a snapshot.
    pub async fn get_snapshots(&self, retry: Option<RetrySetting>) -> Result<Vec<Snapshot>, Status> {
        let req = ListSnapshotsRequest {
            project: self.fully_qualified_project_name(),
            page_size: 0,
            page_token: "".to_string(),
        };
        self.subc.list_snapshots(req, retry).await
    }

    pub fn fully_qualified_topic_name(&self, id: &str) -> String {
        if id.contains('/') {
            id.to_string()
        } else {
            format!("projects/{}/topics/{}", self.project_id, id)
        }
    }

    pub fn fully_qualified_subscription_name(&self, id: &str) -> String {
        if id.contains('/') {
            id.to_string()
        } else {
            format!("projects/{}/subscriptions/{}", self.project_id, id)
        }
    }

    fn fully_qualified_project_name(&self) -> String {
        format!("projects/{}", self.project_id)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::thread;
    use std::time::Duration;

    use serial_test::serial;
    use tokio_util::sync::CancellationToken;
    use uuid::Uuid;

    use google_cloud_googleapis::pubsub::v1::PubsubMessage;

    use crate::client::Client;
    use crate::subscriber::SubscriberConfig;
    use crate::subscription::{ReceiveConfig, SubscriptionConfig};

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt().try_init();
    }

    async fn create_client() -> Client {
        std::env::set_var("PUBSUB_EMULATOR_HOST", "localhost:8681");

        Client::new(Default::default()).await.unwrap()
    }

    async fn do_publish_and_subscribe(ordering_key: &str, bulk: bool) {
        let client = create_client().await;

        let order = !ordering_key.is_empty();
        // create
        let uuid = Uuid::new_v4().hyphenated().to_string();
        let topic_id = &format!("t{}", &uuid);
        let subscription_id = &format!("s{}", &uuid);
        let topic = client.create_topic(topic_id.as_str(), None, None).await.unwrap();
        let publisher = topic.new_publisher(None);
        let config = SubscriptionConfig {
            enable_message_ordering: !ordering_key.is_empty(),
            ..Default::default()
        };
        let subscription = client
            .create_subscription(subscription_id.as_str(), topic_id.as_str(), config, None)
            .await
            .unwrap();

        let cancellation_token = CancellationToken::new();
        //subscribe
        let config = ReceiveConfig {
            worker_count: 2,
            channel_capacity: None,
            subscriber_config: Some(SubscriberConfig {
                ping_interval: Duration::from_secs(1),
                ..Default::default()
            }),
        };
        let cancel_receiver = cancellation_token.clone();
        let (s, mut r) = tokio::sync::mpsc::channel(100);
        let handle = tokio::spawn(async move {
            let _ = subscription
                .receive(
                    move |v, _ctx| {
                        let s2 = s.clone();
                        async move {
                            let _ = v.ack().await;
                            let data = std::str::from_utf8(&v.message.data).unwrap().to_string();
                            tracing::info!(
                                "tid={:?} id={} data={}",
                                thread::current().id(),
                                v.message.message_id,
                                data
                            );
                            let _ = s2.send(data).await;
                        }
                    },
                    cancel_receiver,
                    Some(config),
                )
                .await;
        });

        //publish
        let awaiters = if bulk {
            let messages = (0..100)
                .map(|key| PubsubMessage {
                    data: format!("abc_{key}").into(),
                    ordering_key: ordering_key.to_string(),
                    ..Default::default()
                })
                .collect();
            publisher.publish_bulk(messages).await
        } else {
            let mut awaiters = Vec::with_capacity(100);
            for key in 0..100 {
                let message = PubsubMessage {
                    data: format!("abc_{key}").into(),
                    ordering_key: ordering_key.into(),
                    ..Default::default()
                };
                awaiters.push(publisher.publish(message).await);
            }
            awaiters
        };
        for v in awaiters {
            tracing::info!("sent message_id = {}", v.get().await.unwrap());
        }

        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        cancellation_token.cancel();
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;

        let mut count = 0;
        while let Some(data) = r.recv().await {
            tracing::debug!("{}", data);
            if order {
                assert_eq!(format!("abc_{count}"), data);
            }
            count += 1;
        }
        assert_eq!(count, 100);
        let _ = handle.await;

        let mut publisher = publisher;
        publisher.shutdown().await;
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_publish_subscribe_ordered() {
        do_publish_and_subscribe("ordering", false).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_publish_subscribe_ordered_bulk() {
        do_publish_and_subscribe("ordering", true).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_publish_subscribe_random() {
        do_publish_and_subscribe("", false).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_publish_subscribe_random_bulk() {
        do_publish_and_subscribe("", true).await;
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_lifecycle() {
        let client = create_client().await;

        let uuid = Uuid::new_v4().hyphenated().to_string();
        let topic_id = &format!("t{}", &uuid);
        let subscription_id = &format!("s{}", &uuid);
        let snapshot_id = &format!("snap{}", &uuid);
        let topics = client.get_topics(None).await.unwrap();
        let subs = client.get_subscriptions(None).await.unwrap();
        let snapshots = client.get_snapshots(None).await.unwrap();
        let _topic = client.create_topic(topic_id.as_str(), None, None).await.unwrap();
        let subscription = client
            .create_subscription(subscription_id.as_str(), topic_id.as_str(), SubscriptionConfig::default(), None)
            .await
            .unwrap();

        let _ = subscription
            .create_snapshot(snapshot_id, HashMap::default(), None)
            .await
            .unwrap();

        let topics_after = client.get_topics(None).await.unwrap();
        let subs_after = client.get_subscriptions(None).await.unwrap();
        let snapshots_after = client.get_snapshots(None).await.unwrap();
        assert_eq!(1, topics_after.len() - topics.len());
        assert_eq!(1, subs_after.len() - subs.len());
        assert_eq!(1, snapshots_after.len() - snapshots.len());
    }
}

#[cfg(test)]
mod tests_in_gcp {
    use crate::client::{Client, ClientConfig};
    use crate::publisher::PublisherConfig;
    use google_cloud_gax::conn::Environment;
    use google_cloud_gax::grpc::codegen::tokio_stream::StreamExt;
    use google_cloud_googleapis::pubsub::v1::PubsubMessage;
    use serial_test::serial;
    use std::collections::HashMap;

    use std::time::Duration;
    use tokio::select;
    use tokio_util::sync::CancellationToken;

    fn make_msg(key: &str) -> PubsubMessage {
        PubsubMessage {
            data: if key.is_empty() {
                "empty".into()
            } else {
                key.to_string().into()
            },
            ordering_key: key.into(),
            ..Default::default()
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_with_auth() {
        let config = ClientConfig::default().with_auth().await.unwrap();
        if let Environment::Emulator(_) = config.environment {
            unreachable!()
        }
    }

    #[tokio::test]
    #[serial]
    #[ignore]
    async fn test_publish_ordering_in_gcp_flush_buffer() {
        let client = Client::new(ClientConfig::default().with_auth().await.unwrap())
            .await
            .unwrap();
        let topic = client.topic("test-topic2");
        let publisher = topic.new_publisher(Some(PublisherConfig {
            flush_interval: Duration::from_secs(3),
            workers: 3,
            ..Default::default()
        }));

        let mut awaiters = vec![];
        for key in ["", "key1", "key2", "key3", "key3"] {
            awaiters.push(publisher.publish(make_msg(key)).await);
        }
        for awaiter in awaiters.into_iter() {
            tracing::info!("msg id {}", awaiter.get().await.unwrap());
        }

        // check same key
        let mut awaiters = vec![];
        for key in ["", "key1", "key2", "key3", "key3"] {
            awaiters.push(publisher.publish(make_msg(key)).await);
        }
        for awaiter in awaiters.into_iter() {
            tracing::info!("msg id {}", awaiter.get().await.unwrap());
        }
    }

    #[tokio::test]
    #[serial]
    #[ignore]
    async fn test_publish_ordering_in_gcp_limit_exceed() {
        let client = Client::new(ClientConfig::default().with_auth().await.unwrap())
            .await
            .unwrap();
        let topic = client.topic("test-topic2");
        let publisher = topic.new_publisher(Some(PublisherConfig {
            flush_interval: Duration::from_secs(30),
            workers: 1,
            bundle_size: 8,
            ..Default::default()
        }));

        let mut awaiters = vec![];
        for key in ["", "key1", "key2", "key3", "key1", "key2", "key3", ""] {
            awaiters.push(publisher.publish(make_msg(key)).await);
        }
        for awaiter in awaiters.into_iter() {
            tracing::info!("msg id {}", awaiter.get().await.unwrap());
        }

        // check same key twice
        let mut awaiters = vec![];
        for key in ["", "key1", "key2", "key3", "key1", "key2", "key3", ""] {
            awaiters.push(publisher.publish(make_msg(key)).await);
        }
        for awaiter in awaiters.into_iter() {
            tracing::info!("msg id {}", awaiter.get().await.unwrap());
        }
    }

    #[tokio::test]
    #[serial]
    #[ignore]
    async fn test_publish_ordering_in_gcp_bulk() {
        let client = Client::new(ClientConfig::default().with_auth().await.unwrap())
            .await
            .unwrap();
        let topic = client.topic("test-topic2");
        let publisher = topic.new_publisher(Some(PublisherConfig {
            flush_interval: Duration::from_secs(30),
            workers: 2,
            bundle_size: 8,
            ..Default::default()
        }));

        let msgs = ["", "", "key1", "key1", "key2", "key2", "key3", "key3"]
            .map(make_msg)
            .to_vec();
        for awaiter in publisher.publish_bulk(msgs).await.into_iter() {
            tracing::info!("msg id {}", awaiter.get().await.unwrap());
        }

        // check same key twice
        let msgs = ["", "", "key1", "key1", "key2", "key2", "key3", "key3"]
            .map(make_msg)
            .to_vec();
        for awaiter in publisher.publish_bulk(msgs).await.into_iter() {
            tracing::info!("msg id {}", awaiter.get().await.unwrap());
        }
    }
    #[tokio::test]
    #[serial]
    #[ignore]
    async fn test_subscribe_exactly_once_delivery() {
        let client = Client::new(ClientConfig::default().with_auth().await.unwrap())
            .await
            .unwrap();

        // Check if the subscription is exactly_once_delivery
        let subscription = client.subscription("eod-test");
        let config = subscription.config(None).await.unwrap().1;
        assert!(config.enable_exactly_once_delivery);

        // publish message
        let ctx = CancellationToken::new();
        let ctx_pub = ctx.clone();
        let publisher = client.topic("eod-test").new_publisher(None);
        let pub_task = tokio::spawn(async move {
            tracing::info!("start publisher");
            loop {
                if ctx_pub.is_cancelled() {
                    tracing::info!("finish publisher");
                    return;
                }
                publisher
                    .publish_blocking(PubsubMessage {
                        data: "msg".into(),
                        ..Default::default()
                    })
                    .get()
                    .await
                    .unwrap();
            }
        });

        // subscribe message
        let ctx_sub = ctx.child_token();
        let sub_task = tokio::spawn(async move {
            tracing::info!("start subscriber");
            let mut stream = subscription.subscribe(None).await.unwrap();
            let mut msgs = HashMap::new();
            while let Some(message) = select! {
                msg = stream.next() => msg,
                _ = ctx_sub.cancelled() => None
            } {
                let msg_id = &message.message.message_id;
                // heavy task
                tokio::time::sleep(Duration::from_secs(1)).await;
                *msgs.entry(msg_id.clone()).or_insert(0) += 1;
                message.ack().await.unwrap();
            }
            stream.dispose().await;
            tracing::info!("finish subscriber");
            msgs
        });

        tokio::time::sleep(Duration::from_secs(60)).await;

        // check redelivered messages
        ctx.cancel();
        pub_task.await.unwrap();
        let received_msgs = sub_task.await.unwrap();
        assert!(!received_msgs.is_empty());

        tracing::info!("Number of received messages = {}", received_msgs.len());
        for (msg_id, count) in received_msgs {
            assert_eq!(count, 1, "msg_id = {msg_id}, count = {count}");
        }
    }
}
