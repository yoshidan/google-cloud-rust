use crate::apiv1::default_setting;
use google_cloud_gax::call_option::{Backoff, BackoffRetrySettings, BackoffRetryer};
use google_cloud_gax::invoke::invoke_reuse;
use google_cloud_gax::util::create_request;
use google_cloud_googleapis::pubsub::v1::subscriber_client::SubscriberClient as InternalSubscriberClient;
use google_cloud_googleapis::pubsub::v1::{
    AcknowledgeRequest, CreateSnapshotRequest, DeleteSnapshotRequest, DeleteSubscriptionRequest,
    DeleteTopicRequest, DetachSubscriptionRequest, DetachSubscriptionResponse, GetSnapshotRequest,
    GetSubscriptionRequest, GetTopicRequest, ListSnapshotsRequest, ListSubscriptionsRequest,
    ListSubscriptionsResponse, ListTopicSnapshotsRequest, ListTopicSubscriptionsRequest,
    ListTopicsRequest, ModifyAckDeadlineRequest, ModifyPushConfigRequest, PullRequest,
    PullResponse, Snapshot, StreamingPullRequest, StreamingPullResponse, Subscription, Topic,
    UpdateSnapshotRequest, UpdateSubscriptionRequest, UpdateTopicRequest,
};
use google_cloud_googleapis::{Code, Status};
use google_cloud_grpc::conn::Channel;
use tonic::{Response, Streaming};

#[derive(Clone)]
pub struct SubscriberClient {
    inner: InternalSubscriberClient<Channel>,
}

impl SubscriberClient {
    /// create new Subscriber client
    pub fn new(inner: InternalSubscriberClient<Channel>) -> SubscriberClient {
        SubscriberClient { inner }
    }

    /// merge call setting
    fn get_call_setting(call_setting: Option<BackoffRetrySettings>) -> BackoffRetrySettings {
        match call_setting {
            Some(s) => s,
            None => default_setting(),
        }
    }

