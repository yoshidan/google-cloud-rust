use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::bucket_access_controls::BucketAccessControl;
use crate::http::Escape;

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListBucketAccessControlsRequest {
    /// Name of a bucket.
    pub bucket: String,
}

/// The response to a call to BucketAccessControls.ListBucketAccessControls.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListBucketAccessControlsResponse {
    /// The list of items.
    pub items: Vec<BucketAccessControl>,
}

pub fn build(base_url: &str, client: &Client, req: &ListBucketAccessControlsRequest) -> RequestBuilder {
    let url = format!("{}/b/{}/acl", base_url, req.bucket.escape());
    client.get(url)
}
