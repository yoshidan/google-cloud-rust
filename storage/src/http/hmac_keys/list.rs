use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::hmac_keys::HmacKeyMetadata;
use crate::http::Escape;

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListHmacKeysRequest {
    /// Required. The project id to list HMAC keys for.
    pub project_id: String,
    /// An optional filter to only return HMAC keys for one service account.
    pub service_account_email: Option<String>,
    /// An optional bool to return deleted keys that have not been wiped out yet.
    pub show_deleted_keys: Option<bool>,
    /// The maximum number of keys to return.
    pub max_results: Option<i32>,
    /// A previously returned token from ListHmacKeysResponse to get the next page.
    pub page_token: Option<String>,
}

/// Hmac key list response with next page information.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListHmacKeysResponse {
    /// The continuation token, used to page through large result sets. Provide
    /// this value in a subsequent request to return the next page of results.
    pub next_page_token: Option<String>,
    /// The list of items.
    pub items: Option<Vec<HmacKeyMetadata>>,
}

pub(crate) fn build(base_url: &str, client: &Client, req: &ListHmacKeysRequest) -> RequestBuilder {
    let url = format!("{}/projects/{}/hmacKeys", base_url, req.project_id.escape());
    client.get(url)
}
