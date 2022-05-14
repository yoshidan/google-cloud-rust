use std::collections::HashMap;

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NotificationCreationConfig {
    /// The Cloud PubSub topic to which this subscription publishes. Formatted as:
    /// '//pubsub.googleapis.com/projects/{project-identifier}/topics/{my-topic}'
    pub topic: String,
    /// If present, only send notifications about listed event types. If empty,
    /// sent notifications for all event types.
    pub event_types: Option<Vec<String>>,
    /// An optional list of additional attributes to attach to each Cloud PubSub
    /// message published for this notification subscription.
    pub custom_attributes: HashMap<String, String>,
    /// If present, only apply this notification configuration to object names that
    /// begin with this prefix.
    pub object_name_prefix: Option<String>,
    /// The desired content of the Payload.
    pub payload_format: String,
}

/// A subscription to receive Google PubSub notifications.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
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
/// The result of a call to Notifications.ListNotifications
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListNotificationsResponse {
    /// The list of items.
    pub items: Vec<Notification>,
}

/// Request message for DeleteNotification.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteNotificationRequest {
    /// Required. The parent bucket of the notification.
    pub bucket: String,
    /// Required. ID of the notification to delete.
    pub notification: String,
}
/// Request message for GetNotification.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetNotificationRequest {
    /// Required. The parent bucket of the notification.
    pub bucket: String,
    /// Required. Notification ID.
    /// Required.
    pub notification: String,
}
/// Request message for InsertNotification.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertNotificationRequest {
    /// Required. The parent bucket of the notification.
    pub bucket: String,
    /// Properties of the notification to be inserted.
    pub notification: NotificationCreationConfig,
}