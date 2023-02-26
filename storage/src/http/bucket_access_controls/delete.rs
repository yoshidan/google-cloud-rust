use reqwest::{Client, RequestBuilder};

use crate::http::Escape;

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteBucketAccessControlRequest {
    /// Name of a bucket.
    pub bucket: String,
    /// The entity holding the permission. Can be user-emailAddress, group-groupId, group-emailAddress, allUsers, or allAuthenticatedUsers.
    pub entity: String,
}

pub fn build(base_url: &str, client: &Client, req: &DeleteBucketAccessControlRequest) -> RequestBuilder {
    let url = format!("{}/b/{}/acl/{}", base_url, req.bucket.escape(), req.entity.escape());
    client.delete(url)
}
