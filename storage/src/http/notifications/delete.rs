use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::Escape;

/// Request message for DeleteNotification.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteNotificationRequest {
    /// Required. The parent bucket of the notification.
    pub bucket: String,
    /// Required. ID of the notification to delete.
    pub notification: String,
}
pub(crate) fn build(base_url: &str, client: &Client, req: &DeleteNotificationRequest) -> RequestBuilder {
    let url = format!(
        "{}/b/{}/notificationConfigs/{}",
        base_url,
        req.bucket.escape(),
        req.notification.escape()
    );
    client.delete(url)
}
