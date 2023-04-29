use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_channel::Receiver;
use tokio::sync::oneshot;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::timeout;

use google_cloud_gax::grpc::Status;
use google_cloud_gax::retry::RetrySetting;
use google_cloud_googleapis::pubsub::v1::{PublishRequest, PubsubMessage};

use crate::apiv1::publisher_client::PublisherClient;
use crate::util::ToUsize;

pub(crate) struct ReservedMessage {
    pub producer: oneshot::Sender<Result<String, Status>>,
    pub message: PubsubMessage,
}

pub(crate) enum Reserved {
    Single(ReservedMessage),
    Multi(Vec<ReservedMessage>),
}

#[derive(Debug, Clone)]
pub struct PublisherConfig {
    /// worker count. each workers have gRPC channel
    pub workers: usize,
    /// interval for flush bundle message
    pub flush_interval: Duration,
    /// max bundle size to flush
    pub bundle_size: usize,
    pub retry_setting: Option<RetrySetting>,
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
    consumer: oneshot::Receiver<Result<String, Status>>,
}

impl Awaiter {
    pub(crate) fn new(consumer: oneshot::Receiver<Result<String, Status>>) -> Self {
        Self { consumer }
    }
    pub async fn get(self) -> Result<String, Status> {
        match self.consumer.await {
            Ok(v) => v,
            Err(_e) => Err(Status::cancelled("closed")),
        }
    }
}

/// Publisher is a scheduler which is designed for Pub/Sub's Publish flow.
/// Each item is added with a given key.
/// Items added to the empty string key are handled in random order.
/// Items added to any other key are handled sequentially.
#[derive(Clone, Debug)]
pub struct Publisher {
    ordering_senders: Arc<Vec<async_channel::Sender<Reserved>>>,
    sender: async_channel::Sender<Reserved>,
    tasks: Arc<Mutex<Tasks>>,
    fqtn: String,
    pubc: PublisherClient,
}

impl Publisher {
    pub(crate) fn new(fqtn: String, pubc: PublisherClient, config: Option<PublisherConfig>) -> Self {
        let config = config.unwrap_or_default();
        let (sender, receiver) = async_channel::unbounded::<Reserved>();
        let mut receivers = Vec::with_capacity(1 + config.workers);
        let mut ordering_senders = Vec::with_capacity(config.workers);

        // for non-ordering key message
        for _ in 0..config.workers {
            tracing::trace!("start non-ordering publisher : {}", fqtn.clone());
            receivers.push(receiver.clone());
        }

        // for ordering key message
        for _ in 0..config.workers {
            tracing::trace!("start ordering publisher : {}", fqtn.clone());
            let (sender, receiver) = async_channel::unbounded::<Reserved>();
            receivers.push(receiver);
            ordering_senders.push(sender);
        }

        Self {
            sender,
            ordering_senders: Arc::new(ordering_senders),
            tasks: Arc::new(Mutex::new(Tasks::new(fqtn.clone(), pubc.clone(), receivers, config))),
            fqtn,
            pubc,
        }
    }

    /// publish publishes msg to the topic synchronously
    pub async fn publish_immediately(
        &self,
        messages: Vec<PubsubMessage>,
        retry: Option<RetrySetting>,
    ) -> Result<Vec<String>, Status> {
        self.pubc
            .publish(
                PublishRequest {
                    topic: self.fqtn.clone(),
                    messages,
                },
                retry,
            )
            .await
            .map(|v| v.into_inner().message_ids)
    }

    /// publish publishes msg to the topic asynchronously. Messages are batched and
    /// sent according to the topic's PublisherConfig. Publish never blocks.
    ///
    /// publish returns a non-nil Awaiter which will be ready when the
    /// message has been sent (or has failed to be sent) to the server.
    pub async fn publish(&self, message: PubsubMessage) -> Awaiter {
        let (producer, consumer) = oneshot::channel();
        if message.ordering_key.is_empty() {
            let _ = self
                .sender
                .send(Reserved::Single(ReservedMessage { producer, message }))
                .await;
        } else {
            let key = message.ordering_key.as_str().to_usize();
            let index = key % self.ordering_senders.len();
            let _ = self.ordering_senders[index]
                .send(Reserved::Single(ReservedMessage { producer, message }))
                .await;
        }
        Awaiter::new(consumer)
    }

