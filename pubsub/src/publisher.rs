use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use parking_lot::{Mutex, RwLock};
use prost::Message;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use google_cloud_googleapis::pubsub::v1::{PublishRequest, PublishResponse, PubsubMessage};
use google_cloud_googleapis::{Code, Status};
use crate::apiv1::publisher_client::PublisherClient;
use crate::util::ToUsize;

pub struct ReservedMessage {
    producer: oneshot::Sender<Result<String,Status>>,
    message: PubsubMessage,
}

#[derive(Clone)]
pub struct PublisherConfig {
    pub workers: usize,
    pub timeout: Duration,
    pub buffer_size: usize
}

impl Default for PublisherConfig {
    fn default() -> Self {
        Self {
            workers: 3,
            timeout: std::time::Duration::from_secs(3),
            buffer_size: 3
        }
    }
}

pub struct Publisher {
    priority_senders: Vec<async_channel::Sender<ReservedMessage>>,
    sender: async_channel::Sender<ReservedMessage>,
    workers: Vec<JoinHandle<()>>,
    stopped: bool
}

pub struct Awaiter {
    consumer: oneshot::Receiver<Result<String,Status>>
}

impl Awaiter {
    pub(crate) fn new(consumer: oneshot::Receiver<Result<String,Status>>) -> Self {
        Self {
            consumer,
        }
    }
    pub async fn get(&mut self) -> Result<String, Status> {
        match timeout(std::time::Duration::from_secs(3), &mut self.consumer).await {
           Ok(v) => match v {
               Ok(vv) => vv,
               Err(e) => Err(Status::new(tonic::Status::unknown(e.to_string())))
           },
           Err(e) => Err(Status::new(tonic::Status::deadline_exceeded(e.to_string())))
       }
    }
}

impl Publisher {

    pub fn new( topic: String, config: PublisherConfig, pubc: PublisherClient ) -> Self {
        let (sender, receiver) = async_channel::unbounded::<ReservedMessage>();
        let mut receivers = Vec::with_capacity(1 + config.workers);
        let mut priority_senders = Vec::with_capacity(config.workers);
        for _ in 0..config.workers {
            receivers.push(receiver.clone()) ;
        }

        // ordering key message
        for _ in 0..config.workers {
            let (sender, receiver) = async_channel::unbounded::<ReservedMessage>();
            receivers.push(receiver);
            priority_senders.push(sender);
        }

        let workers = receivers.into_iter().map(|receiver| {
            let mut client = pubc.clone();
            let topic_for_worker = topic.clone();
            tokio::spawn(async move {
                let mut buffer = VecDeque::<ReservedMessage>::new();
                loop {
                    match timeout(std::time::Duration::from_millis(1),&mut receiver.recv()).await {
                        Ok(result) => match result {
                            Ok(message) => {
                                buffer.push_back(message);
                                if buffer.len() > 3 {
                                    println!("flush buffer worker");
                                    Self::flush(&mut client, topic_for_worker.as_str(), buffer).await;
                                    buffer = VecDeque::new();
                                }
                            }
                            Err(e) => {
                                //closed
                                println!("closed worker");
                                break;
                            }
                        },
                        Err(e) => {
                            if !buffer.is_empty() {
                                Self::flush(&mut client, topic_for_worker.as_str(), buffer).await;
                                buffer = VecDeque::new();
                                println!("done flush worker")
                            }
                        }
                    }
                }
            })
        }).collect();
        Self {
            workers,
            sender,
            priority_senders,
            stopped: false
        }
    }

    pub async fn publish(&self, message: PubsubMessage) -> Awaiter{

        let (producer, mut consumer) = oneshot::channel();
        if message.ordering_key.is_empty() {
            self.sender.send( ReservedMessage {
                producer,
                message
            }).await;
        }else {
            let key = message.ordering_key.as_str().to_usize();
            let index = key % self.priority_senders.len();
            self.priority_senders[index].send(ReservedMessage {
                producer,
                message
            }).await;
        }
        return Awaiter {
            consumer,
        }
    }

    pub fn stop(&mut self) {
        if self.stopped {
            return
        }
        self.stopped = true;
        self.sender.close();
        for worker in self.workers.iter() {
            worker.abort();
        }
    }


    async fn flush(client: &mut PublisherClient, topic: &str, buffer: VecDeque<ReservedMessage>) {
        let mut data = Vec::<PubsubMessage> ::with_capacity(buffer.len());
        let mut callback = Vec::<oneshot::Sender<Result<String,Status>>>::with_capacity(buffer.len());
        buffer.into_iter().for_each(|r| {
            data.push(r.message);
            callback.push(r.producer);
        });
        let result = client.publish(PublishRequest {
            topic: topic.to_string(),
            messages: data,
        }, None).await.map(|v| v.into_inner().message_ids);

        // notify to receivers
        match result {
            Ok(message_ids) => {
                for (i, p) in callback.into_iter().enumerate() {
                    p.send(Ok(message_ids[i].to_string()));
                }
            },
            Err(status) => {
                for p in callback.into_iter() {
                    //TODO copy error
                    p.send(Err(Status::new(tonic::Status::new(status.source.code().clone(), status.source.message().clone()))));
                }
            }
        };
    }

}

impl Drop for Publisher {

    fn drop(&mut self) {
       self.stop() ;
    }

}
