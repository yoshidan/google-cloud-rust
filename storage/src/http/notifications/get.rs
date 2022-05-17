

use crate::http::{Escape, BASE_URL};
use reqwest::{Client, RequestBuilder};


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
