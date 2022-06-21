use std::collections::HashMap;

use google_cloud_gax::cancel::CancellationToken;
use std::time::Duration;

use google_cloud_gax::grpc::{Code, Status};
use google_cloud_gax::retry::RetrySetting;

use crate::apiv1::publisher_client::PublisherClient;
use crate::apiv1::subscriber_client::SubscriberClient;
use crate::publisher::{Publisher, PublisherConfig};
use crate::subscription::Subscription;
use google_cloud_googleapis::pubsub::v1::{
    DeleteTopicRequest, GetTopicRequest, ListTopicSubscriptionsRequest, MessageStoragePolicy, SchemaSettings,
    Topic as InternalTopic,
};

pub struct TopicConfig {
    pub labels: HashMap<String, String>,
    pub message_storage_policy: Option<MessageStoragePolicy>,
    pub kms_key_name: String,
    pub schema_settings: Option<SchemaSettings>,
    pub satisfies_pzs: bool,
    pub message_retention_duration: Option<Duration>,
}

impl Default for TopicConfig {
    fn default() -> Self {
        Self {
            labels: HashMap::default(),
            message_storage_policy: None,
            kms_key_name: "".to_string(),
            schema_settings: None,
            satisfies_pzs: false,
            message_retention_duration: None,
        }
    }
}

/// Topic is a reference to a PubSub topic.
///
/// The methods of Topic are safe for use by multiple tasks.
#[derive(Clone, Debug)]
pub struct Topic {
    fqtn: String,
    pubc: PublisherClient,
    subc: SubscriberClient,
}

impl Topic {
    pub(crate) fn new(fqtn: String, pubc: PublisherClient, subc: SubscriberClient) -> Self {
        Self { fqtn, pubc, subc }
    }

    /// id returns the unique identifier of the topic within its project.
    pub fn id(&self) -> String {
        self.fqtn
            .rfind('/')
            .map_or("".to_string(), |i| self.fqtn[(i + 1)..].to_string())
    }

    /// fully_qualified_name returns the printable globally unique name for the topic.
    pub fn fully_qualified_name(&self) -> &str {
        self.fqtn.as_str()
    }

    pub fn new_publisher(&self, config: Option<PublisherConfig>) -> Publisher {
        Publisher::new(self.fqtn.clone(), self.pubc.clone(), config)
    }

