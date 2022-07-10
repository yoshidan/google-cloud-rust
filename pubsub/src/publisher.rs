use async_channel::{Receiver, TryRecvError};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

use tokio::select;
use tokio::sync::Mutex;

use google_cloud_gax::cancel::CancellationToken;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

use google_cloud_gax::grpc::Status;
use google_cloud_gax::retry::RetrySetting;
use google_cloud_googleapis::pubsub::v1::{PublishRequest, PubsubMessage};

use crate::apiv1::publisher_client::PublisherClient;
use crate::util::ToUsize;

pub(crate) struct ReservedMessage {
    pub producer: oneshot::Sender<Result<String, Status>>,
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
    pub async fn get(self, cancel: Option<CancellationToken>) -> Result<String, Status> {
        let onetime = self.consumer;
        let awaited = match cancel {
            Some(cancel) => {
                select! {
                    _ = cancel.cancelled() => return Err(Status::cancelled("cancelled")),
                    v = onetime => v
                }
            }
            None => onetime.await,
        };
        match awaited {
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
    ordering_senders: Arc<Vec<async_channel::Sender<ReservedMessage>>>,
    sender: async_channel::Sender<ReservedMessage>,
    tasks: Arc<Mutex<Tasks>>,
    fqtn: String,
    pubc: PublisherClient,
}

impl Publisher {
    pub(crate) fn new(fqtn: String, pubc: PublisherClient, config: Option<PublisherConfig>) -> Self {
        let config = config.unwrap_or_default();
        let (sender, receiver) = async_channel::unbounded::<ReservedMessage>();
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
            let (sender, receiver) = async_channel::unbounded::<ReservedMessage>();
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
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Vec<String>, Status> {
        self.pubc
            .publish(
                PublishRequest {
                    topic: self.fqtn.clone(),
                    messages,
                },
                cancel,
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
        if self.sender.is_closed() {
            let (tx, rx) = tokio::sync::oneshot::channel();
            drop(tx);
            return Awaiter::new(rx);
        }

        let (producer, consumer) = oneshot::channel();
        if message.ordering_key.is_empty() {
            let _ = self.sender.send(ReservedMessage { producer, message }).await;
        } else {
            let key = message.ordering_key.as_str().to_usize();
            let index = key % self.ordering_senders.len();
            let _ = self.ordering_senders[index]
                .send(ReservedMessage { producer, message })
                .await;
        }
        Awaiter::new(consumer)
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
        receivers: Vec<async_channel::Receiver<ReservedMessage>>,
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
        receiver: Receiver<ReservedMessage>,
        mut client: PublisherClient,
        topic: String,
        retry: Option<RetrySetting>,
        flush_interval: Duration,
        bundle_size: usize,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(flush_interval);
            let mut bundle = VecDeque::<ReservedMessage>::new();
            while !receiver.is_closed() {
                interval_timer.tick().await;

                loop {
                    match receiver.try_recv() {
                        Ok(message) => {
                            bundle.push_back(message);
                            if bundle.len() >= bundle_size {
                                tracing::trace!("maximum buffer {} : {}", bundle.len(), topic);
                                Self::flush(&mut client, topic.as_str(), &mut bundle, retry.clone()).await;
                                debug_assert!(bundle.is_empty());
                                break;
                            }
                        }
                        Err(e) => match e {
                            TryRecvError::Empty => {
                                if !bundle.is_empty() {
                                    tracing::trace!("elapsed: flush buffer : {}", topic);
                                    Self::flush(&mut client, topic.as_str(), &mut bundle, retry.clone()).await;
                                    debug_assert!(bundle.is_empty());
                                }
                                break;
                            }
                            TryRecvError::Closed => {
                                break;
                            }
                        },
                    }
                }
            }

            tracing::trace!("stop publisher : {}", topic);
            if !bundle.is_empty() {
                tracing::trace!("flush rest buffer : {}", topic);
                Self::flush(&mut client, topic.as_str(), &mut bundle, retry.clone()).await;
                debug_assert!(bundle.is_empty());
            }
        })
    }

    /// flush publishes the messages in buffer.
    async fn flush(
        client: &mut PublisherClient,
        topic: &str,
        bundle: &mut VecDeque<ReservedMessage>,
        retry_setting: Option<RetrySetting>,
    ) {
        let mut data = Vec::<PubsubMessage>::with_capacity(bundle.len());
        let mut callback = Vec::<oneshot::Sender<Result<String, Status>>>::with_capacity(bundle.len());

        while let Some(r) = bundle.pop_front() {
            data.push(r.message);
            callback.push(r.producer);
        }

        let req = PublishRequest {
            topic: topic.to_string(),
            messages: data,
        };
        let result = client
            .publish(req, None, retry_setting)
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
                    let status = Status::new(code, &(*status.message()).to_string());
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
