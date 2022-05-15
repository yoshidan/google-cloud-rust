use reqwest::{Client, RequestBuilder};
use crate::http::{BASE_URL, Escape};
use crate::http::channels::Channel;
use crate::http::object_access_controls::insert::ObjectAccessControlsCreationConfig;

/// Request message for InsertDefaultObjectAccessControl.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertDefaultObjectAccessControlRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// Properties of the object access control being inserted.
    pub object_access_control: ObjectAccessControlsCreationConfig,
}

pub(crate) fn build(client: &Client, req: &InsertDefaultObjectAccessControlRequest) -> RequestBuilder {
    let url = format!("{}/b/{}/defaultObjectAcl", BASE_URL, req.bucket.escape());
    client.post(url).json(&req.object_access_control)
}