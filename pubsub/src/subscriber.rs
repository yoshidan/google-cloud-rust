use std::time::Duration;
use async_channel::Sender;
use google_cloud_gax::call_option::BackoffRetrySettings;
use google_cloud_googleapis::pubsub::v1::{AcknowledgeRequest, ModifyAckDeadlineRequest, PubsubMessage};
use google_cloud_googleapis::{Status};

use crate::apiv1::subscriber_client::{create_default_streaming_pull_request, SubscriberClient};

pub struct ReceivedMessage {
   pub message: PubsubMessage ,
   ack_id: String,
   subscription: String,
   subscriber_client: SubscriberClient
}

impl ReceivedMessage {
    pub async fn ack(&mut self) -> Result<(), Status> {
       self.subscriber_client.acknowledge(AcknowledgeRequest {
           subscription: self.subscription.to_string(),
           ack_ids: vec![self.ack_id.to_string()]
       }, None).await.map(|e| e.into_inner())
    }

    pub async fn nack(&mut self) -> Result<(), Status> {
        self.subscriber_client.modify_ack_deadline(ModifyAckDeadlineRequest {
            subscription: self.subscription.to_string(),
            ack_deadline_seconds: 0,
            ack_ids: vec![self.ack_id.to_string()]
        }, None).await.map(|e| e.into_inner())
    }
}

#[derive(Clone)]
pub struct SubscriberConfig {
    pub ping_interval: Duration,
    pub retry_setting: Option<BackoffRetrySettings>
}

impl Default for SubscriberConfig {
    fn default() -> Self {
        Self {
            ping_interval: std::time::Duration::from_secs(10),
            retry_setting: None,
        }
    }
}

pub struct Subscriber {
    pub ping_sender: Sender<bool>,
}

impl Subscriber {

    pub fn new(subscription: String, mut client: SubscriberClient, queue: async_channel::Sender<ReceivedMessage>, opt: Option<SubscriberConfig>) -> Self {
        let config = opt.unwrap_or_default();

        let (ping_sender,ping_receiver) = async_channel::unbounded();

        // ping request
        let ping_sender_clone = ping_sender.clone();
        tokio::spawn(async move {
            while !ping_sender_clone.is_closed() {
                ping_sender_clone.send(true).await;
                tokio::time::sleep(config.ping_interval).await;
            }
            println!("ping closed");
        });

        tokio::spawn(async move {
            println!("start subscriber");
            let request = create_default_streaming_pull_request(subscription.to_string());
            let response = client.streaming_pull(request, ping_receiver, config.retry_setting).await;

            match response {
                Ok(r) => {
                    let mut stream = r.into_inner();
                    while let Ok(Some(message)) = stream.message().await {
                        for m in message.received_messages {
                            if let Some(mes) = m.message {
                                log::debug!("message received: {}", mes.message_id);
                                let v = queue.send(ReceivedMessage {
                                    message: mes,
                                    ack_id: m.ack_id,
                                    subscription: subscription.to_string(),
                                    subscriber_client: client.clone()
                                }).await;
                                if v.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    // streaming request is closed when the ping_sender closed.
                    println!("closed subs");
                },
                Err(e)=> {
                    println!("subscribe error {:?}", e)
                }
            };
            ()
        });
        return Self{
            ping_sender,
        }
    }

    pub fn stop(& self) {
        self.ping_sender.close();
    }
}

impl Drop for Subscriber {
    fn drop(&mut self) {
        self.stop();
    }
}