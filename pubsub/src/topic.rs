use std::sync::Arc;
use parking_lot::RwLock;
use tokio::sync::oneshot::channel;
use google_cloud_googleapis::Code::NotFound;
use google_cloud_googleapis::pubsub::v1::{DeleteTopicRequest, GetTopicRequest, ListTopicSubscriptionsRequest, PubsubMessage};
use google_cloud_googleapis::spanner::admin::database::v1::backup::State;
use google_cloud_googleapis::Status;
use crate::apiv1::publisher_client::PublisherClient;
use crate::apiv1::subscriber_client::SubscriberClient;
use crate::publisher::{Awaiter, Publisher};
use crate::subscription::Subscription;

/// Topic is a reference to a PubSub topic.
pub struct Topic {
   name: String,
   stopped: RwLock<bool>,
   pubc: PublisherClient,
   subc: SubscriberClient,
   scheduler: Publisher
}

impl Topic {

   fn new(name: String,
          pubc: PublisherClient,
          subc: SubscriberClient,
          scheduler: Publisher) -> Self {
      Self {
         name,
         stopped: RwLock::new(false),
         pubc,
         subc,
         scheduler
      }
   }

   /// id returns the unique identifier of the topic within its project.
   fn id(&self) -> Option<String> {
      self.name.rfind('/').map(|i| self.name[(i + 1)..].to_string())
   }

   // string returns the printable globally unique name for the topic.
   fn string(&self) -> &str {
     self.name.as_str()
   }

   /// delete deletes the topic.
   async fn delete(&mut self) -> Result<(),Status>{
      self.pubc.delete_topic(DeleteTopicRequest {
         topic: self.name.to_string()
      }, None).await.map(|v| v.into_inner())
   }

   /// exists reports whether the topic exists on the server.
   async fn exists(&mut self) -> Result<bool,Status>{
      if self.name == "_deleted-topic_" {
         return Ok(false)
      }
      match self.pubc.get_topic(GetTopicRequest{
         topic: self.name.to_string()
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
   async fn subscriptions(&mut self) -> Result<Vec<Subscription>,Status>{
      self.pubc.list_topic_subscriptions(ListTopicSubscriptionsRequest{
         topic: self.name.to_string(),
         page_size: 0,
         page_token: "".to_string(),
      }, None).await.map(|v| v.into_iter().map(|sub_name| Subscription::new(sub_name, self.subc.clone())).collect())
   }

   async fn publish(&mut self, message: PubsubMessage) -> Awaiter {
      if *self.stopped.read() {
         let (sender,receiver) = channel();
         sender.send(Err(Status::new(tonic::Status::unavailable("stopped"))));
         return Awaiter::new(receiver);
      }
      return self.scheduler.publish(message).await;
   }

   pub fn stop(&mut self) {
      if self.stop_if_needed() {
         self.scheduler.stop();
      }
   }

   fn stop_if_needed(&self) -> bool {
      let mut w = self.stopped.write();
      if *w {
         return false
      }
      *w = true;
      return true;
   }
}