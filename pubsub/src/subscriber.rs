use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::{oneshot, Mutex};
use tokio::task::JoinHandle;
use tokio::time::sleep;

use google_cloud_gax::grpc::{Code, Status, Streaming};
use google_cloud_gax::retry::RetrySetting;
use google_cloud_googleapis::pubsub::v1::{
    AcknowledgeRequest, ModifyAckDeadlineRequest, PubsubMessage, ReceivedMessage as InternalReceivedMessage,
    StreamingPullRequest, StreamingPullResponse,
};

use crate::apiv1::default_retry_setting;
use crate::apiv1::subscriber_client::{create_empty_streaming_pull_request, SubscriberClient};

#[derive(Debug, Clone)]
pub struct ReceivedMessage {
    pub message: PubsubMessage,
    ack_id: String,
    subscription: String,
    subscriber_client: SubscriberClient,
    delivery_attempt: Option<usize>,
}

impl ReceivedMessage {
    pub(crate) fn new(
        subscription: String,
        subc: SubscriberClient,
        message: PubsubMessage,
        ack_id: String,
        delivery_attempt: Option<usize>,
    ) -> Self {
        Self {
            message,
            ack_id,
            subscription,
            subscriber_client: subc,
            delivery_attempt,
        }
    }

    pub fn ack_id(&self) -> &str {
        self.ack_id.as_str()
    }

    pub async fn ack(&self) -> Result<(), Status> {
        ack(
            &self.subscriber_client,
            self.subscription.to_string(),
            vec![self.ack_id.to_string()],
        )
        .await
    }

    pub async fn nack(&self) -> Result<(), Status> {
        nack(
            &self.subscriber_client,
            self.subscription.to_string(),
            vec![self.ack_id.to_string()],
        )
        .await
    }

    pub async fn modify_ack_deadline(&self, ack_deadline_seconds: i32) -> Result<(), Status> {
        modify_ack_deadline(
            &self.subscriber_client,
            self.subscription.to_string(),
            vec![self.ack_id.to_string()],
            ack_deadline_seconds,
        )
        .await
    }

    /// The approximate number of times that Cloud Pub/Sub has attempted to deliver
    /// the associated message to a subscriber.
    ///
    /// The returned value, if present, will be greater than zero.
    ///
    /// For more information refer to the
    /// [protobuf definition](https://github.com/googleapis/googleapis/blob/3c7c76fb63d0f511cdb8c3c1cbc157315f6fbfd3/google/pubsub/v1/pubsub.proto#L1099-L1115).
    pub fn delivery_attempt(&self) -> Option<usize> {
        self.delivery_attempt
    }
}

#[derive(Debug, Clone)]
pub struct SubscriberConfig {
    /// ping interval for Bi Directional Streaming
    pub ping_interval: Duration,
    pub retry_setting: Option<RetrySetting>,
    /// It is important for exactly_once_delivery
    /// The ack deadline to use for the stream. This must be provided in
    /// the first request on the stream, but it can also be updated on subsequent
    /// requests from client to server. The minimum deadline you can specify is 10
    /// seconds. The maximum deadline you can specify is 600 seconds (10 minutes).
    pub stream_ack_deadline_seconds: i32,
    /// Flow control settings for the maximum number of outstanding messages. When
    /// there are `max_outstanding_messages` or more currently sent to the
    /// streaming pull client that have not yet been acked or nacked, the server
    /// stops sending more messages. The sending of messages resumes once the
    /// number of outstanding messages is less than this value. If the value is
    /// <= 0, there is no limit to the number of outstanding messages. This
    /// property can only be set on the initial StreamingPullRequest. If it is set
    /// on a subsequent request, the stream will be aborted with status
    /// `INVALID_ARGUMENT`.
    pub max_outstanding_messages: i64,
    pub max_outstanding_bytes: i64,
}

