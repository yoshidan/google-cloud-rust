use crate::http::channels::Channel;
use crate::http::notifications::Notification;
use crate::http::object_access_controls::insert::ObjectAccessControlCreationConfig;
use crate::http::object_access_controls::Projection;
use crate::http::objects::Object;
use crate::http::{Escape, BASE_URL};
use reqwest::{Client, RequestBuilder};
use std::collections::HashMap;

/// Request message for GetNotification.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListObjectsRequest {
    #[serde(skip_serializing)]
    pub bucket: String,
    pub delimiter: Option<String>,
    pub end_offset: Option<String>,
    pub include_trailing_delimiter: Option<bool>,
    pub max_results: Option<i32>,
    pub page_token: Option<String>,
    pub prefix: Option<String>,
    pub projection: Option<Projection>,
    pub start_offset: Option<String>,
    pub versions: Option<bool>,
}

/// The result of a call to Objects.ListObjects
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListObjectsResponse {
    /// The list of prefixes of objects matching-but-not-listed up to and including
    /// the requested delimiter.
    pub prefixes: Option<Vec<String>>,
    /// The list of items.
    pub items: Option<Vec<Object>>,
    /// The continuation token, used to page through large result sets. Provide
    /// this value in a subsequent request to return the next page of results.
    pub next_page_token: Option<String>,
}

pub(crate) fn build(client: &Client, req: &ListObjectsRequest) -> RequestBuilder {
    let url = format!("{}/b/{}/o", BASE_URL, req.bucket.escape());
    client.get(url).query(&req)
}
