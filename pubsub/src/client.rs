use google_cloud_gax::call_option::BackoffRetrySettings;
use google_cloud_googleapis::Status;
use crate::apiv1::conn_pool::ConnectionManager;
use crate::apiv1::publisher_client::PublisherClient;
use crate::apiv1::subscriber_client::SubscriberClient;
use crate::publisher::{PublisherConfig};
use crate::subscription::{Subscription, SubscriptionConfig};
use crate::topic::Topic;
use google_cloud_googleapis::pubsub::v1::{DetachSubscriptionRequest, ListSubscriptionsRequest, ListTopicsRequest, Subscription as InternalSubscription, Topic as InternalTopic};
use google_cloud_grpc::conn::Error;

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
    pub async fn create_subscription(&self, subscription_id: &str, topic: &Topic, cfg: SubscriptionConfig, retry_option: Option<BackoffRetrySettings>) -> Result<Subscription, Status> {
        self.subc.create_subscription(InternalSubscription{
            name: self.subscription_name(subscription_id),
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
    pub async fn subscriptions(&self, retry_option: Option<BackoffRetrySettings>) -> Result<Vec<Subscription>, Status> {
        self.subc.list_subscriptions(ListSubscriptionsRequest {
            project: self.project_id.to_string(),
            page_size: 0,
            page_token: "".to_string()
        }, retry_option).await.map(|v| v.into_iter().map( |x |
            Subscription::new(x.name.to_string(), self.subc.clone())).collect()
        )
    }

    /// subscription creates a reference to a subscription.
    pub fn subscription(&self, id: &str) -> Subscription {
        Subscription::new(self.subscription_name(id), self.subc.clone())
    }

    /// detach_subscription detaches a subscription from its topic. All messages
    /// retained in the subscription are dropped. Subsequent `Pull` and `StreamingPull`
    /// requests will return FAILED_PRECONDITION. If the subscription is a push
    /// subscription, pushes to the endpoint will stop.
    pub async fn detach_subscription(&self, sub_id: &str, retry_option: Option<BackoffRetrySettings>) -> Result<(), Status> {
        self.pubc.detach_subscription(DetachSubscriptionRequest{
            subscription: self.subscription_name(sub_id),
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
    pub async fn create_topic(&self, topic_id: &str, publisher_config: Option<PublisherConfig>, retry_option:Option<BackoffRetrySettings>) -> Result<Topic, Status> {
        self.pubc.create_topic(InternalTopic {
            name: self.topic_name(topic_id),
            labels: Default::default(),
            message_storage_policy: None,
            kms_key_name: "".to_string(),
            schema_settings: None,
            satisfies_pzs: false,
            message_retention_duration: None
        }, retry_option).await.map(|_v| self.topic(topic_id, publisher_config))
    }

    /// topics returns an iterator which returns all of the topics for the client's project.
    pub async fn topics(&self, publisher_config: Option<PublisherConfig>, retry_option: Option<BackoffRetrySettings>) -> Result<Vec<Topic>, Status> {
        let config = publisher_config.unwrap_or_default();
        self.pubc.list_topics(ListTopicsRequest {
            project: self.project_id.to_string(),
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
        Topic::new(self.topic_name(id), self.pubc.clone(), self.subc.clone(), config)
    }

    fn topic_name(&self, id: &str) -> String {
        format!("projects/{}/topics/{}", self.project_id, id)
    }

    fn subscription_name(&self, id: &str) -> String {
        format!("projects/{}/subscriptions/{}", self.project_id, id)
    }
}