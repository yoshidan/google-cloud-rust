use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::object_access_controls::ObjectAccessControl;

/// Request message for PatchObjectAccessControl.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct PatchObjectAccessControlRequest {
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
    #[serde(skip_serializing)]
    pub entity: String,
    /// Required. Name of the object.
    /// Required.
    #[serde(skip_serializing)]
    pub object: String,
    /// If present, selects a specific revision of this object (as opposed to the
    /// latest version, the default).
    pub generation: Option<i64>,
    /// The ObjectAccessControl for updating.
    #[serde(skip_serializing)]
    pub acl: ObjectAccessControl,
}

pub(crate) fn build(base_url: &str, client: &Client, req: &PatchObjectAccessControlRequest) -> RequestBuilder {
    let url = format!("{}/b/{}/o/{}/acl/{}", base_url, req.bucket, req.object, req.entity);
    client.patch(url).query(&req).json(&req.acl)
}
