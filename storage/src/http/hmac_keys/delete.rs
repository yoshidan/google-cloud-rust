use crate::http::{Escape, BASE_URL};
use reqwest::{Client, RequestBuilder};

/// Request object to delete a given HMAC key.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteHmacKeyRequest {
    /// Required. The identifying key for the HMAC to delete.
    pub access_id: String,
    /// Required. The project id the HMAC key lies in.
    pub project_id: String,
}
pub(crate) fn build(client: &Client, req: &DeleteHmacKeyRequest) -> RequestBuilder {
    let url = format!(
        "{}/projects/{}/hmacKeys/{}",
        BASE_URL,
        req.project_id.escape(),
        req.access_id.escape()
    );
    client.delete(url)
}
