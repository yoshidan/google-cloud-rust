use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

use tokio::select;

use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::timeout;

use google_cloud_gax::retry::RetrySetting;
use google_cloud_gax::status::Status;
use google_cloud_googleapis::pubsub::v1::{PublishRequest, PubsubMessage};
use tokio_util::sync::CancellationToken;

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
#[derive(Clone)]
pub struct Publisher {
    ordering_senders: Arc<Vec<async_channel::Sender<ReservedMessage>>>,
    sender: async_channel::Sender<ReservedMessage>,
    worker: Arc<Mutex<Worker>>,
}

impl Publisher {
    pub(crate) fn new(
        fqtn: String,
        pubc: PublisherClient,
        config: Option<PublisherConfig>,
    ) -> Self {
        let config = config.unwrap_or_default();
        let (sender, receiver) = async_channel::unbounded::<ReservedMessage>();
        let mut receivers = Vec::with_capacity(1 + config.workers);
        let mut ordering_senders = Vec::with_capacity(config.workers);

        // for non-ordering key message
        for _ in 0..config.workers {
            log::trace!("start non-ordering publisher : {}", fqtn.clone());
            receivers.push(receiver.clone());
        }

        // for ordering key message
        for _ in 0..config.workers {
            log::trace!("start ordering publisher : {}", fqtn.clone());
            let (sender, receiver) = async_channel::unbounded::<ReservedMessage>();
            receivers.push(receiver);
            ordering_senders.push(sender);
        }

        Self {
            sender,
            ordering_senders: Arc::new(ordering_senders),
            worker: Arc::new(Mutex::new(Worker::start(
                fqtn.to_string(),
                pubc,
                receivers,
                config,
            ))),
        }
    }

    /// publish publishes msg to the topic asynchronously. Messages are batched and
    /// sent according to the topic's PublisherConfig. Publish never blocks.
    ///
    /// publish returns a non-nil Awaiter which will be ready when the
    /// message has been sent (or has failed to be sent) to the server.
    ///
    /// publish creates tasks for batching and sending messages. These tasks
    /// need to be stopped by calling t.stop(). Once stopped, future calls to Publish
    /// will immediately return a Awaiter with an error.
    pub async fn publish(&self, message: PubsubMessage) -> Awaiter {
        if self.sender.is_closed() {
            let (mut tx, rx) = tokio::sync::oneshot::channel();
            tx.closed();
            return Awaiter::new(rx);
        }

        let (producer, consumer) = oneshot::channel();
        if message.ordering_key.is_empty() {
            self.sender
                .send(ReservedMessage { producer, message })
                .await;
        } else {
            let key = message.ordering_key.as_str().to_usize();
            let index = key % self.ordering_senders.len();
            self.ordering_senders[index]
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
        self.worker.lock().done().await;
    }
}

struct Worker {
    tasks: Option<Vec<JoinHandle<()>>>,
}

impl Worker {
    pub fn start(
        topic: String,
        pubc: PublisherClient,
        receivers: Vec<async_channel::Receiver<ReservedMessage>>,
        config: PublisherConfig,
    ) -> Self {
        let tasks = receivers
            .into_iter()
            .map(|receiver| {
                let mut client = pubc.clone();
                let topic_for_worker = topic.clone();
                let retry_setting = config.retry_setting.clone();
                tokio::spawn(async move {
                    let mut bundle = VecDeque::<ReservedMessage>::new();
                    while !receiver.is_closed() {
                        let result =
                            match timeout(config.flush_interval, &mut receiver.recv()).await {
                                Ok(result) => result,
                                //timed out
                                Err(_e) => {
                                    if !bundle.is_empty() {
                                        log::trace!("elapsed: flush buffer : {}", topic_for_worker);
                                        Self::flush(
                                            &mut client,
                                            topic_for_worker.as_str(),
                                            bundle,
                                            retry_setting.clone(),
                                        )
                                        .await;
                                        bundle = VecDeque::new();
                                    }
                                    continue;
                                }
                            };
                        match result {
                            Ok(message) => {
                                bundle.push_back(message);
                                if bundle.len() >= config.bundle_size {
                                    log::trace!(
                                        "maximum buffer {} : {}",
                                        bundle.len(),
                                        topic_for_worker
                                    );
                                    Self::flush(
                                        &mut client,
                                        topic_for_worker.as_str(),
                                        bundle,
                                        retry_setting.clone(),
                                    )
                                    .await;
                                    bundle = VecDeque::new();
                                }
                            }
                            //closed
                            Err(_e) => break,
                        };
                    }

                    log::trace!("stop publisher : {}", topic_for_worker);
                    if !bundle.is_empty() {
                        log::trace!("flush rest buffer : {}", topic_for_worker);
                        Self::flush(
                            &mut client,
                            topic_for_worker.as_str(),
                            bundle,
                            retry_setting.clone(),
                        )
                        .await;
                    }
                })
            })
            .collect();

        Self { tasks: Some(tasks) }
    }

    /// flush publishes the messages in buffer.
    async fn flush(
        client: &mut PublisherClient,
        topic: &str,
        bundle: VecDeque<ReservedMessage>,
        retry_setting: Option<RetrySetting>,
    ) {
        let mut data = Vec::<PubsubMessage>::with_capacity(bundle.len());
        let mut callback =
            Vec::<oneshot::Sender<Result<String, Status>>>::with_capacity(bundle.len());
        bundle.into_iter().for_each(|r| {
            data.push(r.message);
            callback.push(r.producer);
        });
        let result = client
            .publish(
                CancellationToken::new(),
                PublishRequest {
                    topic: topic.to_string(),
                    messages: data,
                },
                retry_setting,
            )
            .await
            .map(|v| v.into_inner().message_ids);

        // notify to receivers
        match result {
            Ok(message_ids) => {
                for (i, p) in callback.into_iter().enumerate() {
                    p.send(Ok(message_ids[i].to_string()));
                }
            }
            Err(status) => {
                for p in callback.into_iter() {
                    p.send(Err(Status::new(tonic::Status::new(
                        status.source.code().clone(),
                        &(*status.source.message()).to_string(),
                    ))));
                }
            }
        };
    }

    /// done waits for all the workers finish.
    pub async fn done(&mut self) {
        if let Some(tasks) = self.tasks.take() {
            for task in tasks {
                task.await;
            }
        }
    }
}
