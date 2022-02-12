use std::collections::HashMap;
use std::time::Duration;
use google_cloud_googleapis::Status;
use crate::apiv1::conn_pool::ConnectionManager;
use crate::apiv1::publisher_client::PublisherClient;
use crate::apiv1::subscriber_client::SubscriberClient;
use crate::publisher::{Publisher, PublisherConfig};
use crate::subscription::{Subscription, SubscriptionConfig};
use crate::topic::Topic;
use google_cloud_googleapis::pubsub::v1::{DeadLetterPolicy, DetachSubscriptionRequest, ExpirationPolicy, ListSubscriptionsRequest, ListTopicsRequest, PushConfig, RetryPolicy, Subscription as InternalSubscription, Topic as InternalTopic};
use google_cloud_grpc::conn::Error;

pub struct Config {
    pub pool_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pool_size: 4
        }
    }
}


pub struct Client {
   project_id: String,
   cm: ConnectionManager,
}

impl Client {
    pub async fn new(project_id: &str, config: Option<Config>) -> Result<Self, Error> {
        let pool_size = config.unwrap_or(Config::default()).pool_size;
        let emulator_host = match std::env::var("PUBSUB_EMULATOR_HOST") {
            Ok(s) => Some(s),
            Err(_) => None,
        };
        let cm = ConnectionManager::new(pool_size, emulator_host).await?;
        return Ok(Self {
           project_id: project_id.to_string(),
           cm,
        })
    }

    pub async fn create_subscription(&self, subscription_id: &str, topic: &Topic, op: SubscriptionConfig) -> Result<Subscription, Status> {
        let mut client = SubscriberClient::new(self.cm.conn()) ;
        client.create_subscription(InternalSubscription{
            name: self.subscription_name(subscription_id),
            topic: topic.string().to_string(),
            push_config: op.push_config,
            ack_deadline_seconds: op.ack_deadline_seconds,
            labels: op.labels,
            enable_message_ordering: op.enable_message_ordering,
            expiration_policy: op.expiration_policy,
            filter: op.filter,
            dead_letter_policy: op.dead_letter_policy,
            retry_policy: op.retry_policy,
            detached: op.detached,
            message_retention_duration: op.message_retention_duration.map(|v| v.into()),
            retain_acked_messages: op.retain_acked_messages,
            topic_message_retention_duration: op.topic_message_retention_duration.map(|v| v.into())
        }, None).await.map(|v| self.subscription(subscription_id))
    }

    pub async fn subscriptions(&self) -> Result<Vec<Subscription>, Status> {
        let mut subc= SubscriberClient::new(self.cm.conn());
        subc.list_subscriptions(ListSubscriptionsRequest {
            project: self.project_id.to_string(),
            page_size: 0,
            page_token: "".to_string()
        }, None).await.map(|v| v.into_iter().map( |x |
            Subscription::new(x.name.to_string(), subc.clone())).collect()
        )
    }

    pub fn subscription(&self, id: &str) -> Subscription {
        let subc= SubscriberClient::new(self.cm.conn());
        Subscription::new(self.subscription_name(id), subc)
    }

    pub async fn detach_subscription(&self, sub_id: &str) -> Result<(), Status> {
        let mut client = PublisherClient::new(self.cm.conn());
        client.detach_subscription(DetachSubscriptionRequest{
            subscription: self.subscription_name(sub_id),
        }, None).await.map(|v| ())
    }

    pub async fn create_topic(&self, topic_id: &str, topic_config: Option<PublisherConfig>) -> Result<Topic, Status> {
        let mut client = PublisherClient::new(self.cm.conn()) ;
        client.create_topic(InternalTopic {
            name: self.topic_name(topic_id),
            labels: Default::default(),
            message_storage_policy: None,
            kms_key_name: "".to_string(),
            schema_settings: None,
            satisfies_pzs: false,
            message_retention_duration: None
        }, None).await.map(|v| self.topic(topic_id, topic_config))
    }

    pub async fn topics(&self, config: Option<PublisherConfig>) -> Result<Vec<Topic>, Status> {
        let mut pubc = PublisherClient::new(self.cm.conn());
        let opt = config.unwrap_or_default();
        pubc.list_topics(ListTopicsRequest {
            project: self.project_id.to_string(),
            page_size: 0,
            page_token: "".to_string()
        }, None).await.map(|v| {
            v.into_iter().map(|x| {
                Topic::new(
                    x.name.to_string(),
                    pubc.clone(),
                    SubscriberClient::new(self.cm.conn()),
                    opt.clone(),
                    )
            }).collect()
        })
    }

    pub fn topic(&self, id: &str, config: Option<PublisherConfig>) -> Topic {
        let pubc = PublisherClient::new(self.cm.conn());
        let subc= SubscriberClient::new(self.cm.conn());
        Topic::new(self.topic_name(id), pubc, subc, config.unwrap_or(PublisherConfig::default()))
    }

    fn topic_name(&self, id: &str) -> String {
        format!("projects/{}/topics/{}", self.project_id, id)
    }

    fn subscription_name(&self, id: &str) -> String {
        format!("projects/{}/subscriptions/{}", self.project_id, id)
    }
}