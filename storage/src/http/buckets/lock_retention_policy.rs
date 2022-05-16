use crate::http::buckets::Policy;
use crate::http::object_access_controls::Projection;
use crate::http::{Error, Escape, BASE_URL};
use percent_encoding::utf8_percent_encode;
use reqwest::{Client, RequestBuilder};

/// Request message for LockRetentionPolicy.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LockRetentionPolicyRequest {
    /// Required. Name of a bucket.
    #[serde(skip_serializing)]
    pub bucket: String,
    /// Makes the operation conditional on whether bucket's current metageneration
    /// matches the given value. Must be positive.
    pub if_metageneration_match: i64,
}

pub(crate) fn build(client: &Client, req: &LockRetentionPolicyRequest) -> RequestBuilder {
    let url = format!("{}/b/{}/lockRetentionPolicy", BASE_URL, req.bucket.escape());
    client.post(url).query(&req)
}