    /// publish_bulk publishes msg to the topic asynchronously. Messages are batched and
    /// sent according to the topic's PublisherConfig. Publish never blocks.
    ///
    /// publish_bulk returns a non-nil Awaiter which will be ready when the
    /// message has been sent (or has failed to be sent) to the server.
    pub async fn publish_bulk(&self, messages: Vec<PubsubMessage>) -> Vec<Awaiter> {
        let mut awaiters = Vec::with_capacity(messages.len());
        let mut split_by_key = HashMap::<String, Vec<ReservedMessage>>::with_capacity(messages.len());
        for message in messages {
            let (producer, consumer) = oneshot::channel();
            awaiters.push(Awaiter::new(consumer));
            split_by_key
                .entry(message.ordering_key.clone())
                .or_default()
                .push(ReservedMessage { producer, message });
        }

        for e in split_by_key {
            if e.0.is_empty() {
                let _ = self.sender.send(Reserved::Multi(e.1)).await;
            } else {
                let key = e.0.as_str().to_usize();
                let index = key % self.ordering_senders.len();
                let _ = self.ordering_senders[index].send(Reserved::Multi(e.1)).await;
            }
        }
        awaiters
    }

    pub async fn shutdown(&mut self) {
        self.sender.close();
        for s in self.ordering_senders.iter() {
            s.close();
        }
        self.tasks.lock().await.done().await;
    }
}

#[derive(Debug)]
struct Tasks {
    inner: Option<Vec<JoinHandle<()>>>,
}

impl Tasks {
    pub fn new(
        topic: String,
        pubc: PublisherClient,
        receivers: Vec<async_channel::Receiver<Reserved>>,
        config: PublisherConfig,
    ) -> Self {
        let tasks = receivers
            .into_iter()
            .map(|receiver| {
                Self::run_task(
                    receiver,
                    pubc.clone(),
                    topic.clone(),
                    config.retry_setting.clone(),
                    config.flush_interval,
                    config.bundle_size,
                )
            })
            .collect();

        Self { inner: Some(tasks) }
    }

    fn run_task(
        receiver: Receiver<Reserved>,
        mut client: PublisherClient,
        topic: String,
        retry: Option<RetrySetting>,
        flush_interval: Duration,
        bundle_size: usize,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut bundle = Vec::<ReservedMessage>::with_capacity(bundle_size);
            while !receiver.is_closed() {
                let result = match timeout(flush_interval, &mut receiver.recv()).await {
                    Ok(result) => result,
                    //timed out
                    Err(_e) => {
                        if !bundle.is_empty() {
                            tracing::trace!("elapsed: flush buffer : {}", topic);
                            Self::flush(&mut client, topic.as_str(), bundle, retry.clone()).await;
                            bundle = Vec::new();
                        }
                        continue;
                    }
                };
                match result {
                    Ok(reserved) => {
                        match reserved {
                            Reserved::Single(message) => bundle.push(message),
                            Reserved::Multi(messages) => bundle.extend(messages),
                        }
                        if bundle.len() >= bundle_size {
                            tracing::trace!("maximum buffer {} : {}", bundle.len(), topic);
                            Self::flush(&mut client, topic.as_str(), bundle, retry.clone()).await;
                            bundle = Vec::new();
                        }
                    }
                    //closed
                    Err(_e) => break,
                };
            }

            tracing::trace!("stop publisher : {}", topic);
            if !bundle.is_empty() {
                tracing::trace!("flush rest buffer : {}", topic);
                Self::flush(&mut client, topic.as_str(), bundle, retry.clone()).await;
            }
        })
    }

    /// flush publishes the messages in buffer.
    async fn flush(
        client: &mut PublisherClient,
        topic: &str,
        bundle: Vec<ReservedMessage>,
        retry_setting: Option<RetrySetting>,
    ) {
        let mut data = Vec::<PubsubMessage>::with_capacity(bundle.len());
        let mut callback = Vec::<oneshot::Sender<Result<String, Status>>>::with_capacity(bundle.len());
        bundle.into_iter().for_each(|r| {
            data.push(r.message);
            callback.push(r.producer);
        });
        let req = PublishRequest {
            topic: topic.to_string(),
            messages: data,
        };
        let result = client
            .publish(req, retry_setting)
            .await
            .map(|v| v.into_inner().message_ids);

        // notify to receivers
        match result {
            Ok(message_ids) => {
                for (i, p) in callback.into_iter().enumerate() {
                    let message_id = &message_ids[i];
                    if p.send(Ok(message_id.to_string())).is_err() {
                        tracing::error!("failed to notify : id={message_id}");
                    }
                }
            }
            Err(status) => {
                for p in callback.into_iter() {
                    let code = status.code();
                    let status = Status::new(code, (*status.message()).to_string());
                    if p.send(Err(status)).is_err() {
                        tracing::error!("failed to notify : status={}", code);
                    }
                }
            }
        };
    }

    /// done waits for all the workers finish.
    pub async fn done(&mut self) {
        if let Some(tasks) = self.inner.take() {
            for task in tasks {
                let _ = task.await;
            }
        }
    }
}
