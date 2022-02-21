use tokio_util::sync::CancellationToken;
use google_cloud_googleapis::Status;
use crate::apiv1::conn_pool::ConnectionManager;
use crate::apiv1::publisher_client::PublisherClient;
use crate::apiv1::subscriber_client::SubscriberClient;
use crate::publisher::{PublisherConfig};
use crate::subscription::{Subscription, SubscriptionConfig};
use crate::topic::Topic;
use google_cloud_googleapis::pubsub::v1::{DetachSubscriptionRequest, ListSubscriptionsRequest, ListTopicsRequest, Subscription as InternalSubscription, Topic as InternalTopic};
use google_cloud_grpc::conn::Error;
use crate::apiv1::RetrySetting;

pub struct ClientConfig {
    pub pool_size: usize,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            pool_size: 4
        }
    }
}

/// Client is a Google Pub/Sub client scoped to a single project.
///
/// Clients should be reused rather than being created as needed.
/// A Client may be shared by multiple tasks.
#[derive(Clone)]
pub struct Client {
    project_id: String,
    pubc: PublisherClient,
    subc: SubscriberClient,
}

impl Client {
    /// new creates a client to a database. A valid database name has the
    /// form projects/PROJECT_ID/instances/INSTANCE_ID/databases/DATABASE_ID. It uses
    /// a default configuration.
    pub async fn new(project_id: &str, config: Option<ClientConfig>) -> Result<Self, Error> {
        let pool_size = config.unwrap_or_default().pool_size;
        let emulator_host = match std::env::var("PUBSUB_EMULATOR_HOST") {
            Ok(s) => Some(s),
            Err(_) => None,
        };
        let pubc = PublisherClient::new(ConnectionManager::new(pool_size, emulator_host.clone()).await?);
        let subc = SubscriberClient::new(ConnectionManager::new(pool_size, emulator_host).await?);
        return Ok(Self {
            project_id: project_id.to_string(),
            pubc,
            subc
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
    pub async fn create_subscription(&self, ctx: CancellationToken, subscription_id: &str, topic: &Topic, cfg: SubscriptionConfig, retry_option: Option<RetrySetting>) -> Result<Subscription, Status> {
        self.subc.create_subscription(ctx, InternalSubscription{
            name: self.fully_qualified_subscription_name(subscription_id),
            topic: topic.string().to_string(),
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
            topic_message_retention_duration: cfg.topic_message_retention_duration.map(|v| v.into())
        }, retry_option).await.map(|_v| self.subscription(subscription_id))
    }

    /// subscriptions returns an iterator which returns all of the subscriptions for the client's project.
    pub async fn subscriptions(&self, ctx: CancellationToken, retry_option: Option<RetrySetting>) -> Result<Vec<Subscription>, Status> {
        self.subc.list_subscriptions(ctx, ListSubscriptionsRequest {
            project: self.fully_qualified_project_name(),
            page_size: 0,
            page_token: "".to_string()
        }, retry_option).await.map(|v| v.into_iter().map( |x |
            Subscription::new(x.name.to_string(), self.subc.clone())).collect()
        )
    }

    /// subscription creates a reference to a subscription.
    pub fn subscription(&self, id: &str) -> Subscription {
        Subscription::new(self.fully_qualified_subscription_name(id), self.subc.clone())
    }

    /// detach_subscription detaches a subscription from its topic. All messages
    /// retained in the subscription are dropped. Subsequent `Pull` and `StreamingPull`
    /// requests will return FAILED_PRECONDITION. If the subscription is a push
    /// subscription, pushes to the endpoint will stop.
    pub async fn detach_subscription(&self, ctx :CancellationToken, sub_id: &str, retry_option: Option<RetrySetting>) -> Result<(), Status> {
        self.pubc.detach_subscription(ctx, DetachSubscriptionRequest{
            subscription: self.fully_qualified_subscription_name(sub_id),
        }, retry_option).await.map(|_v| ())
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
    pub async fn create_topic(&self, ctx: CancellationToken, topic_id: &str, publisher_config: Option<PublisherConfig>, retry_option:Option<RetrySetting>) -> Result<Topic, Status> {
        self.pubc.create_topic(ctx, InternalTopic {
            name: self.fully_qualified_topic_name(topic_id),
            labels: Default::default(),
            message_storage_policy: None,
            kms_key_name: "".to_string(),
            schema_settings: None,
            satisfies_pzs: false,
            message_retention_duration: None
        }, retry_option).await.map(|_v| self.topic(topic_id, publisher_config))
    }

    /// topics returns an iterator which returns all of the topics for the client's project.
    pub async fn topics(&self, ctx: CancellationToken, publisher_config: Option<PublisherConfig>, retry_option: Option<RetrySetting>) -> Result<Vec<Topic>, Status> {
        let config = publisher_config.unwrap_or_default();
        self.pubc.list_topics(ctx, ListTopicsRequest {
            project: self.fully_qualified_project_name(),
            page_size: 0,
            page_token: "".to_string()
        }, retry_option).await.map(|v| {
            v.into_iter().map(|x| {
                Topic::new(
                    x.name.to_string(),
                    self.pubc.clone(),
                    self.subc.clone(),
                    Some(config.clone()),
                    )
            }).collect()
        })
    }

    /// topic creates a reference to a topic in the client's project.
    ///
    /// If a Topic's Publish method is called, it has background tasks
    /// associated with it. Clean them up by calling topic.stop.
    ///
    /// Avoid creating many Topic instances if you use them to publish.
    pub fn topic(&self, id: &str, config: Option<PublisherConfig>) -> Topic {
        Topic::new(self.fully_qualified_topic_name(id), self.pubc.clone(), self.subc.clone(), config)
    }

    fn fully_qualified_topic_name(&self, id: &str) -> String {
        format!("projects/{}/topics/{}", self.project_id, id)
    }

    fn fully_qualified_subscription_name(&self, id: &str) -> String {
        format!("projects/{}/subscriptions/{}", self.project_id, id)
    }

    fn fully_qualified_project_name(&self) -> String {
        format!("projects/{}", self.project_id)
    }
}

#[cfg(test)]
mod tests {
    
    
    
    use std::thread;
    use std::time::Duration;
    use google_cloud_googleapis::pubsub::v1::PubsubMessage;
    use serial_test::serial;
    use tokio_util::sync::CancellationToken;
    
    use uuid::Uuid;
    use crate::client::{Client};
    
    use crate::subscriber::SubscriberConfig;
    use crate::subscription::{ReceiveConfig, SubscriptionConfig};

    fn create_message(data: &[u8], ordering_key: &str) -> PubsubMessage {
        PubsubMessage {
            data: data.to_vec(),
            attributes: Default::default(),
            message_id: "".to_string(),
            publish_time: None,
            ordering_key: ordering_key.to_string()
        }
    }

    async fn create_client() -> Client {
        std::env::set_var("RUST_LOG","google_cloud_pubsub=trace".to_string());
        env_logger::try_init();
        std::env::set_var("PUBSUB_EMULATOR_HOST","localhost:8681".to_string());
        Client::new("local-project", None).await.unwrap()
    }

    async fn do_publish_and_subscribe(ordering_key: &str) -> Result<(), anyhow::Error> {
        let client = create_client().await;

        let order = !ordering_key.is_empty();
        // create
        let uuid = Uuid::new_v4().to_hyphenated().to_string();
        let topic_name = &format!("t{}", &uuid);
        let subscription_name = &format!("s{}", &uuid);
        let ctx = CancellationToken::new();
        let topic = client.create_topic(ctx, topic_name, None, None).await.unwrap();
        let mut config = SubscriptionConfig::default();
        config.enable_message_ordering = !ordering_key.is_empty();
        let ctx = CancellationToken::new();
        let subscription = client.create_subscription(ctx.clone(), subscription_name , &topic, config, None).await.unwrap();

        let cancellation_token = CancellationToken::new();
        //subscribe
        let mut config = ReceiveConfig {
            worker_count: 2,
            subscriber_config: SubscriberConfig::default(),
        };
        let cancel_receiver = cancellation_token.clone();
        config.subscriber_config.ping_interval = Duration::from_secs(1);
        let (s, mut r) = tokio::sync::mpsc::channel(100);
        let handle = tokio::spawn(async move {
            subscription.receive(cancel_receiver, move |v, _ctx| {
                let s2 = s.clone();
                async move {
                    let _ = v.ack().await;
                    let data = std::str::from_utf8(&v.message.data).unwrap().to_string();
                    log::info!("tid={:?} id={} data={}", thread::current().id(), v.message.message_id, data);
                    s2.send(data).await;
                }
            }, Some(config)).await;
        });

        //publish
        let mut awaiters = Vec::with_capacity(100);
        for v in 0..100 {
            let message = create_message(format!("abc_{}",v).as_bytes(), ordering_key);
            awaiters.push(topic.publish(message).await);
        }
        let ctx = CancellationToken::new();
        for mut v in awaiters {
            log::info!("sent message_id = {}", v.get(ctx.clone()).await.unwrap());
        }

        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        cancellation_token.cancel();
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;

        let mut count = 0;
        while let Some(data) = r.recv().await {
            log::debug!("{}", data);
            if order {
                assert_eq!(format!("abc_{}", count), data);
            }
            count += 1;
        }
        assert_eq!(count, 100);
        let _ = handle.await;

        topic.stop();

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_publish_subscribe_ordered() -> Result<(), anyhow::Error> {
       do_publish_and_subscribe("ordering").await
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_publish_subscribe_random() -> Result<(), anyhow::Error> {
        do_publish_and_subscribe("").await
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_lifecycle() -> Result<(), anyhow::Error>{
       let client = create_client().await;

        let uuid = Uuid::new_v4().to_hyphenated().to_string();
        let topic_id = &format!("t{}", &uuid);
        let subscription_id = &format!("s{}", &uuid);
        let ctx = CancellationToken::new();
        let topics = client.topics(ctx.clone(), None, None) .await.unwrap();
        let subs = client.subscriptions(CancellationToken::new(), None) .await.unwrap();
        let ctx = CancellationToken::new();
        let topic = client.create_topic(ctx.clone(), topic_id, None, None).await.unwrap();
        let _subscription= client.create_subscription(CancellationToken::new(), subscription_id, &topic, SubscriptionConfig::default(), None).await?;
        let topics_after = client.topics(ctx.clone(), None, None) .await.unwrap();
        let subs_after= client.subscriptions(CancellationToken::new(), None) .await.unwrap();
        assert_eq!(1, topics_after.len() - topics.len());
        assert_eq!(1, subs_after.len() - subs.len());
        Ok(())
    }
}
