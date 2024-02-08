use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::Escape;

/// Request message for GetDefaultObjectAccessControl.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
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

pub(crate) fn build(base_url: &str, client: &Client, req: &GetDefaultObjectAccessControlRequest) -> RequestBuilder {
    let url = format!(
        "{}/b/{}/defaultObjectAcl/{}",
        base_url,
        req.bucket.escape(),
        req.entity.escape()
    );
    client.get(url)
}
