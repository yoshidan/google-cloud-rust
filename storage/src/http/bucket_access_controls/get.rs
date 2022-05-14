use reqwest::{Client, RequestBuilder};
use crate::http::{BASE_URL, Error, Escape};

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetBucketAccessControlsRequest {
    /// Name of a bucket.
    pub bucket: String,
    /// The entity holding the permission. Can be user-emailAddress, group-groupId, group-emailAddress, allUsers, or allAuthenticatedUsers.
    pub entity: String,
}

pub fn build(client: &Client, req: &GetBucketAccessControlsRequest) -> RequestBuilder{
    let url = format!("{}/b/{}/acl/{}", BASE_URL, req.bucket.escape(), req.entity.escape());
    client.get(url)
}