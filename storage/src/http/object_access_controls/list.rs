use crate::http::object_access_controls::ObjectACLRole;
use crate::http::BASE_URL;
use reqwest::{Client, RequestBuilder};

/// Request message for GetObjectAccessControl.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListObjectAccessControlsRequest {
    /// Required. Name of a bucket.
    #[serde(skip_serializing)]
    pub bucket: String,
    /// Required. The entity holding the permission. Can be one of:
    /// * `user-`*userId*
    /// * `user-`*emailAddress*
    /// * `group-`*groupId*
    /// * `group-`*emailAddress*
    /// * `allUsers`
    /// * `allAuthenticatedUsers`
    /// Required. Name of the object.
    #[serde(skip_serializing)]
    pub object: String,
    /// If present, selects a specific revision of this object (as opposed to the
    /// latest version, the default).
    pub generation: Option<i64>,
}

pub(crate) fn build(client: &Client, req: &ListObjectAccessControlsRequest) -> RequestBuilder {
    let url = format!("{}/b/{}/o/{}/acl", BASE_URL, req.bucket, req.object);
    client.get(url).query(&req)
}