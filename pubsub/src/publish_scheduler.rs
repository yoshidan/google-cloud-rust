use std::collections::VecDeque;
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use parking_lot::Mutex;
use prost::Message;
use tokio::sync::oneshot::{Sender, Receiver, channel};
use tokio::time::timeout;
use google_cloud_googleapis::pubsub::v1::{PublishRequest, PublishResponse, PubsubMessage};
use google_cloud_googleapis::Status;
use crate::apiv1::publisher_client::PublisherClient;

pub struct ReservedMessage {
    chan: Sender<Result<String,Status>>,
    message: PubsubMessage,
}

pub struct SchedulerConfig {
    max_ticket: usize,
    timeout: Duration,
}

pub struct PublishScheduler {
    ticket: AtomicUsize,
    queue: Arc<Mutex<VecDeque<ReservedMessage>>>,
    config: SchedulerConfig,
    pubc: PublisherClient
}

pub struct Awaiter {
    receiver: Receiver<Result<String,Status>>
}

impl Awaiter {
    pub(crate) fn new(receiver: Receiver<Result<String,Status>>) -> Self {
        Self {
            receiver,
        }
    }
    pub async fn get(&mut self) -> Result<String, Status> {
        match timeout(self.config.session_get_timeout, receiver).await {
           Ok(v) => v,
           Err(e) => Err(Status::new(tonic::Status::deadline_exceeded(e.to_string())))
       }
    }
}

impl PublishScheduler {

    pub fn new() {
    }

    pub async fn publish(&mut self, topic: String, message: PubsubMessage) -> Awaiter{
        let (sender,receiver) = channel();
        let before_fetch = self.ticket.fetch_add(1, Ordering::SeqCst);
        if before_fetch < self.config.max_ticket {
            let result = self.pubc.publish(PublishRequest {
                topic,
                messages: vec![message]
            },None).await.map(|v| v.into_inner().message_ids[0]);
            self.post_publish();
            sender.send(result);
            return Awaiter {
                receiver,
            }
        }
        self.ticket.fetch_sub(1, Ordering::SeqCst);

        //enqueue
        self.queue.push_back(ReservedMessage {
            chan: sender,
            message,
        });
        return Awaiter {
            receiver,
        }
    }

    pub async fn flush(&mut self) {
       loop {
           let v = self.send_queued_data().await;
           if v == 0 {
               break;
           }
       }
    }

    fn post_publish(&mut self) {
        tokio::spawn(async move {
            self.send_queued_data().await;
            // return the ticket
            self.ticket.fetch_sub(1, Ordering::SeqCst);
        });
    }

    async fn send_queued_data(&mut self) -> usize {
        // publish rest
        let (messages, chan) = self.dequeue(10);
        let result = self.pubc.publish(PublishRequest {
            topic,
            messages,
        },None).await.map(|v| v.into_inner().message_ids);

        // notify to receivers
        match result {
            Ok(message_ids) => {
                for (i, sender) in chan.iter().enumerate() {
                    sender.send(Ok(message_ids[i].to_string()));
                }
            },
            Err(status) => {
                chan.into_iter().for_each(|v| {
                    v.send(Err(status));
                });
            }
        }
        return messages.len()
    }

    fn dequeue(&mut self, size: usize) -> (Vec<PubsubMessage>,Vec<Sender<Result<String,Status>>>) {
        let mut locked = self.queue.lock();
        let mut messages = Vec::with_capacity(size);
        let mut target_chan= Vec::with_capacity(size);
        for _ in 0..size {
            if let Some(v) = locked.pop_front() {
                messages.push_back(v.message);
                target_chan.push_back(v.chan);
            }else {
                break;
            }
        }
        (messages, target_chan)
    }
}