impl Default for SubscriberConfig {
    fn default() -> Self {
        // Default retry setting with Cancelled code
        let mut retry = default_retry_setting();
        retry.codes.push(Code::Cancelled);

        Self {
            ping_interval: Duration::from_secs(10),
            retry_setting: Some(retry),
            stream_ack_deadline_seconds: 60,
            max_outstanding_messages: 50,
            max_outstanding_bytes: 1000 * 1000 * 1000,
        }
    }
}

struct UnprocessedMessages{
    tx: Option<oneshot::Sender<Option<Vec<String>>>>,
    ack_ids: Option<Vec<String>>
}

impl UnprocessedMessages {
    fn new(tx: oneshot::Sender<Option<Vec<String>>>) -> Self {
        Self {
            tx: Some(tx),
            ack_ids: Some(vec![])
        }
    }
}

impl Deref for UnprocessedMessages {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        self.ack_ids.as_ref().unwrap()
    }
}

impl DerefMut for UnprocessedMessages {

    fn deref_mut(&mut self) -> &mut Vec<String> {
        self.ack_ids.as_mut().unwrap()
    }
}

impl Drop for UnprocessedMessages {
    fn drop(&mut self) {
        if let Some(tx) = self.tx.take() {
            let _ = tx.send(self.ack_ids.take());
        }
    }
}

#[derive(Debug)]
pub(crate) struct Subscriber {
    client: SubscriberClient,
    subscription: String,
    task_to_ping: JoinHandle<()>,
    task_to_receive: JoinHandle<()>,
    /// Ack id list of unprocessed messages.
    unprocessed_messages_receiver: Option<oneshot::Receiver<Option<Vec<String>>>>,
}

impl Drop for Subscriber {
    fn drop(&mut self) {
        if let Some(rx) = self.unprocessed_messages_receiver.take() {
            tracing::warn!("Subscriber is not disposed. Call dispose() to properly clean up resources.");
            tokio::spawn(async move {
                if let Ok(Some(messages)) = rx.await {
                    if !messages.is_empty() {
                        tracing::debug!("nack {} unprocessed messages", messages.len());
                        // We cannot do anything if nack fails here.
                        let _ = nack(&self.client, self.subscription.clone(), messages).await;
                    }
                }
            });
        }
    }
}

impl Subscriber {
    pub fn spawn(
        subscription: String,
        client: SubscriberClient,
        queue: async_channel::Sender<ReceivedMessage>,
        config: SubscriberConfig,
    ) -> Self {
        let (ping_sender, ping_receiver) = async_channel::unbounded();

        // Build task to ping
        let task_to_ping = async move {
            loop {
                _ = sleep(config.ping_interval);
                let _ = ping_sender.send(true).await;
            }
        };

        let subscription_clone = subscription.clone();
        let client_clone = client.clone();

        // Build task to receive
        let (tx, rx) = oneshot::channel();
        let task_to_receive = async move {
            tracing::trace!("start subscriber: {}", subscription);

            let retryable_codes = match &config.retry_setting {
                Some(v) => v.codes.clone(),
                None => default_retry_setting().codes,
            };

            let mut unprocessed_messages = UnprocessedMessages::new(tx);
            loop {
                let mut request = create_empty_streaming_pull_request();
                request.subscription = subscription.to_string();
                request.stream_ack_deadline_seconds = config.stream_ack_deadline_seconds;
                request.max_outstanding_messages = config.max_outstanding_messages;
                request.max_outstanding_bytes = config.max_outstanding_bytes;

                tracing::trace!("start streaming: {}", subscription);

                let response = {
                    Self::receive(
                        client.clone(),
                        request,
                        ping_receiver.clone(),
                        config.clone(),
                        queue.clone(),
                        &mut unprocessed_messages,
                    )
                    .await
                };

                if let Err(e) = response {
                    if retryable_codes.contains(&e.code()) {
                        tracing::warn!("failed to receive message: will reconnect {:?} : {}", e, subscription);
                        continue;
                    } else {
                        tracing::error!("failed to receive message: will stop {:?} : {}", e, subscription);
                        break;
                    }
                } else {
                    tracing::debug!("stopped to receive message: {}", subscription);
                    break
                }
            }
            tracing::trace!("stop subscriber: {}", subscription);
        };

        // receiver can detect closed when all the senders are closed.
        if !queue.is_closed() {
            queue.close();
        }

        Self {
            client: client_clone,
            subscription: subscription_clone,
            task_to_ping: tokio::spawn(task_to_ping),
            task_to_receive: tokio::spawn(task_to_receive),
            unprocessed_messages_receiver: Some(rx),
        }
    }

