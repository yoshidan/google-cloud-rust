use std::collections::{VecDeque};
use std::time::Duration;

use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use google_cloud_gax::call_option::BackoffRetrySettings;
use google_cloud_googleapis::pubsub::v1::{PublishRequest, PubsubMessage};
use google_cloud_googleapis::{Status};
use crate::apiv1::publisher_client::PublisherClient;
use crate::util::ToUsize;

pub(crate) struct ReservedMessage {
    producer: oneshot::Sender<Result<String,Status>>,
    message: PubsubMessage,
}

#[derive(Clone)]
pub struct PublisherConfig {
    pub workers: usize,
    pub flush_buffer_interval: Duration,
    pub buffer_size: usize,
    pub publish_timeout: Duration,
    pub retry_setting: Option<BackoffRetrySettings>
}

impl Default for PublisherConfig {
    fn default() -> Self {
        Self {
            workers: 3,
            flush_buffer_interval: Duration::from_millis(100),
            buffer_size: 3,
            publish_timeout: Duration::from_secs(3),
            retry_setting: None,
        }
    }
}

pub struct Awaiter {
    consumer: oneshot::Receiver<Result<String,Status>>,
    await_timeout: Duration
}

impl Awaiter {
    pub(crate) fn new(await_timeout: Duration, consumer: oneshot::Receiver<Result<String,Status>>) -> Self {
        Self {
            consumer,
            await_timeout,
        }
    }
    pub async fn get(&mut self) -> Result<String, Status> {
        match timeout(self.await_timeout, &mut self.consumer).await {
           Ok(v) => match v {
               Ok(vv) => vv,
               Err(e) => Err(Status::new(tonic::Status::cancelled(e.to_string())))
           },
           Err(e) => Err(Status::new(tonic::Status::deadline_exceeded(e.to_string())))
       }
    }
}

pub(crate) struct Publisher {
    priority_senders: Vec<async_channel::Sender<ReservedMessage>>,
    sender: async_channel::Sender<ReservedMessage>,
    workers: Vec<JoinHandle<()>>,
    publish_timeout: Duration,
}

impl Publisher {

    pub fn new(topic: String, pubc: PublisherClient, opt: Option<PublisherConfig>) -> Self {
        let config = opt.unwrap_or_default();
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
            let retry_setting = config.retry_setting.clone();
            tokio::spawn(async move {
                let mut buffer = VecDeque::<ReservedMessage>::new();
                while !receiver.is_closed() {
                    match timeout(config.flush_buffer_interval,&mut receiver.recv()).await {
                        Ok(result) => match result {
                            Ok(message) => {
                                buffer.push_back(message);
                                if buffer.len() > config.buffer_size {
                                    println!("flush buffer worker");
                                    Self::flush(&mut client, topic_for_worker.as_str(), buffer, retry_setting.clone()).await;
                                    buffer = VecDeque::new();
                                }
                            }
                            Err(_e) => {
                                //closed
                                println!("closed worker");
                                break;
                            }
                        },
                        Err(_e) => {
                            if !buffer.is_empty() {
                                Self::flush(&mut client, topic_for_worker.as_str(), buffer, retry_setting.clone()).await;
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
            publish_timeout: config.publish_timeout
        }
    }

    pub async fn publish(&self, message: PubsubMessage) -> Awaiter{

        let (producer, consumer) = oneshot::channel();
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
        Awaiter::new(self.publish_timeout, consumer)
    }

    pub fn close(&mut self) {
        self.sender.close();
        for ps in self.priority_senders.iter() {
            ps.close();
        }
    }

    async fn flush(client: &mut PublisherClient, topic: &str, buffer: VecDeque<ReservedMessage>, retry_setting: Option<BackoffRetrySettings>) {
        let mut data = Vec::<PubsubMessage> ::with_capacity(buffer.len());
        let mut callback = Vec::<oneshot::Sender<Result<String,Status>>>::with_capacity(buffer.len());
        buffer.into_iter().for_each(|r| {
            data.push(r.message);
            callback.push(r.producer);
        });
        let result = client.publish(PublishRequest {
            topic: topic.to_string(),
            messages: data,
        }, retry_setting).await.map(|v| v.into_inner().message_ids);

        // notify to receivers
        match result {
            Ok(message_ids) => {
                for (i, p) in callback.into_iter().enumerate() {
                    p.send(Ok(message_ids[i].to_string()));
                }
            },
            Err(status) => {
                for p in callback.into_iter() {
                    p.send(Err(Status::new(tonic::Status::new(status.source.code().clone(), &(*status.source.message()).to_string()))));
                }
            }
        };
    }

}

impl Drop for Publisher {

    fn drop(&mut self) {
       self.close() ;
    }

}