    /// create_subscription creates a subscription to a given topic. See the [resource name rules]
    /// (https://cloud.google.com/pubsub/docs/admin#resource_names (at https://cloud.google.com/pubsub/docs/admin#resource_names)).
    /// If the subscription already exists, returns ALREADY_EXISTS.
    /// If the corresponding topic doesn’t exist, returns NOT_FOUND.
    ///
    /// If the name is not provided in the request, the server will assign a random
    /// name for this subscription on the same project as the topic, conforming
    /// to the [resource name format]
    /// (https://cloud.google.com/pubsub/docs/admin#resource_names (at https://cloud.google.com/pubsub/docs/admin#resource_names)). The generated
    /// name is populated in the returned Subscription object. Note that for REST
    /// API requests, you must specify a name in the request.
    pub async fn create_subscription(
        &mut self,
        req: Subscription,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<Subscription>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let name = &req.name;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("name={}", name), req.clone());
                client
                    .create_subscription(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// updateSubscription updates an existing subscription. Note that certain properties of a
    /// subscription, such as its topic, are not modifiable.
    pub async fn update_subscription(
        &mut self,
        req: UpdateSubscriptionRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<Topic>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let name = &req.subscription?.name;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("subscription.name={}", name), req.clone());
                client
                    .update_subscription(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// get_subscription gets the configuration details of a subscription.
    pub async fn get_subscription(
        &mut self,
        req: GetSubscriptionRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<Subscription>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let subscription = &req.subscription;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("subscription={}", subscription), req.clone());
                client
                    .get_subscription(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// list_subscriptions lists matching subscriptions.
    pub async fn list_subscriptions(
        &mut self,
        mut req: ListSubscriptionsRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Vec<Subscription>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let project = &req.project;
        let mut all = vec![];
        //eager loading
        loop {
            let response = invoke_reuse(
                |client| async {
                    let request = create_request(format!("project={}", project), req.clone());
                    client
                        .list_subscriptions(request)
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

    /// delete_subscription deletes an existing subscription. All messages retained in the subscription
    /// are immediately dropped. Calls to Pull after deletion will return
    /// NOT_FOUND. After a subscription is deleted, a new one may be created with
    /// the same name, but the new one has no association with the old
    /// subscription or its topic unless the same topic is specified.
    pub async fn delete_subscription(
        &mut self,
        req: DeleteSubscriptionRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<()>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let subscription = &req.subscription;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("subscription={}", subscription), req.clone());
                client
                    .delete_subscription(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// ModifyAckDeadline modifies the ack deadline for a specific message. This method is useful
    /// to indicate that more time is needed to process a message by the
    /// subscriber, or to make the message available for redelivery if the
    /// processing was interrupted. Note that this does not modify the
    /// subscription-level ackDeadlineSeconds used for subsequent messages.
    pub async fn modify_ack_deadline(
        &mut self,
        req: ModifyAckDeadlineRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<()>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let subscription = &req.subscription;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("subscription={}", subscription), req.clone());
                client
                    .modify_ack_deadline(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// acknowledge acknowledges the messages associated with the ack_ids in the
    /// AcknowledgeRequest. The Pub/Sub system can remove the relevant messages
    /// from the subscription.
    ///
    /// Acknowledging a message whose ack deadline has expired may succeed,
    /// but such a message may be redelivered later. Acknowledging a message more
    /// than once will not result in an error.
    pub async fn acknowledge(
        &mut self,
        req: AcknowledgeRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<()>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let subscription = &req.subscription;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("subscription={}", subscription), req.clone());
                client
                    .acknowledge(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// pull pulls messages from the server. The server may return UNAVAILABLE if
    /// there are too many concurrent pull requests pending for the given
    /// subscription.
    pub async fn pull(
        &mut self,
        req: PullRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<PullResponse>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let subscription = &req.subscription;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("subscription={}", subscription), req.clone());
                client.pull(request).await.map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// streaming_pull establishes a stream with the server, which sends messages down to the
    /// client. The client streams acknowledgements and ack deadline modifications
    /// back to the server. The server will close the stream and return the status
    /// on any error. The server may close the stream with status UNAVAILABLE to
    /// reassign server-side resources, in which case, the client should
    /// re-establish the stream. Flow control can be achieved by configuring the
    /// underlying RPC channel.
    pub async fn streaming_pull(
        &mut self,
        req: StreamingPullRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<Streaming<StreamingPullResponse>>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let subscription = &req.subscription;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("subscription={}", subscription), req.clone());
                client
                    .streaming_pull(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// modify_push_config modifies the PushConfig for a specified subscription.
    ///
    /// This may be used to change a push subscription to a pull one (signified by
    /// an empty PushConfig) or vice versa, or change the endpoint URL and other
    /// attributes of a push subscription. Messages will accumulate for delivery
    /// continuously through the call regardless of changes to the PushConfig.
    pub async fn modify_push_config(
        &mut self,
        req: ModifyPushConfigRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<()>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let subscription = &req.subscription;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("subscription={}", subscription), req.clone());
                client
                    .modify_push_config(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// get_snapshot gets the configuration details of a snapshot. Snapshots are used in
    /// Seek (at https://cloud.google.com/pubsub/docs/replay-overview)
    /// operations, which allow you to manage message acknowledgments in bulk. That
    /// is, you can set the acknowledgment state of messages in an existing
    /// subscription to the state captured by a snapshot
    pub async fn get_snapshot(
        &mut self,
        req: GetSnapshotRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<Snapshot>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let snapshot = &req.snapshot;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("snapshot={}", snapshot), req.clone());
                client
                    .get_subscription(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// list_snapshots lists the existing snapshots. Snapshots are used in Seek (at https://cloud.google.com/pubsub/docs/replay-overview) operations, which
    /// allow you to manage message acknowledgments in bulk. That is, you can set
    /// the acknowledgment state of messages in an existing subscription to the
    /// state captured by a snapshot.
    pub async fn list_snapshots(
        &mut self,
        mut req: ListSnapshotsRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Vec<Snapshot>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let project = &req.project;
        let mut all = vec![];
        //eager loading
        loop {
            let response = invoke_reuse(
                |client| async {
                    let request = create_request(format!("project={}", project), req.clone());
                    client
                        .list_snapshots(request)
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

    /// create_snapshot creates a snapshot from the requested subscription. Snapshots are used in
    /// Seek (at https://cloud.google.com/pubsub/docs/replay-overview) operations,
    /// which allow you to manage message acknowledgments in bulk. That is, you can
    /// set the acknowledgment state of messages in an existing subscription to the
    /// state captured by a snapshot.
    /// If the snapshot already exists, returns ALREADY_EXISTS.
    /// If the requested subscription doesn’t exist, returns NOT_FOUND.
    /// If the backlog in the subscription is too old – and the resulting snapshot
    /// would expire in less than 1 hour – then FAILED_PRECONDITION is returned.
    /// See also the Snapshot.expire_time field. If the name is not provided in
    /// the request, the server will assign a random
    /// name for this snapshot on the same project as the subscription, conforming
    /// to the [resource name format]
    /// (https://cloud.google.com/pubsub/docs/admin#resource_names (at https://cloud.google.com/pubsub/docs/admin#resource_names)). The
    /// generated name is populated in the returned Snapshot object. Note that for
    /// REST API requests, you must specify a name in the request.
    pub async fn create_snapshot(
        &mut self,
        req: CreateSnapshotRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<Snapshot>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let name = &req.name;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("name={}", name), req.clone());
                client
                    .create_snapshot(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// update_snapshot updates an existing snapshot. Snapshots are used in
    /// Seek (at https://cloud.google.com/pubsub/docs/replay-overview)
    /// operations, which allow
    /// you to manage message acknowledgments in bulk. That is, you can set the
    /// acknowledgment state of messages in an existing subscription to the state
    /// captured by a snapshot.
    pub async fn update_snapshot(
        &mut self,
        req: UpdateSnapshotRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<Snapshot>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let name = &req.snapshot?.name;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("snapshot.name={}", name), req.clone());
                client
                    .update_snapshot(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// delete_snapshot removes an existing snapshot. Snapshots are used in [Seek]
    /// (https://cloud.google.com/pubsub/docs/replay-overview (at https://cloud.google.com/pubsub/docs/replay-overview)) operations, which
    /// allow you to manage message acknowledgments in bulk. That is, you can set
    /// the acknowledgment state of messages in an existing subscription to the
    /// state captured by a snapshot.
    /// When the snapshot is deleted, all messages retained in the snapshot
    /// are immediately dropped. After a snapshot is deleted, a new one may be
    /// created with the same name, but the new one has no association with the old
    /// snapshot or its subscription, unless the same subscription is specified.
    pub async fn delete_snapshot(
        &mut self,
        req: DeleteSnapshotRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<()>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let name = &req.snapshot;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("snapshot={}", name), req.clone());
                client
                    .delete_snapshot(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }
}