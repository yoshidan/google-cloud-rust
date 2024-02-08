use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::notifications::Notification;
use crate::http::Escape;

/// Request message for GetNotification.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListNotificationsRequest {
    /// Required. The parent bucket of the notification.
    pub bucket: String,
}

/// The result of a call to Notifications.ListNotifications
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListNotificationsResponse {
    /// The list of items.
    pub items: Option<Vec<Notification>>,
}

pub(crate) fn build(base_url: &str, client: &Client, req: &ListNotificationsRequest) -> RequestBuilder {
    let url = format!("{}/b/{}/notificationConfigs", base_url, req.bucket.escape());
    client.get(url)
}
