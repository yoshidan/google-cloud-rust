use std::collections::VecDeque;
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use parking_lot::{Mutex, RwLock};
use prost::Message;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use google_cloud_googleapis::pubsub::v1::{PublishRequest, PublishResponse, PubsubMessage, PullRequest, ReceivedMessage, StreamingPullRequest, StreamingPullResponse};
use google_cloud_googleapis::{Code, Status};
use crate::apiv1::publisher_client::PublisherClient;
use crate::apiv1::subscriber_client::{create_default_streaming_pull_request, SubscriberClient};

pub struct Config {
    pub worker_count: i32,
    pub ping_interval_second: u64
}

impl Default for Config {
    fn default() -> Self {
        return Self {
            worker_count: 3,
            ping_interval_second: 10
        }
    }
}

struct Worker {
    pub ping_handle: JoinHandle<()>,
    pub receive_handle: JoinHandle<()>
}

pub struct Subscriber {
   workers: Vec<Worker>,
   stopped: bool
}

impl Subscriber {
    pub fn new(subscription: String, client: SubscriberClient, queue:async_channel::Sender<PubsubMessage> , config: Config) -> Subscriber {
        let workers = (0..config.worker_count).map(|_| {
            Self::start_worker(subscription.clone(), client.clone(), queue.clone(), config.ping_interval_second)
        }).collect();
        Self {
            workers,
            stopped: false
        }
    }

    fn start_worker(subscription: String, mut client: SubscriberClient, mut queue: async_channel::Sender<PubsubMessage>, ping_interval_second: u64) -> Worker {
        let (ping_sender,ping_receiver) = async_channel::unbounded();

        // ping request
        let ping_handle = tokio::spawn(async move {
            let result = ping_sender.send(true).await;
            if result.is_err() {
               log::debug!("receiver closed {:?}", result.unwrap_err())
            }
            tokio::time::sleep(std::time::Duration::from_secs(ping_interval_second)).await;
        });

        let receive_handle = tokio::spawn(async move {
            println!("start subscriber");
            let request = create_default_streaming_pull_request(subscription);
            let response = client.streaming_pull(request, ping_receiver, None).await;

            match response {
                Ok(r) => {
                    let mut stream = r.into_inner();
                    loop {
                        if let Some(message) = stream.message().await.unwrap()
                        {
                            for m in message.received_messages {
                                if let Some(mes) = m.message {
                                    log::debug!("message received: {}", mes.message_id);
                                    queue.send(mes).await;
                                }
                            }
                        }else {
                            println!("may be receiver closed");
                            break;
                        }
                    }
                },
                Err(e)=> {
                    println!("subscribe error {:?}", e)
                }
            };
            ()
        });
        return Worker {
            ping_handle,
            receive_handle
        }
    }

    pub fn stop(&mut self) {
        if !self.stopped {
            return
        }
       for worker in &self.workers {
           worker.ping_handle.abort();
           worker.receive_handle.abort();
       }
        self.stopped = true
    }

    fn drop(&mut self) {
        self.stop()
    }
}