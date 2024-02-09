use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::object_access_controls::ObjectACLRole;

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertObjectAccessControlRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// Required. Name of the object.
    pub object: String,
    /// If present, selects a specific revision of this object (as opposed to the
    /// latest version, the default).
    pub generation: Option<i64>,
    /// Properties of the object access control to be inserted.
    pub acl: ObjectAccessControlCreationConfig,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ObjectAccessControlCreationConfig {
    pub entity: String,
    pub role: ObjectACLRole,
}

pub(crate) fn build(base_url: &str, client: &Client, req: &InsertObjectAccessControlRequest) -> RequestBuilder {
    let url = format!("{}/b/{}/o/{}/acl", base_url, req.bucket, req.object);
    client.post(url).json(&req.acl)
}
