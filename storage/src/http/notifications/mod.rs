use std::collections::HashMap;

pub mod delete;
pub mod get;
pub mod insert;
pub mod list;

/// A subscription to receive Google PubSub notifications.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
pub struct Notification {
    /// The Cloud PubSub topic to which this subscription publishes. Formatted as:
    /// '//pubsub.googleapis.com/projects/{project-identifier}/topics/{my-topic}'
    pub topic: String,
    /// If present, only send notifications about listed event types. If empty,
    /// sent notifications for all event types.
    pub event_types: Option<Vec<EventType>>,
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
    pub payload_format: PayloadFormat,
    /// The ID of the notification.
    pub id: String,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EventType {
    /// Sent when a new object (or a new generation of an existing object) is successfully created in the bucket. This includes copying or rewriting an existing object. A failed upload does not trigger this event.
    ObjectFinalize,
    /// Sent when the metadata of an existing object changes.
    ObjectMetadataUpdate,
    /// Sent when an object has been permanently deleted. This includes objects that are replaced or are deleted as part of the bucket's lifecycle configuration. For buckets with object versioning enabled, this is not sent when an object becomes noncurrent (see OBJECT_ARCHIVE), even if the object becomes noncurrent via the storage.objects.delete method.
    ObjectDelete,
    /// Only sent when a bucket has enabled object versioning. This event indicates that the live version of an object has become a noncurrent version, either because it was explicitly made noncurrent or because it was replaced by the upload of an object of the same name.
    ObjectArchive,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PayloadFormat {
    /// The payload will be a UTF-8 string containing the resource representation of the object’s metadata.
    JsonApiV1,
    /// No payload is included with the notification.
    None,
}

impl Default for PayloadFormat {
    fn default() -> Self {
        Self::JsonApiV1
    }
}
