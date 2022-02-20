use std::sync::Arc;
use parking_lot::{Mutex};

use google_cloud_googleapis::Code::NotFound;
use google_cloud_googleapis::pubsub::v1::{DeleteTopicRequest, GetTopicRequest, ListTopicSubscriptionsRequest, PubsubMessage};

use google_cloud_googleapis::Status;
use crate::apiv1::publisher_client::PublisherClient;
use crate::apiv1::RetrySetting;
use crate::apiv1::subscriber_client::SubscriberClient;
use crate::publisher::{Awaiter, Publisher, PublisherConfig};
use crate::subscription::Subscription;

/// Topic is a reference to a PubSub topic.
///
/// The methods of Topic are safe for use by multiple tasks.
#[derive(Clone)]
pub struct Topic {
   name: String,
   pubc: PublisherClient,
   subc: SubscriberClient,
   config: Option<PublisherConfig>,
   publisher: Arc<Mutex<Option<Publisher>>>
}

impl Topic {

   pub(crate) fn new(name: String,
          pubc: PublisherClient,
          subc: SubscriberClient,
          config: Option<PublisherConfig>) -> Self {
      Self {
         name,
         pubc,
         subc,
         config,
         publisher: Arc::new(Mutex::new(None))
      }
   }

   /// id returns the unique identifier of the topic within its project.
   pub fn id(&self) -> String {
      self.name.rfind('/').map_or("".to_string(),|i| self.name[(i + 1)..].to_string())
   }

   /// string returns the printable globally unique name for the topic.
   pub fn string(&self) -> &str {
     self.name.as_str()
   }

   /// delete deletes the topic.
   pub async fn delete(&self, opt: Option<RetrySetting>) -> Result<(),Status>{
      self.pubc.delete_topic(DeleteTopicRequest {
         topic: self.name.to_string()
      }, opt).await.map(|v| v.into_inner())
   }

   /// exists reports whether the topic exists on the server.
   pub async fn exists(&self, opt: Option<RetrySetting>) -> Result<bool,Status>{
      if self.name == "_deleted-topic_" {
         return Ok(false)
      }
      match self.pubc.get_topic(GetTopicRequest{
         topic: self.name.to_string()
      }, opt).await {
         Ok(_) => Ok(true),
         Err(e) => if e.code() == NotFound {
            Ok(false)
         }else {
            Err(e)
         }
      }
   }

   /// Subscriptions returns an iterator which returns the subscriptions for this topic.
   ///
   /// Some of the returned subscriptions may belong to a project other than t.
   pub async fn subscriptions(&self, opt: Option<RetrySetting>) -> Result<Vec<Subscription>,Status>{
      self.pubc.list_topic_subscriptions(ListTopicSubscriptionsRequest{
         topic: self.name.to_string(),
         page_size: 0,
         page_token: "".to_string(),
      }, opt).await.map(|v| v.into_iter().map(|sub_name| Subscription::new(sub_name, self.subc.clone())).collect())
   }

   /// publish publishes msg to the topic asynchronously. Messages are batched and
   /// sent according to the topic's PublisherConfig. Publish never blocks.
   ///
   /// publish returns a non-nil Awaiter which will be ready when the
   /// message has been sent (or has failed to be sent) to the server.
   ///
   /// publish creates tasks for batching and sending messages. These tasks
   /// need to be stopped by calling t.stop(). Once stopped, future calls to Publish
   /// will immediately return a Awaiter with an error.
   pub async fn publish(&self, message: PubsubMessage) -> Awaiter {
      let mut lock = self.publisher.lock();
      if lock.is_none() {
         *lock = Some(Publisher::new(self.name.clone(), self.pubc.clone(),self.config.clone()));
      }
      lock.as_ref().unwrap().publish(message).await
   }

   pub async fn stop(&self) {
      if let Some(mut p) = { self.publisher.lock().take() } {
         p.stop().await;
      }
   }

}

impl Drop for Topic {

   fn drop(&mut self) {
      self.stop() ;
   }

}

#[cfg(test)]
mod tests {
   use std::collections::HashMap;
   use std::time::Duration;
   use uuid::Uuid;
   use google_cloud_googleapis::pubsub::v1::{ExpirationPolicy, MessageStoragePolicy, PubsubMessage, Topic as InternalTopic};
   use serial_test::serial;
   use crate::apiv1::conn_pool::ConnectionManager;
   use crate::apiv1::publisher_client::PublisherClient;
   use crate::apiv1::subscriber_client::SubscriberClient;
   use crate::topic::Topic;

   #[tokio::test]
   #[serial]
   async fn test_topic() -> Result<(), anyhow::Error> {
      std::env::set_var("RUST_LOG","google_cloud_pubsub=trace".to_string());
      env_logger::init();
      let cm = ConnectionManager::new(4, Some("localhost:8681".to_string())).await?;
      let client = PublisherClient::new(cm);

      let uuid = Uuid::new_v4().to_hyphenated().to_string();
      let topic_name = format!("projects/local-project/topics/t{}",uuid).to_string();
      let topic = client.create_topic(InternalTopic {
         name: topic_name.to_string(),
         message_retention_duration: None,
         labels: Default::default(),
         message_storage_policy: None,
         kms_key_name: "".to_string(),
         schema_settings: None,
         satisfies_pzs: false
      }, None).await?.into_inner();

      let subcm = ConnectionManager::new(4, Some("localhost:8681".to_string())).await?;
      let subc = SubscriberClient::new(subcm);
      let mut topic = Topic::new(topic.name, client, subc, None);
      assert!(topic.exists(None).await?);

      let subs = topic.subscriptions(None).await?;
      assert_eq!(0, subs.len());

      let msg = PubsubMessage {
         data: "aaa".as_bytes().to_vec(),
         attributes: Default::default(),
         message_id: "".to_string(),
         publish_time: None,
         ordering_key: "".to_string()
      };
      let message_id = topic.publish(msg.clone()).await.get().await;
      assert!(message_id.unwrap().len() > 0);

      topic.stop().await;
      let message_id = topic.publish(msg).await.get().await;
      assert!(message_id.unwrap().len() > 0);

      topic.stop().await;
      topic.delete(None).await?;

      assert!(!topic.exists(None).await?);

      Ok(())

   }
}