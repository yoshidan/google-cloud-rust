use crate::http::channels::Channel;
use crate::http::notifications::Notification;
use crate::http::object_access_controls::insert::ObjectAccessControlCreationConfig;
use crate::http::{Escape, BASE_URL};
use reqwest::{Client, RequestBuilder};
use std::collections::HashMap;

/// Request message for GetNotification.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListNotificationsRequest {
    /// Required. The parent bucket of the notification.
    pub bucket: String,
}

/// The result of a call to Notifications.ListNotifications
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListNotificationsResponse {
    /// The list of items.
    pub items: Option<Vec<Notification>>,
}

pub(crate) fn build(client: &Client, req: &ListNotificationsRequest) -> RequestBuilder {
    let url = format!("{}/b/{}/notificationConfigs", BASE_URL, req.bucket.escape());
    client.get(url)
}
