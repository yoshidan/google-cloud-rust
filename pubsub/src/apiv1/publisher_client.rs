use crate::apiv1::default_setting;
use google_cloud_gax::call_option::{Backoff, BackoffRetrySettings, BackoffRetryer};
use google_cloud_gax::invoke::invoke_reuse;
use google_cloud_gax::util::create_request;
use google_cloud_googleapis::pubsub::v1::publisher_client::PublisherClient as InternalPublisherClient;
use google_cloud_googleapis::pubsub::v1::{DeleteTopicRequest, DetachSubscriptionRequest, DetachSubscriptionResponse, GetTopicRequest, ListTopicSnapshotsRequest, ListTopicSubscriptionsRequest, ListTopicsRequest, Subscription, Topic, UpdateTopicRequest, PublishRequest, PublishResponse};
use google_cloud_googleapis::{Code, Status};
use google_cloud_grpc::conn::Channel;
use tonic::Response;

#[derive(Clone)]
pub struct PublisherClient {
    inner: InternalPublisherClient<Channel>,
}

impl PublisherClient {
    /// create new publisher client
    pub fn new(inner: InternalPublisherClient<Channel>) -> PublisherClient {
        PublisherClient { inner }
    }

    /// merge call setting
    fn get_call_setting(call_setting: Option<BackoffRetrySettings>) -> BackoffRetrySettings {
        match call_setting {
            Some(s) => s,
            None => default_setting(),
        }
    }

    /// create_topic creates the given topic with the given name. See the [resource name rules]
    pub async fn create_topic(
        &mut self,
        req: Topic,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<Topic>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let name = &req.name;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("name={}", name), req.clone());
                client
                    .create_topic(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// update_topic updates an existing topic. Note that certain properties of a
    /// topic are not modifiable.
    pub async fn update_topic(
        &mut self,
        req: UpdateTopicRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<Topic>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let name = &req.topic?.name;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("name={}", name), req.clone());
                client
                    .update_topic(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// publish adds one or more messages to the topic. Returns NOT_FOUND if the topic does not exist.
    pub async fn publish(
        &mut self,
        req: PublishRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<PublishResponse>, Status> {
        let mut setting = match opt {
            Some(opt) => opt,
            None => BackoffRetrySettings {
                retryer: BackoffRetryer {
                    backoff: Backoff::default(),
                    codes: vec![
                        Code::Unavailable,
                        Code::Unknown,
                        Code::Aborted,
                        Code::Internal,
                        Code::ResourceExhausted,
                        Code::DeadlineExceeded,
                        Code::Cancelled,
                    ],
                },
            },
        };
        let name = &req.topic?.name;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("name={}", name), req.clone());
                client
                    .publish(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// get_topic gets the configuration of a topic.
    pub async fn get_topic(
        &mut self,
        req: GetTopicRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<Topic>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let topic = &req.topic;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("topic={}", topic), req.clone());
                client
                    .get_topic(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// list_topics lists matching topics.
    pub async fn list_topics(
        &mut self,
        mut req: ListTopicsRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Vec<Topic>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let project = &req.project;
        let mut all = vec![];
        //eager loading
        loop {
            let response = invoke_reuse(
                |client| async {
                    let request = create_request(format!("project={}", project), req.clone());
                    client
                        .list_topics(request)
                        .await
                        .map_err(|e| (Status::from(e), client))
                        .map(|d| d.into_inner())
                },
                &mut self.inner,
                &mut setting,
            )
            .await?;
            all.extend(response.topics.into_iter());
            if response.next_page_token.is_empty() {
                return Ok(all);
            }
            req.page_token = response.next_page_token;
        }
    }

    /// list_topics lists matching topics.
    pub async fn list_topic_subscriptions(
        &mut self,
        mut req: ListTopicSubscriptionsRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Vec<String>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let topic = &req.topic;
        let mut all = vec![];
        //eager loading
        loop {
            let response = invoke_reuse(
                |client| async {
                    let request = create_request(format!("topic={}", topic), req.clone());
                    client
                        .list_topic_subscriptions(request)
                        .await
                        .map_err(|e| (Status::from(e), client))
                        .map(|d| d.into_inner())
                },
                &mut self.inner,
                &mut setting,
            )
            .await?;
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
        &mut self,
        mut req: ListTopicSnapshotsRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Vec<String>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let topic = &req.topic;
        let mut all = vec![];
        //eager loading
        loop {
            let response = invoke_reuse(
                |client| async {
                    let request = create_request(format!("topic={}", topic), req.clone());
                    client
                        .list_topic_snapshots(request)
                        .await
                        .map_err(|e| (Status::from(e), client))
                        .map(|d| d.into_inner())
                },
                &mut self.inner,
                &mut setting,
            )
            .await?;
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
        &mut self,
        req: DeleteTopicRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<()>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let topic = &req.topic;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("topic={}", topic), req.clone());
                client
                    .delete_topic(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// detach_subscription detaches a subscription from this topic. All messages retained in the
    /// subscription are dropped. Subsequent Pull and StreamingPull requests
    /// will return FAILED_PRECONDITION. If the subscription is a push
    /// subscription, pushes to the endpoint will stop.
    pub async fn detach_subscription(
        &mut self,
        req: DetachSubscriptionRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<DetachSubscriptionResponse>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let subscription = &req.subscription;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("subscription={}", subscription), req.clone());
                client
                    .detach_subscription(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }
}
