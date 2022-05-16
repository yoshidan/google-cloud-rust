use crate::http::{Error, Escape, BASE_URL};
use percent_encoding::utf8_percent_encode;
use reqwest::{Client, RequestBuilder};

/// Request message for DeleteBucket.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DeleteBucketParam {
    /// If set, only deletes the bucket if its metageneration matches this value.
    pub if_metageneration_match: Option<i64>,
    /// If set, only deletes the bucket if its metageneration does not match this
    /// value.
    pub if_metageneration_not_match: Option<i64>,
}

/// Request message for DeleteBucket.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DeleteBucketRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// Parameter.
    pub param: DeleteBucketParam,
}

pub(crate) fn build(client: &Client, req: &DeleteBucketRequest) -> RequestBuilder {
    let url = format!("{}/b/{}", BASE_URL, req.bucket.escape());
    client.delete(url).query(&req.param)
}
