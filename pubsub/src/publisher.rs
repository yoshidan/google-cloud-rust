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
        match timeout(std::time::Duration::from_secs(1), &mut self.consumer).await {
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
        let (sender, receiver) = async_channel::unbounded::<ReservedMessage>();
        let workers = (0..config.workers).map(|_| {
            let mut client = pubc.clone();
            let receiver_for_worker = receiver.clone();
            let topic_for_worker = topic.clone();
            tokio::spawn(async move {
                loop {
                    if let Ok(message) = receiver_for_worker.recv().await {
                        println!("start publish");
                        let result = client.publish(PublishRequest {
                            topic: topic_for_worker.to_string(),
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
                        print!("error");
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

        let (producer, mut consumer) = oneshot::channel();
        self.sender.send(ReservedMessage {
            producer,
            message
        });
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

}

impl Drop for Publisher {

    fn drop(&mut self) {
       self.stop() ;
    }

}