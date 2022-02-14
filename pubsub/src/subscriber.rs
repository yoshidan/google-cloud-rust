use std::collections::VecDeque;
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use async_channel::Sender;
use parking_lot::{Mutex, RwLock};
use prost::Message;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use google_cloud_googleapis::pubsub::v1::{AcknowledgeRequest, ModifyAckDeadlineRequest, PublishRequest, PublishResponse, PubsubMessage, PullRequest, StreamingPullRequest, StreamingPullResponse};
use google_cloud_googleapis::{Code, Status};
use crate::apiv1::publisher_client::PublisherClient;
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

pub struct Subscriber {
    pub ping_sender: Sender<bool>,
}

impl Subscriber {

    pub fn new(subscription: String, mut client: SubscriberClient, queue: async_channel::Sender<ReceivedMessage>, ping_interval_second: u64) -> Self {
        let (ping_sender,ping_receiver) = async_channel::unbounded();

        // ping request
        let ping_sender_clone = ping_sender.clone();
        let ping_handle = tokio::spawn(async move {
            while !ping_sender_clone.is_closed() {
                ping_sender_clone.send(true).await;
                tokio::time::sleep(std::time::Duration::from_secs(ping_interval_second)).await;
            }
            println!("ping closed");
        });

        let receive_handle = tokio::spawn(async move {
            println!("start subscriber");
            let request = create_default_streaming_pull_request(subscription.to_string());
            let response = client.streaming_pull(request, ping_receiver, None).await;

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

    pub fn close(& self) {
        self.ping_sender.close();
    }
}

impl Drop for Subscriber {
    fn drop(&mut self) {
        self.close();
    }
}