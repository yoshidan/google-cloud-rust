use crate::http::channels::Channel;
use crate::http::object_access_controls::insert::ObjectAccessControlCreationConfig;
use crate::http::{Escape, BASE_URL};
use reqwest::{Client, RequestBuilder};
use std::collections::HashMap;

/// Request message for GetNotification.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetNotificationRequest {
    /// Required. The parent bucket of the notification.
    pub bucket: String,
    /// Required. Notification ID.
    pub notification: String,
}

pub(crate) fn build(client: &Client, req: &GetNotificationRequest) -> RequestBuilder {
    let url = format!(
        "{}/b/{}/notificationConfigs/{}",
        BASE_URL,
        req.bucket.escape(),
        req.notification.escape()
    );
    client.get(url)
}
