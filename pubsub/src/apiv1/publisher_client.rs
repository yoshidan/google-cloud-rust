use std::sync::Arc;

use crate::apiv1::conn_pool::ConnectionManager;
use google_cloud_gax::conn::Channel;
use google_cloud_gax::create_request;
use google_cloud_gax::retry::{invoke, RetrySetting};
use google_cloud_gax::status::{Code, Status};
use google_cloud_googleapis::pubsub::v1::publisher_client::PublisherClient as InternalPublisherClient;
use google_cloud_googleapis::pubsub::v1::{
    DeleteTopicRequest, DetachSubscriptionRequest, DetachSubscriptionResponse, GetTopicRequest,
    ListTopicSnapshotsRequest, ListTopicSubscriptionsRequest, ListTopicsRequest, PublishRequest,
    PublishResponse, Topic, UpdateTopicRequest,
};
use tokio_util::sync::CancellationToken;
use tonic::Response;

#[derive(Clone)]
pub(crate) struct PublisherClient {
    cm: Arc<ConnectionManager>,
}

impl PublisherClient {
    /// create new publisher client
    pub fn new(cm: ConnectionManager) -> PublisherClient {
        PublisherClient { cm: Arc::new(cm) }
    }

    fn client(&self) -> InternalPublisherClient<Channel> {
        InternalPublisherClient::new(self.cm.conn())
    }

    /// create_topic creates the given topic with the given name. See the [resource name rules]
    pub async fn create_topic(
        &self,
        ctx: CancellationToken,
        req: Topic,
        opt: Option<RetrySetting>,
    ) -> Result<Response<Topic>, Status> {
        let name = &req.name;
        let action = || async {
            let mut client = self.client();
            let request = create_request(format!("name={}", name), req.clone());
            client.create_topic(request).await.map_err(|e| e.into())
        };
        invoke(ctx, opt, action).await
    }

    /// update_topic updates an existing topic. Note that certain properties of a
    /// topic are not modifiable.
    pub async fn update_topic(
        &self,
        ctx: CancellationToken,
        req: UpdateTopicRequest,
        opt: Option<RetrySetting>,
    ) -> Result<Response<Topic>, Status> {
        let name = match &req.topic {
            Some(t) => t.name.as_str(),
            None => "",
        };
        let action = || async {
            let mut client = self.client();
            let request = create_request(format!("name={}", name), req.clone());
            client.update_topic(request).await.map_err(|e| e.into())
        };
        invoke(ctx, opt, action).await
    }

    /// publish adds one or more messages to the topic. Returns NOT_FOUND if the topic does not exist.
    pub async fn publish(
        &self,
        ctx: CancellationToken,
        req: PublishRequest,
        opt: Option<RetrySetting>,
    ) -> Result<Response<PublishResponse>, Status> {
        let setting = match opt {
            Some(opt) => opt,
            None => {
                let mut default = RetrySetting::default();
                default.codes = vec![
                    Code::Unavailable,
                    Code::Unknown,
                    Code::Aborted,
                    Code::Cancelled,
                    Code::DeadlineExceeded,
                    Code::ResourceExhausted,
                    Code::Internal,
                ];
                default
            }
        };
        let name = &req.topic;
        let action = || async {
            let mut client = self.client();
            let request = create_request(format!("name={}", name), req.clone());
            client.publish(request).await.map_err(|e| e.into())
        };
        invoke(ctx, Some(setting), action).await
    }

    /// get_topic gets the configuration of a topic.
    pub async fn get_topic(
        &self,
        ctx: CancellationToken,
        req: GetTopicRequest,
        opt: Option<RetrySetting>,
    ) -> Result<Response<Topic>, Status> {
        let topic = &req.topic;
        let action = || async {
            let mut client = self.client();
            let request = create_request(format!("topic={}", topic), req.clone());
            client.get_topic(request).await.map_err(|e| e.into())
        };
        invoke(ctx, opt, action).await
    }

