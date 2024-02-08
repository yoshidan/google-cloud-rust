use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::bucket_access_controls::BucketACLRole;
use crate::http::Escape;

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BucketAccessControlCreationConfig {
    /// The entity holding the permission. Can be user-emailAddress, group-groupId, group-emailAddress, allUsers, or allAuthenticatedUsers.
    pub entity: String,
    pub role: BucketACLRole,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertBucketAccessControlRequest {
    /// Name of a bucket.
    pub bucket: String,
    pub acl: BucketAccessControlCreationConfig,
}

pub fn build(base_url: &str, client: &Client, req: &InsertBucketAccessControlRequest) -> RequestBuilder {
    let url = format!("{}/b/{}/acl", base_url, req.bucket.escape());
    client.post(url).json(&req.acl)
}
