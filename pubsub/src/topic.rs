use std::sync::Arc;
use parking_lot::{Mutex};

use google_cloud_googleapis::Code::NotFound;
use google_cloud_googleapis::pubsub::v1::{DeleteTopicRequest, GetTopicRequest, ListTopicSubscriptionsRequest, PubsubMessage};

use google_cloud_googleapis::Status;
use crate::apiv1::publisher_client::PublisherClient;
use crate::apiv1::subscriber_client::SubscriberClient;
use crate::publisher::{Awaiter, Publisher, PublisherConfig};
use crate::subscription::Subscription;

struct InternalTopic {
   name: String,
   pubc: PublisherClient,
   subc: SubscriberClient,
   config: Option<PublisherConfig>,
   publisher: Mutex<Option<Publisher>>
}

/// Topic is a reference to a PubSub topic.
#[derive(Clone)]
pub struct Topic {
   inner: Arc<InternalTopic>
}

impl Topic {

   pub fn new(name: String,
          pubc: PublisherClient,
          subc: SubscriberClient,
          config: Option<PublisherConfig>) -> Self {
      Self {
         inner: Arc::new(InternalTopic {
            name,
            pubc,
            subc,
            config,
            publisher: Mutex::new(None)
         })
      }
   }

   /// id returns the unique identifier of the topic within its project.
   pub fn id(&self) -> Option<String> {
      self.inner.name.rfind('/').map(|i| self.inner.name[(i + 1)..].to_string())
   }

   /// string returns the printable globally unique name for the topic.
   pub fn string(&self) -> &str {
     self.inner.name.as_str()
   }

   /// delete deletes the topic.
   pub async fn delete(&self) -> Result<(),Status>{
      self.inner.pubc.clone().delete_topic(DeleteTopicRequest {
         topic: self.inner.name.to_string()
      }, None).await.map(|v| v.into_inner())
   }

   /// exists reports whether the topic exists on the server.
   pub async fn exists(&self) -> Result<bool,Status>{
      if self.inner.name == "_deleted-topic_" {
         return Ok(false)
      }
      match self.inner.pubc.clone().get_topic(GetTopicRequest{
         topic: self.inner.name.to_string()
      }, None).await {
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
   pub async fn subscriptions(&mut self) -> Result<Vec<Subscription>,Status>{
      self.inner.pubc.clone().list_topic_subscriptions(ListTopicSubscriptionsRequest{
         topic: self.inner.name.to_string(),
         page_size: 0,
         page_token: "".to_string(),
      }, None).await.map(|v| v.into_iter().map(|sub_name| Subscription::new(sub_name, self.inner.subc.clone())).collect())
   }

   pub async fn publish(&self, message: PubsubMessage) -> Awaiter {
      let mut lock = self.inner.publisher.lock();
      if lock.is_none() {
         *lock = Some(Publisher::new(self.inner.name.clone(), self.inner.pubc.clone(),self.inner.config.clone()));
      }
      lock.as_ref().unwrap().publish(message).await
   }

   pub fn close(&self) {
      let mut lock = self.inner.publisher.lock();
      if lock.is_some() {
         if let Some(s) = &mut *lock {
            s.close();
         }
      }
      *lock = None
   }

}

impl Drop for Topic {

   fn drop(&mut self) {
      self.close() ;
   }

}