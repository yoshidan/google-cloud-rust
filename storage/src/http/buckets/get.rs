use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::object_access_controls::Projection;
use crate::http::Escape;

/// Request message for DeleteBucket.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetBucketRequest {
    /// Required. Name of a bucket.
    #[serde(skip_serializing)]
    pub bucket: String,
    /// If set, only deletes the bucket if its metageneration matches this value.
    pub if_metageneration_match: Option<i64>,
    /// If set, only deletes the bucket if its metageneration does not match this
    /// value.
    pub if_metageneration_not_match: Option<i64>,
    /// Set of properties to return. Defaults to `NO_ACL`.
    pub projection: Option<Projection>,
}

pub(crate) fn build(base_url: &str, client: &Client, req: &GetBucketRequest) -> RequestBuilder {
    let url = format!("{}/b/{}", base_url, req.bucket.escape());
    client.get(url).query(&req)
}
