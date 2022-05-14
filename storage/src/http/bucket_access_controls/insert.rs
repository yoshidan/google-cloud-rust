use reqwest::{Client, RequestBuilder};
use crate::http::{BASE_URL, Error, Escape};
use crate::http::bucket_access_controls::BucketACLRole;

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BucketAccessControlsCreationConfig {
    /// The entity holding the permission. Can be user-emailAddress, group-groupId, group-emailAddress, allUsers, or allAuthenticatedUsers.
    pub entity: String,
    pub role: BucketACLRole,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertBucketAccessControlsRequest {
    /// Name of a bucket.
    pub bucket: String,
    pub acl: BucketAccessControlsCreationConfig,
}

pub fn build(client: &Client, req: &InsertBucketAccessControlsRequest) -> RequestBuilder {
    let url = format!("{}/b/{}/acl", BASE_URL, req.bucket.escape());
    client.post(url).json(&req.acl)
}