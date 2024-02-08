use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::Escape;

/// Request object to delete a given HMAC key.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteHmacKeyRequest {
    /// Required. The identifying key for the HMAC to delete.
    pub access_id: String,
    /// Required. The project id the HMAC key lies in.
    pub project_id: String,
}
pub(crate) fn build(base_url: &str, client: &Client, req: &DeleteHmacKeyRequest) -> RequestBuilder {
    let url = format!(
        "{}/projects/{}/hmacKeys/{}",
        base_url,
        req.project_id.escape(),
        req.access_id.escape()
    );
    client.delete(url)
}
