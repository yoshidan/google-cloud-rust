use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::object_access_controls::ObjectAccessControl;
use crate::http::Escape;

/// Request message for InsertDefaultObjectAccessControl.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PatchDefaultObjectAccessControlRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// Required. The entity holding the permission. Can be one of:
    /// * `user-`*userId*
    /// * `user-`*emailAddress*
    /// * `group-`*groupId*
    /// * `group-`*emailAddress*
    /// * `allUsers`
    /// * `allAuthenticatedUsers`
    pub entity: String,
    /// Properties of the object access control being inserted.
    pub object_access_control: ObjectAccessControl,
}

pub(crate) fn build(base_url: &str, client: &Client, req: &PatchDefaultObjectAccessControlRequest) -> RequestBuilder {
    let url = format!(
        "{}/b/{}/defaultObjectAcl/{}",
        base_url,
        req.bucket.escape(),
        req.entity.escape()
    );
    client.patch(url).json(&req.object_access_control)
}
