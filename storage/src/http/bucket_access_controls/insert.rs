use crate::http::bucket_access_controls::BucketACLRole;
use crate::http::{Escape, BASE_URL};
use reqwest::{Client, RequestBuilder};

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BucketAccessControlCreationConfig {
    /// The entity holding the permission. Can be user-emailAddress, group-groupId, group-emailAddress, allUsers, or allAuthenticatedUsers.
    pub entity: String,
    pub role: BucketACLRole,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertBucketAccessControlRequest {
    /// Name of a bucket.
    pub bucket: String,
    pub acl: BucketAccessControlCreationConfig,
}

pub fn build(client: &Client, req: &InsertBucketAccessControlRequest) -> RequestBuilder {
    let url = format!("{}/b/{}/acl", BASE_URL, req.bucket.escape());
    client.post(url).json(&req.acl)
}
