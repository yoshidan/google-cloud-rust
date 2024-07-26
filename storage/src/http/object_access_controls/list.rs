use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

/// Request message for GetObjectAccessControl.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
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
    ///   Required. Name of the object.
    #[serde(skip_serializing)]
    pub object: String,
    /// If present, selects a specific revision of this object (as opposed to the
    /// latest version, the default).
    pub generation: Option<i64>,
}

pub(crate) fn build(base_url: &str, client: &Client, req: &ListObjectAccessControlsRequest) -> RequestBuilder {
    let url = format!("{}/b/{}/o/{}/acl", base_url, req.bucket, req.object);
    client.get(url).query(&req)
}
