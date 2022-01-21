use std::collections::VecDeque;
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use parking_lot::Mutex;
use prost::Message;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use google_cloud_googleapis::pubsub::v1::{PublishRequest, PublishResponse, PubsubMessage};
use google_cloud_googleapis::Status;
use crate::apiv1::publisher_client::PublisherClient;

pub struct ReservedMessage {
    producer: oneshot::Sender<Result<String,Status>>,
    message: PubsubMessage,
}

pub struct SchedulerConfig {
    pub workers: usize,
    pub timeout: Duration,
}

pub struct Publisher {
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
        match timeout(self.config.session_get_timeout, self.consumer).await {
           Ok(v) => match v {
               Ok(vv) => vv,
               Err(e) => Err(Status::new(tonic::Status::unknown(e.to_string())))
           },
           Err(e) => Err(Status::new(tonic::Status::deadline_exceeded(e.to_string())))
       }
    }
}

impl Publisher {

    pub fn new( topic: String, config: SchedulerConfig, pubc: PublisherClient ) -> Self {
        let (sender, receiver) = async_channel::unbounded();
        let workers = (0..config.workers).map(|| {
            let mut client = pubc.clone();
            tokio::spawn(async {
                loop {
                    if let Ok(message) = receiver.recv().await {
                        let result = client.publish(PublishRequest {
                            topic: topic.to_string(),
                            messages: vec![message.message],
                        },None).await.map(|v| v.into_inner().message_ids);

                        // notify to receivers
                        match result {
                            Ok(message_ids) => {
                                message.producer.send(Ok(message_ids[0].to_string()));
                            },
                            Err(status) => {
                                message.producer.send(Err(status));
                            }
                        }
                    }else {
                        break;
                    }
                }
            })
        }).collect();
        Self {
            workers,
            sender,
            stopped: false
        }
    }

    pub async fn publish(&mut self, message: PubsubMessage) -> Awaiter{

        let (producer, consumer) = oneshot::channel();
        self.sender.send(ReservedMessage {
            producer,
            message
        });
        return Awaiter {
            consumer,
        }
    }

    pub async fn stop(&mut self) {
        if self.stopped {
            return
        }
        self.stopped = true;
        self.sender.close();
        for worker in self.workers {
            worker.await;
        }
    }

}

impl Drop for Publisher {

    fn drop(&mut self) {
        if self.stopped {
            return
        }
        self.sender.close();
        self.workers.into_iter().for_each(|v| v.abort())
    }

}