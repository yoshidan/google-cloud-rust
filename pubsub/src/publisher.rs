use std::collections::{VecDeque};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;
use google_cloud_googleapis::pubsub::v1::{PublishRequest, PubsubMessage};
use google_cloud_googleapis::{Status};
use crate::apiv1::conn_pool::ConnectionManager;
use crate::apiv1::publisher_client::PublisherClient;
use crate::apiv1::RetrySetting;
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
    pub retry_setting: Option<RetrySetting>
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

/// Publisher is a scheduler which is designed for Pub/Sub's Publish flow.
/// Each item is added with a given key.
/// Items added to the empty string key are handled in random order.
/// Items added to any other key are handled sequentially.
pub(crate) struct Publisher {
    ordering_senders: Vec<async_channel::Sender<ReservedMessage>>,
    sender: async_channel::Sender<ReservedMessage>,
    publish_timeout: Duration,
    workers: Option<Vec<JoinHandle<()>>>
}

impl Publisher {

    pub fn new(topic: String, pubc: PublisherClient, opt: Option<PublisherConfig>) -> Self {
        let config = opt.unwrap_or_default();
        let (sender, receiver) = async_channel::unbounded::<ReservedMessage>();
        let mut receivers = Vec::with_capacity(1 + config.workers);
        let mut ordering_senders = Vec::with_capacity(config.workers);

        // for non-ordering key message
        for _ in 0..config.workers {
            log::trace!("start non-ordering publisher : {}", topic.clone());
            receivers.push(receiver.clone()) ;
        }

        // for ordering key message
        for _ in 0..config.workers {
            log::trace!("start ordering publisher : {}", topic.clone());
            let (sender, receiver) = async_channel::unbounded::<ReservedMessage>();
            receivers.push(receiver);
            ordering_senders.push(sender);
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
                                if buffer.len() >= config.buffer_size {
                                    log::trace!("maximum buffer {} : {}", buffer.len(), topic_for_worker);
                                    Self::flush(&mut client, topic_for_worker.as_str(), buffer, retry_setting.clone()).await;
                                    buffer = VecDeque::new();
                                }
                            }
                            Err(_e) => {
                                //closed
                                log::trace!("stop publisher : {}", topic_for_worker);
                                break;
                            }
                        },
                        //timed out
                        Err(_e) => {
                            if !buffer.is_empty() {
                                log::trace!("elapsed: flush buffer : {}", topic_for_worker);
                                Self::flush(&mut client, topic_for_worker.as_str(), buffer, retry_setting.clone()).await;
                                buffer = VecDeque::new();
                            }
                        }
                    }
                }
            })
        }).collect();

        Self {
            sender,
            ordering_senders,
            workers: Some(workers),
            publish_timeout: config.publish_timeout
        }
    }

    /// publish publishes message.
    /// If an ordering key is specified, it will be added to the queue so that it will be delivered in order.
    pub async fn publish(&self, message: PubsubMessage) -> Awaiter{

        let (producer, consumer) = oneshot::channel();
        if message.ordering_key.is_empty() {
            self.sender.send( ReservedMessage {
                producer,
                message
            }).await;
        }else {
            let key = message.ordering_key.as_str().to_usize();
            let index = key % self.ordering_senders.len();
            self.ordering_senders[index].send(ReservedMessage {
                producer,
                message
            }).await;
        }
        Awaiter::new(self.publish_timeout, consumer)
    }

    /// flush publishes the messages in buffer.
    async fn flush(client: &mut PublisherClient, topic: &str, buffer: VecDeque<ReservedMessage>, retry_setting: Option<RetrySetting>) {
        let mut data = Vec::<PubsubMessage> ::with_capacity(buffer.len());
        let mut callback = Vec::<oneshot::Sender<Result<String,Status>>>::with_capacity(buffer.len());
        buffer.into_iter().for_each(|r| {
            data.push(r.message);
            callback.push(r.producer);
        });
        let result = client.publish(CancellationToken::new(), PublishRequest {
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

    /// stop stops all the tasks.
    pub async fn stop(&mut self) {
        self.sender.close();
        for ps in self.ordering_senders.iter() {
            ps.close();
        }
        if let Some(workers) = self.workers.take() {
            for w in workers {
                w.await;
            }
        }
    }

}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use google_cloud_googleapis::pubsub::v1::PubsubMessage;
    use crate::apiv1::conn_pool::ConnectionManager;
    use serial_test::serial;
    use tokio::task::JoinHandle;
    use crate::apiv1::publisher_client::PublisherClient;
    use crate::publisher::Publisher;

    #[tokio::test]
    #[serial]
    async fn test_publish() -> Result<(), anyhow::Error> {
        std::env::set_var("RUST_LOG","google_cloud_pubsub=trace".to_string());
        env_logger::init();
        let cons = ConnectionManager::new(4, Some("localhost:8681".to_string())).await?;
        let client = PublisherClient::new(cons);

        let publisher = Arc::new(Publisher::new("projects/local-project/topics/test-topic1".to_string(), client, None));

        let joins : Vec<JoinHandle<String>>= (0..10).map(|i| {
            let p = publisher.clone();
            tokio::spawn(async move {
                let mut result = p.publish(PubsubMessage {
                    data: "abc".into(),
                    attributes: Default::default(),
                    message_id: i.to_string(),
                    publish_time: None,
                    ordering_key: "".to_string()
                }).await;
                let v = result.get().await;
                v.unwrap()
            })
        }).collect();
        for j in joins {
            let v = j.await;
            assert!(v.is_ok());
            log::info!("send message id = {}", v.unwrap());
        }
        Ok(())
    }
}