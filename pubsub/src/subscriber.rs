use std::time::Duration;

use google_cloud_gax::cancel::CancellationToken;
use google_cloud_gax::retry::RetrySetting;
use google_cloud_gax::status::{Code, Status};
use google_cloud_googleapis::pubsub::v1::{
    AcknowledgeRequest, ModifyAckDeadlineRequest, PubsubMessage,
};
use tokio::select;
use tokio::task::JoinHandle;
use tokio::time::sleep;

use crate::apiv1::subscriber_client::{create_default_streaming_pull_request, SubscriberClient};

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
}

impl Default for SubscriberConfig {
    fn default() -> Self {
        Self {
            ping_interval: std::time::Duration::from_secs(10),
            retry_setting: None,
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
            let request = create_default_streaming_pull_request(subscription.to_string());
            let response = client
                .streaming_pull(request, Some(cancel_receiver.clone()), ping_receiver, config.retry_setting)
                .await;

            let mut stream = match response {
                Ok(r) => r.into_inner(),
                Err(e) => {
                    if e.code() == Code::Cancelled {
                        log::trace!("stop subscriber : {}", subscription);
                    } else {
                        log::error!("subscriber error {:?} : {}", e, subscription);
                    }
                    return;
                }
            };
            log::trace!("start streaming: {}", subscription);
            loop {
                select! {
                    _ = cancel_receiver.cancelled() => {
                        queue.close();
                        break;
                    }
                    maybe = stream.message() => {
                        let message = match maybe{
                           Err(_e) => break,
                           Ok(message) => message
                        };
                        let message = match message {
                            Some(m) => m,
                            None => break
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
            // streaming request is closed when the ping_sender closed.
            log::trace!("stop subscriber in streaming: {}", subscription);
        });
        return Self {
            pinger: Some(pinger),
            inner: Some(inner),
        };
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
