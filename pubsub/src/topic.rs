use std::sync::Arc;
use google_cloud_googleapis::Code::NotFound;
use google_cloud_googleapis::pubsub::v1::{DeleteTopicRequest, GetTopicRequest, ListTopicSubscriptionsRequest};
use google_cloud_googleapis::spanner::admin::database::v1::backup::State;
use google_cloud_googleapis::Status;
use crate::apiv1::publisher_client::PublisherClient;
use crate::apiv1::subscriber_client::SubscriberClient;
use crate::publish_scheduler::PublishScheduler;
use crate::subscription::Subscription;

/// Topic is a reference to a PubSub topic.
pub struct Topic {
   name: String,
   stopped: bool,
   pubc: Arc<PublisherClient>,
   subc: Arc<SubscriberClient>,
   scheduler: Arc<PublishScheduler>
}

impl Topic {

   fn new(name: String,
          pubc: Arc<PublisherClient>,
          subc: Arc<SubscriberClient>,
          scheduler: Arc<PublishScheduler>) -> Self {
      Self {
         name,
         stopped: false,
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
}