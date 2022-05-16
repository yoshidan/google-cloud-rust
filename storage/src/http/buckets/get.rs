use percent_encoding::utf8_percent_encode;
use reqwest::{Client, RequestBuilder};
use crate::http::{BASE_URL, Error, Escape};
use crate::http::object_access_controls::Projection;

/// Request message for DeleteBucket.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Default)]
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

pub(crate) fn build(client: &Client, req: &GetBucketRequest) -> RequestBuilder {
    let url = format!("{}/b/{}", BASE_URL, req.bucket.escape());
    client.get(url).query(&req)
}