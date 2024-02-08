use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::object_access_controls::ObjectAccessControl;
use crate::http::Escape;

/// Request message for ListDefaultObjectAccessControls.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListDefaultObjectAccessControlsRequest {
    /// Required. Name of a bucket.
    #[serde(skip_serializing)]
    pub bucket: String,
    /// If set, only deletes the bucket if its metageneration matches this value.
    pub if_metageneration_match: Option<i64>,
    /// If set, only deletes the bucket if its metageneration does not match this
    /// value.
    pub if_metageneration_not_match: Option<i64>,
}

/// Request message for ListDefaultObjectAccessControls.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListDefaultObjectAccessControlsResponse {
    pub kind: String,
    pub items: Option<Vec<ObjectAccessControl>>,
}

pub(crate) fn build(base_url: &str, client: &Client, req: &ListDefaultObjectAccessControlsRequest) -> RequestBuilder {
    let url = format!("{}/b/{}/defaultObjectAcl", base_url, req.bucket.escape());
    client.get(url).query(&req)
}
