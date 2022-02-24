use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use parking_lot::{Mutex};
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use google_cloud_googleapis::Code::NotFound;
use google_cloud_googleapis::pubsub::v1::{DeleteTopicRequest, GetTopicRequest, ListTopicSubscriptionsRequest, MessageStoragePolicy, PubsubMessage, SchemaSettings, Topic as InternalTopic};

use google_cloud_googleapis::Status;
use crate::apiv1::publisher_client::PublisherClient;
use crate::apiv1::RetrySetting;
use crate::apiv1::subscriber_client::SubscriberClient;
use crate::publisher::{Awaiter, Publisher, PublisherConfig, ReservedMessage};
use crate::subscription::Subscription;
use crate::util::ToUsize;

pub struct TopicConfig {
   pub labels: HashMap<String,String>,
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
         message_retention_duration: None
      }
   }
}

/// Topic is a reference to a PubSub topic.
///
/// The methods of Topic are safe for use by multiple tasks.
#[derive(Clone)]
pub struct Topic {
   fqtn: String,
   pubc: PublisherClient,
   subc: SubscriberClient,
   ordering_senders: Vec<async_channel::Sender<ReservedMessage>>,
   sender: Option<async_channel::Sender<ReservedMessage>>,
   publisher: Arc<Mutex<Option<Publisher>>>
}

impl Topic {

   pub(crate) fn new(fqtn: String, pubc: PublisherClient, subc: SubscriberClient) -> Self {
      Self {
         fqtn,
         pubc,
         subc,
         ordering_senders: vec![],
         sender: None,
         publisher: Arc::new(Mutex::new(None)),
      }
   }

   /// id returns the unique identifier of the topic within its project.
   pub fn id(&self) -> String {
      self.fqtn.rfind('/').map_or("".to_string(),|i| self.fqtn[(i + 1)..].to_string())
   }

   /// fully_qualified_name returns the printable globally unique name for the topic.
   pub fn fully_qualified_name(&self) -> &str {
     self.fqtn.as_str()
   }

   pub fn run(&mut self, config: Option<PublisherConfig>) {
      let mut publisher = self.publisher.lock();
      if publisher.is_none() {
         let config = config.unwrap_or_default();
         let (sender, receiver) = async_channel::unbounded::<ReservedMessage>();
         let mut receivers = Vec::with_capacity(1 + config.workers);
         let mut ordering_senders = Vec::with_capacity(config.workers);

         // for non-ordering key message
         for _ in 0..config.workers {
            log::trace!("start non-ordering publisher : {}", self.fqtn.clone());
            receivers.push(receiver.clone());
         }

         // for ordering key message
         for _ in 0..config.workers {
            log::trace!("start ordering publisher : {}", self.fqtn.clone());
            let (sender, receiver) = async_channel::unbounded::<ReservedMessage>();
            receivers.push(receiver);
            ordering_senders.push(sender);
         }
         *publisher = Some(Publisher::start(self.fqtn.to_string(), self.pubc.clone(), receivers, config));
         self.sender = Some(sender);
         self.ordering_senders = ordering_senders;
      }
   }

   /// create creates the topic.
   pub async fn create(&self, ctx: CancellationToken, cfg: Option<TopicConfig>, opt: Option<RetrySetting>) -> Result<(),Status>{
      let topic_config = cfg.unwrap_or_default();
      self.pubc.create_topic(ctx, InternalTopic {
         name: self.fully_qualified_name().to_string(),
         labels: topic_config.labels,
         message_storage_policy: topic_config.message_storage_policy,
         kms_key_name: topic_config.kms_key_name,
         schema_settings:topic_config.schema_settings,
         satisfies_pzs: topic_config.satisfies_pzs,
         message_retention_duration: topic_config.message_retention_duration.map(|v| v.into()),
      }, opt).await.map(|v| ())
   }

   /// delete deletes the topic.
   pub async fn delete(&self, ctx: CancellationToken, opt: Option<RetrySetting>) -> Result<(),Status>{
      self.pubc.delete_topic(ctx, DeleteTopicRequest {
         topic: self.fqtn.to_string()
      }, opt).await.map(|v| v.into_inner())
   }

