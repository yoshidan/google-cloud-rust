use reqwest::{Client, RequestBuilder};
use crate::http::{BASE_URL, Error, Escape};
use crate::http::bucket_access_controls::BucketAccessControl;

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListBucketAccessControlsRequest {
    /// Name of a bucket.
    pub bucket: String,
}

/// The response to a call to BucketAccessControls.ListBucketAccessControls.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListBucketAccessControlsResponse {
    /// The list of items.
    pub items: Vec<BucketAccessControl>,
}

pub fn build(client: &Client, req: &ListBucketAccessControlsRequest) -> RequestBuilder{
    let url = format!("{}/b/{}/acl", BASE_URL, req.bucket.escape());
    client.get(url)
}