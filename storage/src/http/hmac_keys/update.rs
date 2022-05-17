use crate::http::hmac_keys::HmacKeyMetadata;
use crate::http::{Escape, BASE_URL};
use reqwest::{Client, RequestBuilder};
use sha2::digest::Update;

/// Request object to update an HMAC key state.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateHmacKeyRequest {
    /// Required. The id of the HMAC key.
    pub access_id: String,
    /// Required. The project id the HMAC's service account lies in.
    pub project_id: String,
    /// Required. The service account owner of the HMAC key.
    pub metadata: HmacKeyMetadata,
}

pub(crate) fn build(client: &Client, req: &UpdateHmacKeyRequest) -> RequestBuilder {
    let url = format!(
        "{}/projects/{}/hmacKeys/{}",
        BASE_URL,
        req.project_id.escape(),
        req.access_id.escape()
    );
    client.put(url).json(&req.metadata)
}