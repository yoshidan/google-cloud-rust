use crate::apiv1::conn_pool::{ConnectionManager, PUBSUB};
use crate::apiv1::publisher_client::PublisherClient;
use crate::apiv1::subscriber_client::SubscriberClient;
use google_cloud_gax::cancel::CancellationToken;

use crate::subscription::{Subscription, SubscriptionConfig};
use crate::topic::{Topic, TopicConfig};
use google_cloud_gax::conn::Environment;
use google_cloud_gax::grpc::Status;
use google_cloud_gax::retry::RetrySetting;
use google_cloud_googleapis::pubsub::v1::{DetachSubscriptionRequest, ListSubscriptionsRequest, ListTopicsRequest};

#[derive(Debug)]
pub struct ClientConfig {
    pub pool_size: Option<usize>,
    /// The default project is determined by credentials.
    /// - If the GOOGLE_APPLICATION_CREDENTIALS is specified the project_id is from credentials.
    /// - If the server is running on CGP the project_id is from metadata server
    /// - If the PUBSUB_EMULATOR_HOST is specified the project_id is 'local-project'
    pub project_id: Option<String>,

    /// Overriding service endpoint
    pub endpoint: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            pool_size: Some(4),
            project_id: None,
            endpoint: PUBSUB.to_string(),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Auth(#[from] google_cloud_auth::error::Error),
    #[error(transparent)]
    GAX(#[from] google_cloud_gax::conn::Error),
    #[error("invalid project_id")]
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
    /// default creates a default Pub/Sub client.
    pub async fn default() -> Result<Self, Error> {
        Self::new(Default::default()).await
    }

    /// new creates a Pub/Sub client.
    pub async fn new(config: ClientConfig) -> Result<Self, Error> {
        let pool_size = config.pool_size.unwrap_or_default();
        let environment = match std::env::var("PUBSUB_EMULATOR_HOST") {
            Ok(host) => Environment::Emulator(host),
            Err(_) => Environment::GoogleCloud(google_cloud_auth::project().await?),
        };
        let pubc =
            PublisherClient::new(ConnectionManager::new(pool_size, &environment, config.endpoint.as_str()).await?);
        let subc =
            SubscriberClient::new(ConnectionManager::new(pool_size, &environment, config.endpoint.as_str()).await?);

        let project_id = match config.project_id {
            Some(project_id) => project_id,
            None => match environment {
                Environment::GoogleCloud(project) => project.project_id().ok_or(Error::ProjectIdNotFound)?.to_string(),
                Environment::Emulator(_) => "local-project".to_string(),
            },
        };
        Ok(Self { project_id, pubc, subc })
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
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Subscription, Status> {
        let subscription = self.subscription(id);
        subscription
            .create(self.fully_qualified_topic_name(topic_id).as_str(), cfg, cancel, retry)
            .await
            .map(|_v| subscription)
    }

    /// subscriptions returns an iterator which returns all of the subscriptions for the client's project.
    pub async fn get_subscriptions(
        &self,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Vec<Subscription>, Status> {
        let req = ListSubscriptionsRequest {
            project: self.fully_qualified_project_name(),
            page_size: 0,
            page_token: "".to_string(),
        };
        self.subc.list_subscriptions(req, cancel, retry).await.map(|v| {
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
    pub async fn detach_subscription(
        &self,
        fqsn: &str,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<(), Status> {
        let req = DetachSubscriptionRequest {
            subscription: fqsn.to_string(),
        };
        self.pubc.detach_subscription(req, cancel, retry).await.map(|_v| ())
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
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Topic, Status> {
        let topic = self.topic(id);
        topic.create(cfg, cancel, retry).await.map(|_v| topic)
    }

    /// topics returns an iterator which returns all of the topics for the client's project.
    pub async fn get_topics(
        &self,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Vec<String>, Status> {
        let req = ListTopicsRequest {
            project: self.fully_qualified_project_name(),
            page_size: 0,
            page_token: "".to_string(),
        };
        self.pubc
            .list_topics(req, cancel, retry)
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

    use google_cloud_googleapis::pubsub::v1::PubsubMessage;
    use serial_test::serial;
    use std::thread;
    use std::time::Duration;

    use crate::client::Client;
    use google_cloud_gax::cancel::CancellationToken;
    use uuid::Uuid;

    use crate::subscriber::SubscriberConfig;
    use crate::subscription::{ReceiveConfig, SubscriptionConfig};

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt().try_init();
    }

    fn create_message(data: &[u8], ordering_key: &str) -> PubsubMessage {
        PubsubMessage {
            data: data.to_vec(),
            ordering_key: ordering_key.to_string(),
            ..Default::default()
        }
    }

    async fn create_client() -> Client {
        std::env::set_var("PUBSUB_EMULATOR_HOST", "localhost:8681");
        Client::default().await.unwrap()
    }

    async fn do_publish_and_subscribe(ordering_key: &str, bulk: bool) -> Result<(), anyhow::Error> {
        let client = create_client().await;

        let order = !ordering_key.is_empty();
        // create
        let uuid = Uuid::new_v4().hyphenated().to_string();
        let topic_id = &format!("t{}", &uuid);
        let subscription_id = &format!("s{}", &uuid);
        let ctx = Some(CancellationToken::new());
        let topic = client
            .create_topic(topic_id.as_str(), None, ctx.clone(), None)
            .await
            .unwrap();
        let publisher = topic.new_publisher(None);
        let config = SubscriptionConfig {
            enable_message_ordering: !ordering_key.is_empty(),
            ..Default::default()
        };
        let subscription = client
            .create_subscription(subscription_id.as_str(), topic_id.as_str(), config, ctx.clone(), None)
            .await
            .unwrap();

        let cancellation_token = CancellationToken::new();
        //subscribe
        let config = ReceiveConfig {
            worker_count: 2,
            subscriber_config: SubscriberConfig {
                ping_interval: Duration::from_secs(1),
                ..Default::default()
            },
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
                .map(|v| create_message(format!("abc_{}", v).as_bytes(), ordering_key))
                .collect();
            publisher.publish_bulk(messages).await
        } else {
            let mut awaiters = Vec::with_capacity(100);
            for v in 0..100 {
                let message = create_message(format!("abc_{}", v).as_bytes(), ordering_key);
                awaiters.push(publisher.publish(message).await);
            }
            awaiters
        };
        let ctx = CancellationToken::new();
        for v in awaiters {
            tracing::info!("sent message_id = {}", v.get(Some(ctx.clone())).await.unwrap());
        }

        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        cancellation_token.cancel();
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;

        let mut count = 0;
        while let Some(data) = r.recv().await {
            tracing::debug!("{}", data);
            if order {
                assert_eq!(format!("abc_{}", count), data);
            }
            count += 1;
        }
        assert_eq!(count, 100);
        let _ = handle.await;

        let mut publisher = publisher;
        publisher.shutdown().await;

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_publish_subscribe_ordered() -> Result<(), anyhow::Error> {
        do_publish_and_subscribe("ordering", false).await
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_publish_subscribe_ordered_bulk() -> Result<(), anyhow::Error> {
        do_publish_and_subscribe("ordering", true).await
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_publish_subscribe_random() -> Result<(), anyhow::Error> {
        do_publish_and_subscribe("", false).await
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_publish_subscribe_random_bulk() -> Result<(), anyhow::Error> {
        do_publish_and_subscribe("", true).await
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_lifecycle() -> Result<(), anyhow::Error> {
        let client = create_client().await;

        let uuid = Uuid::new_v4().hyphenated().to_string();
        let topic_id = &format!("t{}", &uuid);
        let subscription_id = &format!("s{}", &uuid);
        let ctx = Some(CancellationToken::new());
        let topics = client.get_topics(ctx.clone(), None).await.unwrap();
        let subs = client.get_subscriptions(ctx.clone(), None).await.unwrap();
        let _topic = client
            .create_topic(topic_id.as_str(), None, ctx.clone(), None)
            .await
            .unwrap();
        let _subscription = client
            .create_subscription(
                subscription_id.as_str(),
                topic_id.as_str(),
                SubscriptionConfig::default(),
                ctx.clone(),
                None,
            )
            .await?;
        let topics_after = client.get_topics(ctx.clone(), None).await.unwrap();
        let subs_after = client.get_subscriptions(ctx.clone(), None).await.unwrap();
        assert_eq!(1, topics_after.len() - topics.len());
        assert_eq!(1, subs_after.len() - subs.len());
        Ok(())
    }
}
