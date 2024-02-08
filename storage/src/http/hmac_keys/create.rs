use reqwest::header::CONTENT_LENGTH;
use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::hmac_keys::HmacKeyMetadata;
use crate::http::Escape;

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateHmacKeyRequest {
    /// Required. The project that the HMAC-owning service account lives in.
    #[serde(skip_serializing)]
    pub project_id: String,
    /// Required. The service account to create the HMAC for.
    pub service_account_email: String,
}
/// Create hmac response.  The only time the secret for an HMAC will be returned.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreateHmacKeyResponse {
    /// Key metadata.
    pub metadata: HmacKeyMetadata,
    /// HMAC key secret material.
    pub secret: String,
}

pub(crate) fn build(base_url: &str, client: &Client, req: &CreateHmacKeyRequest) -> RequestBuilder {
    let url = format!("{}/projects/{}/hmacKeys", base_url, req.project_id.escape());
    client
        .post(url)
        .query(&req)
        // Content-Length header is required
        .header(CONTENT_LENGTH, 0)
}
