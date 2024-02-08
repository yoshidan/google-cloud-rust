use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::Escape;

/// Request message for LockRetentionPolicy.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LockRetentionPolicyRequest {
    /// Required. Name of a bucket.
    #[serde(skip_serializing)]
    pub bucket: String,
    /// Makes the operation conditional on whether bucket's current metageneration
    /// matches the given value. Must be positive.
    pub if_metageneration_match: i64,
}

#[allow(dead_code)]
pub(crate) fn build(base_url: &str, client: &Client, req: &LockRetentionPolicyRequest) -> RequestBuilder {
    let url = format!("{}/b/{}/lockRetentionPolicy", base_url, req.bucket.escape());
    client.post(url).query(&req)
}
