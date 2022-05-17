
use crate::http::{Escape, BASE_URL};
use reqwest::{Client, RequestBuilder};

/// Request message for GetDefaultObjectAccessControl.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetDefaultObjectAccessControlRequest {
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
}

pub(crate) fn build(client: &Client, req: &GetDefaultObjectAccessControlRequest) -> RequestBuilder {
    let url = format!(
        "{}/b/{}/defaultObjectAcl/{}",
        BASE_URL,
        req.bucket.escape(),
        req.entity.escape()
    );
    client.get(url)
}
