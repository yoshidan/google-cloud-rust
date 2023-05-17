use std::time::Duration;

use tokio::select;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use google_cloud_gax::grpc::{Code, Status, Streaming};
use google_cloud_gax::retry::RetrySetting;
use google_cloud_googleapis::pubsub::v1::{
    AcknowledgeRequest, ModifyAckDeadlineRequest, PubsubMessage, ReceivedMessage as InternalReceivedMessage,
    StreamingPullResponse,
};

use crate::apiv1::default_retry_setting;
use crate::apiv1::subscriber_client::{create_empty_streaming_pull_request, SubscriberClient};

pub struct ReceivedMessage {
    pub message: PubsubMessage,
    ack_id: String,
    subscription: String,
    subscriber_client: SubscriberClient,
}

impl ReceivedMessage {
    pub(crate) fn new(subscription: String, subc: SubscriberClient, message: PubsubMessage, ack_id: String) -> Self {
        Self {
            message,
            ack_id,
            subscription,
            subscriber_client: subc,
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
}

#[derive(Debug, Clone)]
pub struct SubscriberConfig {
    /// ping interval for Bi Directional Streaming
    pub ping_interval: Duration,
    pub retry_setting: Option<RetrySetting>,
    pub stream_ack_deadline_seconds: i32,
    pub max_outstanding_messages: i64,
    pub max_outstanding_bytes: i64,
}

impl Default for SubscriberConfig {
    fn default() -> Self {
        Self {
            ping_interval: std::time::Duration::from_secs(10),
            retry_setting: Some(default_retry_setting()),
            stream_ack_deadline_seconds: 60,
            max_outstanding_messages: 1000,
            max_outstanding_bytes: 1000 * 1000 * 1000,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Subscriber {
    pinger: Option<JoinHandle<()>>,
    inner: Option<JoinHandle<()>>,
}

impl Subscriber {
    pub fn start(
        ctx: CancellationToken,
        subscription: String,
        client: SubscriberClient,
        queue: async_channel::Sender<ReceivedMessage>,
        opt: Option<SubscriberConfig>,
    ) -> Self {
        let config = opt.unwrap_or_default();

        let (ping_sender, ping_receiver) = async_channel::unbounded();

        // ping request
        let subscription_clone = subscription.to_string();

        let cancel_receiver = ctx.clone();
        let pinger = tokio::spawn(async move {
            loop {
                select! {
                    _ = ctx.cancelled() => {
                        ping_sender.close();
                        break;
                    }
                    _ = sleep(config.ping_interval) => {
                        let _ = ping_sender.send(true).await;
                    }
                }
            }
            tracing::trace!("stop pinger : {}", subscription_clone);
        });

        let inner = tokio::spawn(async move {
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

                let response = client
                    .streaming_pull(request, ping_receiver.clone(), config.retry_setting.clone())
                    .await;

                let stream = match response {
                    Ok(r) => r.into_inner(),
                    Err(e) => {
                        if e.code() == Code::Cancelled {
                            tracing::trace!("stop subscriber : {}", subscription);
                            break;
                        } else if retryable_codes.contains(&e.code()) {
                            tracing::warn!("failed to start streaming: will reconnect {:?} : {}", e, subscription);
                            continue;
                        } else {
                            tracing::error!("failed to start streaming: will stop {:?} : {}", e, subscription);
                            break;
                        }
                    }
                };
                match Self::recv(
                    client.clone(),
                    stream,
                    subscription.as_str(),
                    cancel_receiver.clone(),
                    queue.clone(),
                )
                .await
                {
                    Ok(_) => break,
                    Err(e) => {
                        if retryable_codes.contains(&e.code()) {
                            tracing::trace!("reconnect - '{:?}' : {} ", e, subscription);
                            continue;
                        } else {
                            tracing::error!("terminated subscriber streaming with error {:?} : {}", e, subscription);
                            break;
                        }
                    }
                }
            }
            // streaming request is closed when the ping_sender closed.
            tracing::trace!("stop subscriber in streaming: {}", subscription);
        });
        Self {
            pinger: Some(pinger),
            inner: Some(inner),
        }
    }

    async fn recv(
        client: SubscriberClient,
        mut stream: Streaming<StreamingPullResponse>,
        subscription: &str,
        cancel: CancellationToken,
        queue: async_channel::Sender<ReceivedMessage>,
    ) -> Result<(), Status> {
        tracing::trace!("start streaming: {}", subscription);
        loop {
            select! {
                _ = cancel.cancelled() => {
                    queue.close();
                    return Ok(());
                }
                maybe = stream.message() => {
                    let message = maybe?;
                    let message = match message {
                        Some(m) => m,
                        None => return Ok(())
                    };
                    let _ = handle_message(&queue, &client, subscription, message.received_messages).await;
                }
            }
        }
    }

    pub async fn done(&mut self) {
        if let Some(v) = self.pinger.take() {
            let _ = v.await;
        }
        if let Some(v) = self.inner.take() {
            let _ = v.await;
        }
    }
}

async fn handle_message(
    queue: &async_channel::Sender<ReceivedMessage>,
    client: &SubscriberClient,
    subscription: &str,
    messages: Vec<InternalReceivedMessage>,
) -> usize {
    let mut nack_targets = vec![];
    for received_message in messages {
        if let Some(message) = received_message.message {
            let id = message.message_id.clone();
            tracing::debug!("message received: msg_id={id}");
            if let Err(err) = queue
                .send(ReceivedMessage::new(
                    subscription.to_string(),
                    client.clone(),
                    message,
                    received_message.ack_id.clone(),
                ))
                .await
            {
                tracing::error!(%err, "failed to send receiver queue -> so nack immediately : msg_id={id}");
                nack_targets.push(received_message.ack_id);
            }
        }
    }
    let size = nack_targets.len();
    if size > 0 {
        // Nack immediately although the queue is closed only when the cancellation token is closed.
        if let Err(err) = nack(client, subscription.to_string(), nack_targets).await {
            tracing::error!(
                "failed to nack immediately {err}. The messages will be redelivered after the ack deadline."
            );
        }
    }
    size
}

async fn nack(subscriber_client: &SubscriberClient, subscription: String, ack_ids: Vec<String>) -> Result<(), Status> {
    if ack_ids.is_empty() {
        return Ok(());
    }
    let req = ModifyAckDeadlineRequest {
        subscription,
        ack_deadline_seconds: 0,
        ack_ids,
    };
    subscriber_client
        .modify_ack_deadline(req, None)
        .await
        .map(|e| e.into_inner())
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

    use google_cloud_gax::conn::Environment;
    use google_cloud_googleapis::pubsub::v1::{PublishRequest, PubsubMessage, PullRequest};

    use crate::apiv1::conn_pool::ConnectionManager;
    use crate::apiv1::publisher_client::PublisherClient;
    use crate::apiv1::subscriber_client::SubscriberClient;
    use crate::subscriber::handle_message;

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt().try_init();
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_handle_message_immediately_nack() {
        let cm = || async {
            ConnectionManager::new(4, "", &Environment::Emulator("localhost:8681".to_string()))
                .await
                .unwrap()
        };
        let subc = SubscriberClient::new(cm().await);
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
        let nack_size = handle_message(&queue, &subc, subscription, messages).await;
        assert_eq!(1, nack_size);
    }
}
