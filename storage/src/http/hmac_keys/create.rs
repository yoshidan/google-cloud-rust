use crate::http::hmac_keys::HmacKeyMetadata;
use crate::http::{Escape, BASE_URL};
use reqwest::header::CONTENT_LENGTH;
use reqwest::{Client, RequestBuilder};

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateHmacKeyRequest {
    /// Required. The project that the HMAC-owning service account lives in.
    #[serde(skip_serializing)]
    pub project_id: String,
    /// Required. The service account to create the HMAC for.
    pub service_account_email: String,
}
/// Create hmac response.  The only time the secret for an HMAC will be returned.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateHmacKeyResponse {
    /// Key metadata.
    pub metadata: HmacKeyMetadata,
    /// HMAC key secret material.
    pub secret: String,
}

pub(crate) fn build(client: &Client, req: &CreateHmacKeyRequest) -> RequestBuilder {
    let url = format!("{}/projects/{}/hmacKeys", BASE_URL, req.project_id.escape());
    client
        .post(url)
        .query(&req)
        // Content-Length header is required
        .header(CONTENT_LENGTH, 0)
}
