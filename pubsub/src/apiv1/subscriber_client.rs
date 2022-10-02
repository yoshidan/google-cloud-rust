use std::sync::Arc;

use google_cloud_gax::cancel::CancellationToken;
use google_cloud_gax::conn::Channel;
use google_cloud_gax::create_request;
use google_cloud_gax::grpc::Status;
use google_cloud_gax::grpc::{IntoStreamingRequest, Response, Streaming};
use google_cloud_gax::retry::{invoke, RetrySetting};
use google_cloud_googleapis::pubsub::v1::subscriber_client::SubscriberClient as InternalSubscriberClient;
use google_cloud_googleapis::pubsub::v1::{
    AcknowledgeRequest, CreateSnapshotRequest, DeleteSnapshotRequest, DeleteSubscriptionRequest, GetSnapshotRequest,
    GetSubscriptionRequest, ListSnapshotsRequest, ListSnapshotsResponse, ListSubscriptionsRequest,
    ListSubscriptionsResponse, ModifyAckDeadlineRequest, ModifyPushConfigRequest, PullRequest, PullResponse,
    SeekRequest, SeekResponse, Snapshot, StreamingPullRequest, StreamingPullResponse, Subscription,
    UpdateSnapshotRequest, UpdateSubscriptionRequest,
};

use crate::apiv1::conn_pool::ConnectionManager;

