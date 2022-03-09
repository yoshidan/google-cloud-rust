use std::time::Duration;

use google_cloud_gax::cancel::CancellationToken;
use google_cloud_gax::grpc::{Code, Status, Streaming};
use google_cloud_gax::retry::RetrySetting;
use google_cloud_googleapis::pubsub::v1::{AcknowledgeRequest, ModifyAckDeadlineRequest, PubsubMessage, StreamingPullRequest, StreamingPullResponse};
use tokio::select;
use tokio::task::JoinHandle;
use tokio::time::sleep;

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

    pub async fn ack(&self) -> Result<(), Status> {
        let req = AcknowledgeRequest {
            subscription: self.subscription.to_string(),
            ack_ids: vec![self.ack_id.to_string()],
        };
        self.subscriber_client
            .acknowledge(req, None, None)
            .await
            .map(|e| e.into_inner())
    }

    pub async fn nack(&self) -> Result<(), Status> {
        let req = ModifyAckDeadlineRequest {
            subscription: self.subscription.to_string(),
            ack_deadline_seconds: 0,
            ack_ids: vec![self.ack_id.to_string()],
        };
        self.subscriber_client
            .modify_ack_deadline(req, None, None)
            .await
            .map(|e| e.into_inner())
    }
}

#[derive(Clone)]
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
            retry_setting: None,
            stream_ack_deadline_seconds: 60,
            max_outstanding_messages: 1000,
            max_outstanding_bytes: 1000 * 1000 * 1000,
        }
    }
}

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

        let cancel_receiver = ctx.clone();
        let (ping_sender, ping_receiver) = async_channel::unbounded();

        // ping request
        let subscription_clone = subscription.to_string();

        let pinger = tokio::spawn(async move {
            loop {
                select! {
                    _ = cancel_receiver.cancelled() => {
                        ping_sender.close();
                        break;
                    }
                    _ = sleep(config.ping_interval) => {
                        ping_sender.send(true).await;
                    }
                }
            }
            log::trace!("stop pinger : {}", subscription_clone);
        });

        let cancel_receiver = ctx.clone();
        let inner = tokio::spawn(async move {
            log::trace!("start subscriber: {}", subscription);
            loop {
                let mut request = create_empty_streaming_pull_request();
                request.subscription = subscription.to_string();
                request.stream_ack_deadline_seconds = config.stream_ack_deadline_seconds;
                request.max_outstanding_messages = config.max_outstanding_messages;
                request.max_outstanding_bytes = config.max_outstanding_bytes;

                let response = client
                    .streaming_pull(request, Some(cancel_receiver.clone()), ping_receiver.clone(), config.retry_setting.clone())
                    .await;

                let mut stream = match response {
                    Ok(r) => r.into_inner(),
                    Err(e) => {
                        if e.code() == Code::Cancelled {
                            log::trace!("stop subscriber : {}", subscription);
                        } else {
                            log::error!("subscriber error {:?} : {}", e, subscription);
                        }
                        break;
                    }
                };
                match Self::recv(client.clone(), stream, subscription.as_str(), cancel_receiver.clone(), queue.clone()).await {
                    Ok(_) => break,
                    Err(e)  => {
                        if e.code() == Code::Unavailable || e.code() == Code::Unknown || e.code() == Code::Internal {
                            log::trace!("reconnect - '{:?}' : {} ", e, subscription);
                            continue;
                        } else {
                            log::error!("streaming error {:?} : {}", e, subscription);
                            break;
                        }
                    }
                }
            }
            // streaming request is closed when the ping_sender closed.
            log::trace!("stop subscriber in streaming: {}", subscription);
        });
        return Self {
            pinger: Some(pinger),
            inner: Some(inner),
        };
    }

    async fn recv(client: SubscriberClient, mut stream: Streaming<StreamingPullResponse>, subscription: &str, cancel: CancellationToken, queue: async_channel::Sender<ReceivedMessage>)  -> Result<(),Status>{
        log::trace!("start streaming: {}", subscription);
        loop {
            select! {
                _ = cancel.cancelled() => {
                    queue.close();
                    return Ok(());
                }
                maybe = stream.message() => {
                    let message = match maybe{
                       Err(e) => return Err(e),
                       Ok(message) => message
                    };
                    let message = match message {
                        Some(m) => m,
                        None => return Ok(())
                    };
                    for m in message.received_messages {
                        if let Some(mes) = m.message {
                            log::debug!("message received: {}", mes.message_id);
                            queue.send(ReceivedMessage::new(subscription.to_string(), client.clone(), mes, m.ack_id)).await;
                        }
                    }
                }
            }
        }
    }

    pub async fn done(&mut self) {
        if let Some(v) = self.pinger.take() {
            v.await;
        }
        if let Some(v) = self.inner.take() {
            v.await;
        }
    }
}
