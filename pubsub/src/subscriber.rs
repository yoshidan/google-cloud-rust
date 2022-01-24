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
use google_cloud_googleapis::pubsub::v1::{PublishRequest, PublishResponse, PubsubMessage, PullRequest, StreamingPullRequest, StreamingPullResponse};
use google_cloud_googleapis::{Code, Status};
use crate::apiv1::publisher_client::PublisherClient;
use crate::apiv1::subscriber_client::SubscriberClient;

pub struct Subscriber {
   workers: Vec<JoinHandle<()>>
}

impl Subscriber {
    pub fn new(subsciption: String, subc: SubscriberClient) -> Subscriber {
        let workers = (0..3).map(|_| {
            let mut client = subc.clone();
            let subscription_for_worker = subsciption.clone();
            tokio::spawn(async move {
                println!("start subscriber");
                let request = StreamingPullRequest {
                    subscription: subscription_for_worker.to_string(),
                    ack_ids: vec![],
                    modify_deadline_seconds: vec![],
                    modify_deadline_ack_ids: vec![],
                    stream_ack_deadline_seconds: 10,
                    client_id: "".to_string(),
                    max_outstanding_messages: 1000,
                    max_outstanding_bytes: 1000 * 1000 * 1000
                };
                let response2 = client.pull(PullRequest {
                    subscription: subscription_for_worker,
                    return_immediately: false,
                    max_messages: 10
                }, None).await;

             //   let m = response2.unwrap().into_inner().received_messages;
              //  println!("{}", m.len());
                let response = client.streaming_pull(request, None).await;

                match response {
                    Ok(r) => {
                        let mut stream = r.into_inner();
                        loop {

                            if let Some(message) = stream.message().await.unwrap()
                            {
                                for m in message.received_messages {
                                    if let Some(mes) = m.message {
                                        println!("recv {}", mes.message_id);
                                    }
                                }
                            }else {
                              //  println!("recv nothing");
                                tokio::time::sleep(std::time::Duration::from_millis(10));
                            }
                        }
                    },
                    Err(e)=> {
                        print!("{:?}", e)
                    }
                };
                ()
            })
        }).collect();
        Self {
            workers,
        }
    }
}