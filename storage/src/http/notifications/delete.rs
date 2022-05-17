use crate::http::{Escape, BASE_URL};
use reqwest::{Client, RequestBuilder};

/// Request message for DeleteNotification.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteNotificationRequest {
    /// Required. The parent bucket of the notification.
    pub bucket: String,
    /// Required. ID of the notification to delete.
    pub notification: String,
}
pub(crate) fn build(client: &Client, req: &DeleteNotificationRequest) -> RequestBuilder {
    let url = format!(
        "{}/b/{}/notificationConfigs/{}",
        BASE_URL,
        req.bucket.escape(),
        req.notification.escape()
    );
    client.delete(url)
}
