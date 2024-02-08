use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::object_access_controls::insert::ObjectAccessControlCreationConfig;
use crate::http::Escape;

/// Request message for InsertDefaultObjectAccessControl.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertDefaultObjectAccessControlRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// Properties of the object access control being inserted.
    pub object_access_control: ObjectAccessControlCreationConfig,
}

pub(crate) fn build(base_url: &str, client: &Client, req: &InsertDefaultObjectAccessControlRequest) -> RequestBuilder {
    let url = format!("{}/b/{}/defaultObjectAcl", base_url, req.bucket.escape());
    client.post(url).json(&req.object_access_control)
}
