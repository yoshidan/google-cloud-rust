use std::collections::{VecDeque};
use std::time::Duration;

use tokio::select;

use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::timeout;

use tokio_util::sync::CancellationToken;
use google_cloud_googleapis::pubsub::v1::{PublishRequest, PubsubMessage};
use google_cloud_googleapis::{Status};

use crate::apiv1::publisher_client::PublisherClient;
use crate::apiv1::RetrySetting;


pub(crate) struct ReservedMessage {
    pub producer: oneshot::Sender<Result<String,Status>>,
    pub message: PubsubMessage,
}

#[derive(Clone)]
pub struct PublisherConfig {
    /// worker count. each workers have gRPC channel
    pub workers: usize,
    /// interval for flush bundle message
    pub flush_interval: Duration,
    /// max bundle size to flush
    pub bundle_size: usize,
    pub retry_setting: Option<RetrySetting>
}

impl Default for PublisherConfig {
    fn default() -> Self {
        Self {
            workers: 3,
            flush_interval: Duration::from_millis(100),
            bundle_size: 3,
            retry_setting: None,
        }
    }
}

pub struct Awaiter {
    consumer: oneshot::Receiver<Result<String,Status>>,
}

impl Awaiter {
    pub(crate) fn new(consumer: oneshot::Receiver<Result<String,Status>>) -> Self {
        Self {
            consumer,
        }
    }
    pub async fn get(self, ctx: CancellationToken) -> Result<String, Status> {
        let onetime = self.consumer;
        select! {
            _ = ctx.cancelled() => Err(tonic::Status::cancelled("cancelled").into()),
            v = onetime => match v {
                Ok(vv) => vv,
                Err(_e) => Err(tonic::Status::cancelled("closed").into())
            }
        }
    }
}

/// Publisher is a scheduler which is designed for Pub/Sub's Publish flow.
/// Each item is added with a given key.
/// Items added to the empty string key are handled in random order.
/// Items added to any other key are handled sequentially.
pub(crate) struct Publisher {
    workers: Option<Vec<JoinHandle<()>>>
}

impl Publisher {

    pub fn start(topic: String, pubc: PublisherClient, receivers: Vec<async_channel::Receiver<ReservedMessage>>, config: PublisherConfig) -> Self {

        let workers = receivers.into_iter().map(|receiver| {
            let mut client = pubc.clone();
            let topic_for_worker = topic.clone();
            let retry_setting = config.retry_setting.clone();
            tokio::spawn(async move {
                let mut bundle = VecDeque::<ReservedMessage>::new();
                while !receiver.is_closed() {
                    match timeout(config.flush_interval,&mut receiver.recv()).await {
                        Ok(result) => match result {
                            Ok(message) => {
                                bundle.push_back(message);
                                if bundle.len() >= config.bundle_size {
                                    log::trace!("maximum buffer {} : {}", bundle.len(), topic_for_worker);
                                    Self::flush(&mut client, topic_for_worker.as_str(), bundle, retry_setting.clone()).await;
                                    bundle = VecDeque::new();
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
                            if !bundle.is_empty() {
                                log::trace!("elapsed: flush buffer : {}", topic_for_worker);
                                Self::flush(&mut client, topic_for_worker.as_str(), bundle, retry_setting.clone()).await;
                                bundle = VecDeque::new();
                            }
                        }
                    }
                }
            })
        }).collect();

        Self {
            workers: Some(workers)
        }
    }

    /// flush publishes the messages in buffer.
    async fn flush(client: &mut PublisherClient, topic: &str, bundle: VecDeque<ReservedMessage>, retry_setting: Option<RetrySetting>) {
        let mut data = Vec::<PubsubMessage> ::with_capacity(bundle.len());
        let mut callback = Vec::<oneshot::Sender<Result<String,Status>>>::with_capacity(bundle.len());
        bundle.into_iter().for_each(|r| {
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

    /// shutdown stops all the tasks.
    pub async fn shutdown(&mut self) {
        if let Some(workers ) = self.workers.take() {
            for worker in workers {
                worker.await;
            }
        }
    }

}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;
    use google_cloud_googleapis::pubsub::v1::PubsubMessage;
    use crate::apiv1::conn_pool::ConnectionManager;
    use serial_test::serial;
    use tokio::sync::oneshot;
    use tokio::task::JoinHandle;
    use tokio_util::sync::CancellationToken;
    use crate::apiv1::publisher_client::PublisherClient;
    use crate::publisher::{Awaiter, Publisher, PublisherConfig, ReservedMessage};

    #[tokio::test]
    #[serial]
    async fn test_publish() -> Result<(), anyhow::Error> {
        std::env::set_var("RUST_LOG","google_cloud_pubsub=trace".to_string());
        env_logger::init();
        let cons = ConnectionManager::new(4, Some("localhost:8681".to_string())).await?;
        let client = PublisherClient::new(cons);

        let (sender, receiver) = async_channel::unbounded::<ReservedMessage>();
        let publisher = Arc::new(Publisher::start("projects/local-project/topics/test-topic1".to_string(), client, vec![receiver.clone(),receiver], PublisherConfig::default()));

        let ctx = CancellationToken::new();
        let joins : Vec<JoinHandle<String>> = (0..10).map(|i| {
            let _p = publisher.clone();
            let ctx = ctx.clone();
            let s = sender.clone();
            tokio::spawn(async move {
                let message = PubsubMessage {
                    data: "abc".into(),
                    attributes: Default::default(),
                    message_id: i.to_string(),
                    publish_time: None,
                    ordering_key: "".to_string()
                };
                let (producer, consumer) = oneshot::channel();
                s.send( ReservedMessage {
                    producer,
                    message
                }).await;
                Awaiter::new(consumer).get(ctx.clone()).await.unwrap()
            })
        }).collect();
        for j in joins {
            let v = j.await;
            assert!(v.is_ok());
            log::info!("send message id = {}", v.unwrap());
        }
        Ok(())
    }

    #[tokio::test]
    #[serial]
    async fn test_publish_cancel() -> Result<(), anyhow::Error> {
        std::env::set_var("RUST_LOG","google_cloud_pubsub=trace".to_string());
        env_logger::init();
        let cons = ConnectionManager::new(4, Some("localhost:8681".to_string())).await?;
        let client = PublisherClient::new(cons);

        let mut opt = PublisherConfig::default();
        opt.flush_interval = Duration::from_secs(10);
        let (sender, receiver) = async_channel::unbounded::<ReservedMessage>();
        let _publisher = Arc::new(Publisher::start("projects/local-project/topics/test-topic1".to_string(), client, vec![receiver], opt));
        let ctx = CancellationToken::new();
        let message = PubsubMessage {
            data: "abc".into(),
            attributes: Default::default(),
            message_id: "".to_string(),
            publish_time: None,
            ordering_key: "".to_string()
        };
        let (producer, consumer) = oneshot::channel();
        sender.send( ReservedMessage {
            producer,
            message
        }).await;
        let result = Awaiter::new(consumer);
        let child = ctx.clone();
        let j = tokio::spawn(async move {
            let v = result.get(child).await;
            assert!(v.is_err());
            println!("{}", v.unwrap_err());
        });
        ctx.cancel();
        j.await;
        Ok(())
    }
}