    /// list_topics lists matching topics.
    pub async fn list_topics(
        &self,
        ctx: CancellationToken,
        mut req: ListTopicsRequest,
        opt: Option<RetrySetting>,
    ) -> Result<Vec<Topic>, Status> {
        let project = &req.project;
        let mut all = vec![];
        //eager loading
        loop {
            let action = || async {
                let mut client = self.client();
                let request = create_request(format!("project={}", project), req.clone());
                client
                    .list_topics(request)
                    .await
                    .map_err(|e| Status::from(e))
                    .map(|d| d.into_inner())
            };
            let response = invoke(ctx.clone(), opt.clone(), action).await?;
            all.extend(response.topics.into_iter());
            if response.next_page_token.is_empty() {
                return Ok(all);
            }
            req.page_token = response.next_page_token;
        }
    }

    /// list_topics lists matching topics.
    pub async fn list_topic_subscriptions(
        &self,
        ctx: CancellationToken,
        mut req: ListTopicSubscriptionsRequest,
        opt: Option<RetrySetting>,
    ) -> Result<Vec<String>, Status> {
        let topic = &req.topic;
        let mut all = vec![];
        //eager loading
        loop {
            let action = || async {
                let mut client = self.client();
                let request = create_request(format!("topic={}", topic), req.clone());
                client
                    .list_topic_subscriptions(request)
                    .await
                    .map_err(|e| Status::from(e))
                    .map(|d| d.into_inner())
            };
            let response = invoke(ctx.clone(), opt.clone(), action).await?;
            all.extend(response.subscriptions.into_iter());
            if response.next_page_token.is_empty() {
                return Ok(all);
            }
            req.page_token = response.next_page_token;
        }
    }

    /// list_topic_snapshots lists the names of the snapshots on this topic. Snapshots are used in
    /// Seek (at https://cloud.google.com/pubsub/docs/replay-overview) operations,
    /// which allow you to manage message acknowledgments in bulk. That is, you can
    /// set the acknowledgment state of messages in an existing subscription to the
    /// state captured by a snapshot.
    pub async fn list_topic_snapshots(
        &self,
        ctx: CancellationToken,
        mut req: ListTopicSnapshotsRequest,
        opt: Option<RetrySetting>,
    ) -> Result<Vec<String>, Status> {
        let topic = &req.topic;
        let mut all = vec![];
        //eager loading
        loop {
            let action = || async {
                let mut client = self.client();
                let request = create_request(format!("topic={}", topic), req.clone());
                client
                    .list_topic_snapshots(request)
                    .await
                    .map_err(|e| Status::from(e))
                    .map(|d| d.into_inner())
            };
            let response = invoke(ctx.clone(), opt.clone(), action).await?;
            all.extend(response.snapshots.into_iter());
            if response.next_page_token.is_empty() {
                return Ok(all);
            }
            req.page_token = response.next_page_token;
        }
    }

    /// delete_topic deletes the topic with the given name. Returns NOT_FOUND if the topic
    /// does not exist. After a topic is deleted, a new topic may be created with
    /// the same name; this is an entirely new topic with none of the old
    /// configuration or subscriptions. Existing subscriptions to this topic are
    /// not deleted, but their topic field is set to _deleted-topic_.
    pub async fn delete_topic(
        &self,
        ctx: CancellationToken,
        req: DeleteTopicRequest,
        opt: Option<RetrySetting>,
    ) -> Result<Response<()>, Status> {
        let topic = &req.topic;
        let action = || async {
            let mut client = self.client();
            let request = create_request(format!("topic={}", topic), req.clone());
            client.delete_topic(request).await.map_err(|e| e.into())
        };
        invoke(ctx, opt, action).await
    }

    /// detach_subscription detaches a subscription from this topic. All messages retained in the
    /// subscription are dropped. Subsequent Pull and StreamingPull requests
    /// will return FAILED_PRECONDITION. If the subscription is a push
    /// subscription, pushes to the endpoint will stop.
    pub async fn detach_subscription(
        &self,
        ctx: CancellationToken,
        req: DetachSubscriptionRequest,
        opt: Option<RetrySetting>,
    ) -> Result<Response<DetachSubscriptionResponse>, Status> {
        let subscription = &req.subscription;
        let action = || async {
            let mut client = self.client();
            let request = create_request(format!("subscription={}", subscription), req.clone());
            client
                .detach_subscription(request)
                .await
                .map_err(|e| e.into())
        };
        invoke(ctx, opt, action).await
    }
}