    async fn receive(
        client: SubscriberClient,
        request: StreamingPullRequest,
        ping_receiver: async_channel::Receiver<bool>,
        config: SubscriberConfig,
        queue: async_channel::Sender<ReceivedMessage>,
        unprocessed_messages: &mut Vec<String>,
    ) -> Result<(), Status> {
        let subscription = request.subscription.to_string();

        // Call the streaming_pull method with the provided request and ping_receiver
        let response = client
            .streaming_pull(request, ping_receiver.clone(), config.retry_setting.clone())
            .await?;
        let mut stream = response.into_inner();

        // Process the stream
        loop {
            let message = stream.message().await?;
            let messages = match message {
                Some(m) => m.received_messages,
                None => return Ok(()),
            };

            let mut msgs = vec![];
            for received_message in messages {
                if let Some(message) = received_message.message {
                    let id = message.message_id.clone();
                    tracing::debug!("message received: msg_id={id}");
                    let msg = ReceivedMessage::new(
                        subscription.clone(),
                        client.clone(),
                        message,
                        received_message.ack_id.clone(),
                        (received_message.delivery_attempt > 0).then_some(received_message.delivery_attempt as usize),
                    );
                    unprocessed_messages.push(msg.ack_id.clone());
                    msgs.push(msg);
                }
            }

            for msg in msgs.drain(..) {
                let ack_id = msg.ack_id.clone();
                if queue.send(msg).await.is_ok() {
                    unprocessed_messages.retain(|e| *e != ack_id);
                }else {
                    if let Err(err) = nack(&client, subscription.clone(), vec![ack_id]).await {
                        tracing::error!("failed to nack message: {:?}", err);
                    }else {
                        unprocessed_messages.clear();
                    }
                    // Receiver is closed permanently.
                    return Ok(())
                }
            }
        }
    }

    pub async fn dispose(mut self) -> Result<(), Status> {
        self.task_to_ping.abort();
        self.task_to_receive.abort();

        if let Some(rx) = self.unprocessed_messages_receiver.take() {
            if let Ok(Some(messages)) = rx.await {
                // Nack all the unprocessed messages
                if !messages.is_empty() {
                    tracing::debug!("nack {} unprocessed messages", messages.len());
                    nack(&self.client, self.subscription, messages).await?;
                }
            }
        }
        Ok(())
    }
}

async fn modify_ack_deadline(
    subscriber_client: &SubscriberClient,
    subscription: String,
    ack_ids: Vec<String>,
    ack_deadline_seconds: i32,
) -> Result<(), Status> {
    if ack_ids.is_empty() {
        return Ok(());
    }
    let req = ModifyAckDeadlineRequest {
        subscription,
        ack_deadline_seconds,
        ack_ids,
    };
    subscriber_client
        .modify_ack_deadline(req, None)
        .await
        .map(|e| e.into_inner())
}

async fn nack(subscriber_client: &SubscriberClient, subscription: String, ack_ids: Vec<String>) -> Result<(), Status> {
    modify_ack_deadline(subscriber_client, subscription, ack_ids, 0).await
}

pub(crate) async fn ack(
    subscriber_client: &SubscriberClient,
    subscription: String,
    ack_ids: Vec<String>,
) -> Result<(), Status> {
    if ack_ids.is_empty() {
        return Ok(());
    }
    let req = AcknowledgeRequest { subscription, ack_ids };
    subscriber_client.acknowledge(req, None).await.map(|e| e.into_inner())
}