pub(crate) fn create_empty_streaming_pull_request() -> StreamingPullRequest {
    StreamingPullRequest {
        subscription: "".to_string(),
        ack_ids: vec![],
        modify_deadline_seconds: vec![],
        modify_deadline_ack_ids: vec![],
        stream_ack_deadline_seconds: 0,
        client_id: "".to_string(),
        max_outstanding_messages: 0,
        max_outstanding_bytes: 0,
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SubscriberClient {
    cm: Arc<ConnectionManager>,
}

#[allow(dead_code)]
impl SubscriberClient {
    /// create new Subscriber client
    pub fn new(cm: ConnectionManager) -> SubscriberClient {
        SubscriberClient { cm: Arc::new(cm) }
    }

    #[inline]
    fn client(&self) -> InternalSubscriberClient<Channel> {
        InternalSubscriberClient::new(self.cm.conn())
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
    #[cfg(not(feature = "trace"))]
    pub async fn create_subscription(
        &self,
        req: Subscription,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Subscription>, Status> {
        self._create_subscription(req, cancel, retry).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn create_subscription(
        &self,
        req: Subscription,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Subscription>, Status> {
        self._create_subscription(req, cancel, retry).await
    }

    #[inline(always)]
    async fn _create_subscription(
        &self,
        req: Subscription,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Subscription>, Status> {
        let name = &req.name;
        let action = || async {
            let mut client = self.client();
            let request = create_request(format!("name={}", name), req.clone());
            client.create_subscription(request).await
        };
        invoke(cancel, retry, action).await
    }

    /// updateSubscription updates an existing subscription. Note that certain properties of a
    /// subscription, such as its topic, are not modifiable.
    #[cfg(not(feature = "trace"))]
    pub async fn update_subscription(
        &self,
        req: UpdateSubscriptionRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Subscription>, Status> {
        self._update_subscription(req, cancel, retry).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn update_subscription(
        &self,
        req: UpdateSubscriptionRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Subscription>, Status> {
        self._update_subscription(req, cancel, retry).await
    }

    #[inline(always)]
    async fn _update_subscription(
        &self,
        req: UpdateSubscriptionRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Subscription>, Status> {
        let name = match &req.subscription {
            Some(s) => s.name.as_str(),
            None => "",
        };
        let action = || async {
            let mut client = self.client();
            let request = create_request(format!("subscription.name={}", name), req.clone());
            client.update_subscription(request).await
        };
        invoke(cancel, retry, action).await
    }

    /// get_subscription gets the configuration details of a subscription.
    #[cfg(not(feature = "trace"))]
    pub async fn get_subscription(
        &self,
        req: GetSubscriptionRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Subscription>, Status> {
        self._get_subscription(req, cancel, retry).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn get_subscription(
        &self,
        req: GetSubscriptionRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Subscription>, Status> {
        self._get_subscription(req, cancel, retry).await
    }

    #[inline(always)]
    async fn _get_subscription(
        &self,
        req: GetSubscriptionRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Subscription>, Status> {
        let subscription = &req.subscription;
        let action = || async {
            let mut client = self.client();
            let request = create_request(format!("subscription={}", subscription), req.clone());
            client.get_subscription(request).await
        };
        invoke(cancel, retry, action).await
    }

    /// list_subscriptions lists matching subscriptions.
    #[cfg(not(feature = "trace"))]
    pub async fn list_subscriptions(
        &self,
        req: ListSubscriptionsRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Vec<Subscription>, Status> {
        self._list_subscriptions(req, cancel, retry).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn list_subscriptions(
        &self,
        req: ListSubscriptionsRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Vec<Subscription>, Status> {
        self._list_subscriptions(req, cancel, retry).await
    }

    #[inline(always)]
    async fn _list_subscriptions(
        &self,
        mut req: ListSubscriptionsRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Vec<Subscription>, Status> {
        let project = &req.project;
        let mut all = vec![];
        //eager loading
        loop {
            let action = || async {
                let mut client = self.client();
                let request = create_request(format!("project={}", project), req.clone());
                client.list_subscriptions(request).await.map(|d| d.into_inner())
            };
            let response: ListSubscriptionsResponse = invoke(cancel.clone(), retry.clone(), action).await?;
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
    #[cfg(not(feature = "trace"))]
    pub async fn delete_subscription(
        &self,
        req: DeleteSubscriptionRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<()>, Status> {
        self._delete_subscription(req, cancel, retry).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn delete_subscription(
        &self,
        req: DeleteSubscriptionRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<()>, Status> {
        self._delete_subscription(req, cancel, retry).await
    }

    #[inline(always)]
    async fn _delete_subscription(
        &self,
        req: DeleteSubscriptionRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<()>, Status> {
        let subscription = &req.subscription;
        let action = || async {
            let mut client = self.client();
            let request = create_request(format!("subscription={}", subscription), req.clone());
            client.delete_subscription(request).await
        };
        invoke(cancel, retry, action).await
    }

    /// ModifyAckDeadline modifies the ack deadline for a specific message. This method is useful
    /// to indicate that more time is needed to process a message by the
    /// subscriber, or to make the message available for redelivery if the
    /// processing was interrupted. Note that this does not modify the
    /// subscription-level ackDeadlineSeconds used for subsequent messages.
    #[cfg(not(feature = "trace"))]
    pub async fn modify_ack_deadline(
        &self,
        req: ModifyAckDeadlineRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<()>, Status> {
        self._modify_ack_deadline(req, cancel, retry).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn modify_ack_deadline(
        &self,
        req: ModifyAckDeadlineRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<()>, Status> {
        self._modify_ack_deadline(req, cancel, retry).await
    }

    #[inline(always)]
    async fn _modify_ack_deadline(
        &self,
        req: ModifyAckDeadlineRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<()>, Status> {
        let subscription = &req.subscription;
        let action = || async {
            let mut client = self.client();
            let request = create_request(format!("subscription={}", subscription), req.clone());
            client.modify_ack_deadline(request).await
        };
        invoke(cancel, retry, action).await
    }

    /// acknowledge acknowledges the messages associated with the ack_ids in the
    /// AcknowledgeRequest. The Pub/Sub system can remove the relevant messages
    /// from the subscription.
    ///
    /// Acknowledging a message whose ack deadline has expired may succeed,
    /// but such a message may be redelivered later. Acknowledging a message more
    /// than once will not result in an error.
    #[cfg(not(feature = "trace"))]
    pub async fn acknowledge(
        &self,
        req: AcknowledgeRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<()>, Status> {
        self._acknowledge(req, cancel, retry).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn acknowledge(
        &self,
        req: AcknowledgeRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<()>, Status> {
        self._acknowledge(req, cancel, retry).await
    }

    #[inline(always)]
    async fn _acknowledge(
        &self,
        req: AcknowledgeRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<()>, Status> {
        let subscription = &req.subscription;
        let action = || async {
            let mut client = self.client();
            let request = create_request(format!("subscription={}", subscription), req.clone());
            client.acknowledge(request).await
        };
        invoke(cancel, retry, action).await
    }

    /// pull pulls messages from the server. The server may return UNAVAILABLE if
    /// there are too many concurrent pull requests pending for the given
    /// subscription.
    #[cfg(not(feature = "trace"))]
    pub async fn pull(
        &self,
        req: PullRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<PullResponse>, Status> {
        self._pull(req, cancel, retry).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn pull(
        &self,
        req: PullRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<PullResponse>, Status> {
        self._pull(req, cancel, retry).await
    }

    #[inline(always)]
    async fn _pull(
        &self,
        req: PullRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<PullResponse>, Status> {
        let subscription = &req.subscription;
        let action = || async {
            let mut client = self.client();
            let request = create_request(format!("subscription={}", subscription), req.clone());
            client.pull(request).await
        };
        invoke(cancel, retry, action).await
    }

    /// streaming_pull establishes a stream with the server, which sends messages down to the
    /// client. The client streams acknowledgements and ack deadline modifications
    /// back to the server. The server will close the stream and return the status
    /// on any error. The server may close the stream with status UNAVAILABLE to
    /// reassign server-side resources, in which case, the client should
    /// re-establish the stream. Flow control can be achieved by configuring the
    /// underlying RPC channel.
    #[cfg(not(feature = "trace"))]
    pub async fn streaming_pull(
        &self,
        req: StreamingPullRequest,
        cancel: Option<CancellationToken>,
        ping_receiver: async_channel::Receiver<bool>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Streaming<StreamingPullResponse>>, Status> {
        self._streaming_pull(req, cancel, ping_receiver, retry).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn streaming_pull(
        &self,
        req: StreamingPullRequest,
        cancel: Option<CancellationToken>,
        ping_receiver: async_channel::Receiver<bool>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Streaming<StreamingPullResponse>>, Status> {
        self._streaming_pull(req, cancel, ping_receiver, retry).await
    }

    #[inline(always)]
    async fn _streaming_pull(
        &self,
        req: StreamingPullRequest,
        cancel: Option<CancellationToken>,
        ping_receiver: async_channel::Receiver<bool>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Streaming<StreamingPullResponse>>, Status> {
        let action = || async {
            let mut client = self.client();
            let base_req = req.clone();
            let rx = ping_receiver.clone();
            let request = Box::pin(async_stream::stream! {
                yield base_req.clone();

                // ping message.
                // must be empty request
                while let Ok(_r) = rx.recv().await {
                   yield create_empty_streaming_pull_request();
                }
            });
            let mut v = request.into_streaming_request();
            let target = v.metadata_mut();
            target.append(
                "x-goog-request-params",
                format!("subscription={}", req.subscription).parse().unwrap(),
            );
            client.streaming_pull(v).await
        };
        invoke(cancel, retry, action).await
    }

    /// modify_push_config modifies the PushConfig for a specified subscription.
    ///
    /// This may be used to change a push subscription to a pull one (signified by
    /// an empty PushConfig) or vice versa, or change the endpoint URL and other
    /// attributes of a push subscription. Messages will accumulate for delivery
    /// continuously through the call regardless of changes to the PushConfig.
    #[cfg(not(feature = "trace"))]
    pub async fn modify_push_config(
        &self,
        req: ModifyPushConfigRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<()>, Status> {
        self._modify_push_config(req, cancel, retry).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn modify_push_config(
        &self,
        req: ModifyPushConfigRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<()>, Status> {
        self._modify_push_config(req, cancel, retry).await
    }

    #[inline(always)]
    async fn _modify_push_config(
        &self,
        req: ModifyPushConfigRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<()>, Status> {
        let subscription = &req.subscription;
        let action = || async {
            let mut client = self.client();
            let request = create_request(format!("subscription={}", subscription), req.clone());
            client.modify_push_config(request).await
        };
        invoke(cancel, retry, action).await
    }

    /// get_snapshot gets the configuration details of a snapshot. Snapshots are used in
    /// Seek (at https://cloud.google.com/pubsub/docs/replay-overview)
    /// operations, which allow you to manage message acknowledgments in bulk. That
    /// is, you can set the acknowledgment state of messages in an existing
    /// subscription to the state captured by a snapshot
    #[cfg(not(feature = "trace"))]
    pub async fn get_snapshot(
        &self,
        req: GetSnapshotRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Snapshot>, Status> {
        self._get_snapshot(req, cancel, retry).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn get_snapshot(
        &self,
        req: GetSnapshotRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Snapshot>, Status> {
        self._get_snapshot(req, cancel, retry).await
    }

    #[inline(always)]
    async fn _get_snapshot(
        &self,
        req: GetSnapshotRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Snapshot>, Status> {
        let snapshot = &req.snapshot;
        let action = || async {
            let mut client = self.client();
            let request = create_request(format!("snapshot={}", snapshot), req.clone());
            client.get_snapshot(request).await
        };
        invoke(cancel, retry, action).await
    }

    /// list_snapshots lists the existing snapshots. Snapshots are used in Seek (at https://cloud.google.com/pubsub/docs/replay-overview) operations, which
    /// allow you to manage message acknowledgments in bulk. That is, you can set
    /// the acknowledgment state of messages in an existing subscription to the
    /// state captured by a snapshot.
    #[cfg(not(feature = "trace"))]
    pub async fn list_snapshots(
        &self,
        req: ListSnapshotsRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Vec<Snapshot>, Status> {
        self._list_snapshots(req, cancel, retry).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn list_snapshots(
        &self,
        req: ListSnapshotsRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Vec<Snapshot>, Status> {
        self._list_snapshots(req, cancel, retry).await
    }

    #[inline(always)]
    async fn _list_snapshots(
        &self,
        mut req: ListSnapshotsRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Vec<Snapshot>, Status> {
        let project = &req.project;
        let mut all = vec![];
        //eager loading
        loop {
            let action = || async {
                let mut client = self.client();
                let request = create_request(format!("project={}", project), req.clone());
                client.list_snapshots(request).await.map(|d| d.into_inner())
            };
            let response: ListSnapshotsResponse = invoke(cancel.clone(), retry.clone(), action).await?;
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
    #[cfg(not(feature = "trace"))]
    pub async fn create_snapshot(
        &self,
        req: CreateSnapshotRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Snapshot>, Status> {
        self._create_snapshot(req, cancel, retry).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn create_snapshot(
        &self,
        req: CreateSnapshotRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Snapshot>, Status> {
        self._create_snapshot(req, cancel, retry).await
    }

    #[inline(always)]
    async fn _create_snapshot(
        &self,
        req: CreateSnapshotRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Snapshot>, Status> {
        let name = &req.name;
        let action = || async {
            let mut client = self.client();
            let request = create_request(format!("name={}", name), req.clone());
            client.create_snapshot(request).await
        };
        invoke(cancel, retry, action).await
    }

    /// update_snapshot updates an existing snapshot. Snapshots are used in
    /// Seek (at https://cloud.google.com/pubsub/docs/replay-overview)
    /// operations, which allow
    /// you to manage message acknowledgments in bulk. That is, you can set the
    /// acknowledgment state of messages in an existing subscription to the state
    /// captured by a snapshot.
    #[cfg(not(feature = "trace"))]
    pub async fn update_snapshot(
        &self,
        req: UpdateSnapshotRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Snapshot>, Status> {
        self._update_snapshot(req, cancel, retry).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn update_snapshot(
        &self,
        req: UpdateSnapshotRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Snapshot>, Status> {
        self._update_snapshot(req, cancel, retry).await
    }

    #[inline(always)]
    async fn _update_snapshot(
        &self,
        req: UpdateSnapshotRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Snapshot>, Status> {
        let name = match &req.snapshot {
            Some(v) => v.name.as_str(),
            None => "",
        };
        let action = || async {
            let mut client = self.client();
            let request = create_request(format!("snapshot.name={}", name), req.clone());
            client.update_snapshot(request).await
        };
        invoke(cancel, retry, action).await
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
    #[cfg(not(feature = "trace"))]
    pub async fn delete_snapshot(
        &self,
        req: DeleteSnapshotRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<()>, Status> {
        self._delete_snapshot(req, cancel, retry).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn delete_snapshot(
        &self,
        req: DeleteSnapshotRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<()>, Status> {
        self._delete_snapshot(req, cancel, retry).await
    }

    #[inline(always)]
    async fn _delete_snapshot(
        &self,
        req: DeleteSnapshotRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<()>, Status> {
        let name = &req.snapshot;
        let action = || async {
            let mut client = self.client();
            let request = create_request(format!("snapshot={}", name), req.clone());
            client.delete_snapshot(request).await
        };
        invoke(cancel, retry, action).await
    }

    // seek [seeks](https://cloud.google.com/pubsub/docs/replay-overview) a subscription to
    // a point back in time (with a TimeStamp) or to a saved snapshot.
    pub async fn seek(
        &self,
        req: SeekRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<SeekResponse>, Status> {
        let action = || async {
            let mut client = self.client();
            let subscription = req.subscription.clone();
            let request = create_request(format!("subscription={}", subscription), req.clone());
            client.seek(request).await
        };
        invoke(cancel, retry, action).await
    }
}
