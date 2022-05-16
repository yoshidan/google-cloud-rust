use std::collections::HashMap;

pub mod delete;
pub mod get;
pub mod insert;
pub mod list;

/// A subscription to receive Google PubSub notifications.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
pub struct Notification {
    /// The Cloud PubSub topic to which this subscription publishes. Formatted as:
    /// '//pubsub.googleapis.com/projects/{project-identifier}/topics/{my-topic}'
    pub topic: String,
    /// If present, only send notifications about listed event types. If empty,
    /// sent notifications for all event types.
    pub event_types: Option<Vec<String>>,
    /// An optional list of additional attributes to attach to each Cloud PubSub
    /// message published for this notification subscription.
    pub custom_attributes: Option<HashMap<String, String>>,
    /// HTTP 1.1 \[<https://tools.ietf.org/html/rfc7232#section-2.3\][Entity> tag]
    /// for this subscription notification.
    pub etag: String,
    /// If present, only apply this notification configuration to object names that
    /// begin with this prefix.
    pub object_name_prefix: Option<String>,
    /// The desired content of the Payload.
    pub payload_format: String,
    /// The ID of the notification.
    pub id: String,
}
