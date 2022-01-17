use std::ops::Sub;
use std::sync::Arc;
use crate::apiv1::subscriber_client::SubscriberClient;

/// Subscription is a reference to a PubSub subscription.
pub struct Subscription {
    name: String,
    subc: Arc<SubscriberClient<>>
}

impl Subscription {
    pub(crate) fn new(name: String, subc: Arc<SubscriberClient>) -> Self {
        Self {
            name,
            subc
        }
    }
}