    /// create creates the topic.
    pub async fn create(
        &self,
        cfg: Option<TopicConfig>,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<(), Status> {
        let topic_config = cfg.unwrap_or_default();
        let req = InternalTopic {
            name: self.fully_qualified_name().to_string(),
            labels: topic_config.labels,
            message_storage_policy: topic_config.message_storage_policy,
            kms_key_name: topic_config.kms_key_name,
            schema_settings: topic_config.schema_settings,
            satisfies_pzs: topic_config.satisfies_pzs,
            message_retention_duration: topic_config.message_retention_duration.map(|v| v.into()),
        };
        self.pubc.create_topic(req, cancel, retry).await.map(|_v| ())
    }

    /// delete deletes the topic.
    pub async fn delete(&self, cancel: Option<CancellationToken>, retry: Option<RetrySetting>) -> Result<(), Status> {
        let req = DeleteTopicRequest {
            topic: self.fqtn.to_string(),
        };
        self.pubc.delete_topic(req, cancel, retry).await.map(|v| v.into_inner())
    }

    /// exists reports whether the topic exists on the server.
    pub async fn exists(&self, cancel: Option<CancellationToken>, retry: Option<RetrySetting>) -> Result<bool, Status> {
        if self.fqtn == "_deleted-topic_" {
            return Ok(false);
        }
        let req = GetTopicRequest {
            topic: self.fqtn.to_string(),
        };
        match self.pubc.get_topic(req, cancel, retry).await {
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

    /// Subscriptions returns an iterator which returns the subscriptions for this topic.
    pub async fn subscriptions(
        &self,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Vec<Subscription>, Status> {
        let req = ListTopicSubscriptionsRequest {
            topic: self.fqtn.to_string(),
            page_size: 0,
            page_token: "".to_string(),
        };
        self.pubc.list_topic_subscriptions(req, cancel, retry).await.map(|v| {
            v.into_iter()
                .map(|sub_name| Subscription::new(sub_name, self.subc.clone()))
                .collect()
        })
    }
}

#[cfg(test)]
mod tests {

    use crate::apiv1::conn_pool::ConnectionManager;
    use crate::apiv1::publisher_client::PublisherClient;
    use crate::apiv1::subscriber_client::SubscriberClient;
    use crate::publisher::{Publisher, PublisherConfig};
    use crate::topic::Topic;
    use google_cloud_gax::cancel::CancellationToken;
    use google_cloud_gax::conn::Environment;
    use google_cloud_gax::grpc::Status;
    use google_cloud_googleapis::pubsub::v1::PubsubMessage;
    use serial_test::serial;
    use std::time::Duration;
    use tokio::task::JoinHandle;
    use tokio::time::sleep;
    use uuid::Uuid;

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt().try_init();
    }

    async fn create_topic() -> Result<Topic, anyhow::Error> {
        let cm1 = ConnectionManager::new(4, &Environment::Emulator("localhost:8681".to_string())).await?;
        let pubc = PublisherClient::new(cm1);
        let cm2 = ConnectionManager::new(4, &Environment::Emulator("localhost:8681".to_string())).await?;
        let subc = SubscriberClient::new(cm2);

        let uuid = Uuid::new_v4().to_hyphenated().to_string();
        let topic_name = format!("projects/local-project/topics/t{}", uuid);
        let ctx = CancellationToken::new();

        // Create topic.
        let topic = Topic::new(topic_name, pubc, subc);
        if !topic.exists(Some(ctx.clone()), None).await? {
            topic.create(None, Some(ctx.clone()), None).await?;
        }
        Ok(topic)
    }

    async fn publish(ctx: CancellationToken, publisher: Publisher) -> Vec<JoinHandle<Result<String, Status>>> {
        (0..10)
            .into_iter()
            .map(|_i| {
                let publisher = publisher.clone();
                let ctx = ctx.clone();
                tokio::spawn(async move {
                    let msg = PubsubMessage {
                        data: "abc".into(),
                        ..Default::default()
                    };
                    let awaiter = publisher.publish(msg).await;
                    awaiter.get(Some(ctx)).await
                })
            })
            .collect()
    }

    #[tokio::test]
    #[serial]
    async fn test_publish() -> Result<(), anyhow::Error> {
        let ctx = CancellationToken::new();
        let topic = create_topic().await?;
        let publisher = topic.new_publisher(None);

        // Publish message.
        let tasks = publish(ctx.clone(), publisher.clone()).await;

        // Wait for all publish task finish
        for task in tasks {
            let message_id = task.await??;
            tracing::trace!("{}", message_id);
            assert!(!message_id.is_empty())
        }

        // Wait for publishers in topic finish.
        let mut publisher = publisher;
        publisher.shutdown().await;

        // Can't publish messages
        let result = publisher
            .publish(PubsubMessage::default())
            .await
            .get(Some(ctx.clone()))
            .await;
        assert!(result.is_err());

        topic.delete(Some(ctx.clone()), None).await?;

        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_publish_cancel() -> Result<(), anyhow::Error> {
        let ctx = CancellationToken::new();
        let topic = create_topic().await?;
        let config = PublisherConfig {
            flush_interval: Duration::from_secs(10),
            bundle_size: 11,
            ..Default::default()
        };
        let publisher = topic.new_publisher(Some(config));

        // Publish message.
        let tasks = publish(ctx.clone(), publisher.clone()).await;

        // Shutdown after 1 sec
        sleep(Duration::from_secs(1)).await;
        let mut publisher = publisher;
        publisher.shutdown().await;

        // Confirm flush bundle.
        for task in tasks {
            let message_id = task.await?;
            assert!(message_id.is_ok());
            assert!(!message_id.unwrap().is_empty());
        }

        // Can't publish messages
        let result = publisher
            .publish(PubsubMessage::default())
            .await
            .get(Some(ctx.clone()))
            .await;
        assert!(result.is_err());

        topic.delete(Some(ctx.clone()), None).await?;
        assert!(!topic.exists(Some(ctx), None).await?);

        Ok(())
    }
}