   /// exists reports whether the topic exists on the server.
   pub async fn exists(&self, ctx: CancellationToken, opt: Option<RetrySetting>) -> Result<bool,Status>{
      if self.fqtn == "_deleted-topic_" {
         return Ok(false)
      }
      match self.pubc.get_topic(ctx, GetTopicRequest{
         topic: self.fqtn.to_string()
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
   pub async fn subscriptions(&self, ctx: CancellationToken, opt: Option<RetrySetting>) -> Result<Vec<Subscription>,Status>{
      self.pubc.list_topic_subscriptions(ctx, ListTopicSubscriptionsRequest{
         topic: self.fqtn.to_string(),
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
      if self.is_shutdown() {
         let (mut tx, rx) = tokio::sync::oneshot::channel();
         tx.closed();
         return Awaiter::new(rx)
      }

      let (producer, consumer) = oneshot::channel();
      if message.ordering_key.is_empty() {
         self.sender.as_ref().unwrap().send( ReservedMessage {
            producer,
            message
         }).await;
      }else {
         let key = message.ordering_key.as_str().to_usize();
         let index = key % self.ordering_senders.len();
         self.ordering_senders[index].send(ReservedMessage {
            producer,
            message
         }).await;
      }
      Awaiter::new(consumer)
   }

   pub async fn shutdown(&self) {
      match self.sender.as_ref() {
         Some(sender) => {
            sender.close();
            for s in &self.ordering_senders {
               s.close();
            }
            let mut publisher = self.publisher.lock().borrow_mut().take().unwrap();
            publisher.done().await;
         },
         None => {}
      }
   }

   fn is_shutdown(&self) -> bool{
      match self.sender.as_ref() {
         Some(v) => v.is_closed(),
         None => true
      }
   }

}

#[cfg(test)]
mod tests {
   use std::borrow::BorrowMut;
   use std::time::Duration;
   use uuid::Uuid;
   use google_cloud_googleapis::pubsub::v1::{PubsubMessage, Topic as InternalTopic};
   use serial_test::serial;
   use tokio::task::JoinHandle;
   use tokio_util::sync::CancellationToken;
   use google_cloud_googleapis::Code;
   use crate::apiv1::conn_pool::ConnectionManager;
   use crate::apiv1::publisher_client::PublisherClient;
   use crate::apiv1::subscriber_client::SubscriberClient;
   use crate::publisher::PublisherConfig;
   use crate::topic::Topic;

   #[ctor::ctor]
   fn init() {
      std::env::set_var("RUST_LOG","google_cloud_pubsub=trace".to_string());
      env_logger::try_init();
   }

   #[tokio::test]
   #[serial]
   async fn test_topic() -> Result<(), anyhow::Error> {
      let cm1 = ConnectionManager::new(4, Some("localhost:8681".to_string())).await?;
      let pubc = PublisherClient::new(cm1);
      let cm2 = ConnectionManager::new(4, Some("localhost:8681".to_string())).await?;
      let subc = SubscriberClient::new(cm2);

      let uuid = Uuid::new_v4().to_hyphenated().to_string();
      let topic_name = format!("projects/local-project/topics/t{}",uuid).to_string();
      let ctx = CancellationToken::new();

      // Create topic.
      let mut topic = Topic::new(topic_name, pubc, subc);
      if !topic.exists(ctx.clone(), None).await? {
         topic.create(ctx.clone(), None, None).await?;
      }
      
      // Start publisher.
      topic.run(None);
      let topic = topic;

      // Publish message.
      let tasks : Vec<JoinHandle<String>> = (0..10).into_iter().map(|_i| {
         let topic = topic.clone();
         let ctx = ctx.clone();
         tokio::spawn(async move {
            let mut msg = PubsubMessage::default();
            msg.data = "abc".into();
            let mut awaiter = topic.publish(msg).await;
            awaiter.get(ctx).await.unwrap()
         })
      }).collect();

      // Wait for all publish task finish
      for task in tasks {
         let message_id= task.await?;
         log::trace!("{}", message_id);
         assert!(!message_id.is_empty())
      }

      // Wait for publishers in topic finish.
      topic.shutdown();

      topic.delete(ctx.clone(), None).await?;

      Ok(())

   }
}