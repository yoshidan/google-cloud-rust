use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::select;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use google_cloud_gax::grpc::{Code, Status, Streaming};
use google_cloud_gax::retry::RetrySetting;
use google_cloud_googleapis::pubsub::v1::{AcknowledgeRequest, ModifyAckDeadlineRequest, PubsubMessage, ReceivedMessage as InternalReceivedMessage, StreamingPullRequest, StreamingPullResponse};

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
        Self {
            ping_interval: std::time::Duration::from_secs(10),
            retry_setting: Some(default_retry_setting()),
            stream_ack_deadline_seconds: 60,
            max_outstanding_messages: 50,
            max_outstanding_bytes: 1000 * 1000 * 1000,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Subscriber {
    task_to_ping: JoinHandle<()>,
    task_to_receive: JoinHandle<()>,
    /// Ack id list of unprocessed messages.
    unprocessed_messages: Arc<Mutex<Vec<ReceivedMessage>>>,
}

impl Subscriber {
    pub fn new(
        subscription: String,
        client: SubscriberClient,
        queue: async_channel::Sender<ReceivedMessage>,
        config: SubscriberConfig,
    ) -> Self {
        let (ping_sender, ping_receiver) = async_channel::unbounded();

        // Build task to ping
        let task_to_ping = async move {
            loop {
                _ = sleep(config.ping_interval) ;
                let _ = ping_sender.send(true).await;
            }
            tracing::trace!("stop ping");
        };

        // Build task to receive
        let mut unprocessed_messages  = Arc::new(Mutex::new(Vec::new()));
        let task_to_receive = async move {
            tracing::trace!("start subscriber: {}", subscription);
            let retryable_codes = match &config.retry_setting {
                Some(v) => v.codes.clone(),
                None => default_retry_setting().codes,
            };

            loop {
                let mut request = create_empty_streaming_pull_request();
                request.subscription = subscription.to_string();
                request.stream_ack_deadline_seconds = config.stream_ack_deadline_seconds;
                request.max_outstanding_messages = config.max_outstanding_messages;
                request.max_outstanding_bytes = config.max_outstanding_bytes;

                tracing::trace!("start streaming: {}", subscription);

                let mut lock = unprocessed_messages.lock().unwrap();
                let response = Self::start_streaming(
                    client.clone(),
                    request,
                    ping_receiver.clone(),
                    config.clone(),
                    queue.clone(),
                    &mut lock);

                if let Err(e) = response {
                    if retryable_codes.contains(&e.code()) {
                        tracing::warn!("failed to start streaming: will reconnect {:?} : {}", e, subscription);
                        continue;
                    } else {
                        tracing::error!("failed to start streaming: will stop {:?} : {}", e, subscription);
                        break;
                    }
                };
            }

            // streaming request is closed when the ping_sender closed.
            tracing::trace!("stop subscriber: {}", subscription);
        };

        Self {
            task_to_ping: tokio::spawn(task_to_ping),
            task_to_receive: tokio::spawn(task_to_receive),
            unprocessed_messages,
        }
    }

    async fn start_streaming(
        client:SubscriberClient,
        request: StreamingPullRequest,
        ping_receiver: async_channel::Receiver<bool>,
        config: SubscriberConfig,
        queue: async_channel::Sender<ReceivedMessage>,
        unprocessed_messages: &mut Vec<ReceivedMessage>,
    ) -> Result<(), Status>{
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
                None => return Ok(())
            };

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
                    unprocessed_messages.push(msg);
                }
            }

            for msg in unprocessed_messages.drain(..) {
                if let Err(e) = queue.send(msg).await {
                    tracing::error!("failed to send message to queue: msg_id={} ack_id={}", &e.0.message.message_id, &e.0.ack_id);
                    unprocessed_messages.push(e.0);
                }
            }
        }
    }

    pub async fn run(mut self, ctx: CancellationToken) -> Result<(), Status>{
        select ! {
            _ = &self.task_to_receive => {
                &self.task_to_ping.abort();
                tracing::warn!("streaming finished unexpectedly");
            },
            _ = ctx.cancelled() => {
                &self.task_to_ping.abort();
                let _ = &self.task_to_receive.await;
                tracing::trace!("streaming finished successfully");
            }
        }
        if self.unprocessed_messages.lock().unwrap().is_empty() {
            return Ok(());
        }
        let first = self.unprocessed_messages.lock().unwrap().first().unwrap();
        nack(
            &first.subscriber_client, (&first).subscription.to_string(),
            self.unprocessed_messages.lock().unwrap().iter().map(|m| m.ack_id.clone()).collect(),
        )
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

#[cfg(test)]
mod tests {
    use serial_test::serial;
    use tokio_util::sync::CancellationToken;

    use google_cloud_gax::conn::{ConnectionOptions, Environment};
    use google_cloud_googleapis::pubsub::v1::{PublishRequest, PubsubMessage, PullRequest};

    use crate::apiv1::conn_pool::ConnectionManager;
    use crate::apiv1::publisher_client::PublisherClient;
    use crate::apiv1::subscriber_client::SubscriberClient;

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt().try_init();
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_handle_message_immediately_nack() {
        let cm = || async {
            ConnectionManager::new(
                4,
                "",
                &Environment::Emulator("localhost:8681".to_string()),
                &ConnectionOptions::default(),
            )
            .await
            .unwrap()
        };
        let subc = SubscriberClient::new(cm().await, cm().await);
        let pubc = PublisherClient::new(cm().await);

        pubc.publish(
            PublishRequest {
                topic: "projects/local-project/topics/test-topic1".to_string(),
                messages: vec![PubsubMessage {
                    data: "hoge".into(),
                    ..Default::default()
                }],
            },
            None,
        )
        .await
        .unwrap();

        let subscription = "projects/local-project/subscriptions/test-subscription1";
        let response = subc
            .pull(
                PullRequest {
                    subscription: subscription.to_string(),
                    max_messages: 1,
                    ..Default::default()
                },
                None,
            )
            .await
            .unwrap()
            .into_inner();

        let messages = response.received_messages;
        let (queue, _) = async_channel::unbounded();
        queue.close();
        let nack_size = handle_message(&CancellationToken::new(), &queue, &subc, subscription, messages).await;
        assert_eq!(1, nack_size);
    }
}
