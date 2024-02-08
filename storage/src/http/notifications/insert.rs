use std::collections::HashMap;

use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::notifications::{EventType, PayloadFormat};
use crate::http::Escape;

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Default, Debug)]
pub struct NotificationCreationConfig {
    /// The Cloud PubSub topic to which this subscription publishes. Formatted as:
    /// '//pubsub.googleapis.com/projects/{project-identifier}/topics/{my-topic}'
    pub topic: String,
    /// If present, only send notifications about listed event types. If empty,
    /// sent notifications for all event types.
    pub event_types: Option<Vec<EventType>>,
    /// An optional list of additional attributes to attach to each Cloud PubSub
    /// message published for this notification subscription.
    pub custom_attributes: HashMap<String, String>,
    /// If present, only apply this notification configuration to object names that
    /// begin with this prefix.
    pub object_name_prefix: Option<String>,
    /// The desired content of the Payload.
    pub payload_format: PayloadFormat,
}

/// Request message for InsertNotification.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertNotificationRequest {
    /// Required. The parent bucket of the notification.
    pub bucket: String,
    /// Properties of the notification to be inserted.
    pub notification: NotificationCreationConfig,
}

pub(crate) fn build(base_url: &str, client: &Client, req: &InsertNotificationRequest) -> RequestBuilder {
    let url = format!("{}/b/{}/notificationConfigs", base_url, req.bucket.escape());
    client.post(url).json(&req.notification)
}
