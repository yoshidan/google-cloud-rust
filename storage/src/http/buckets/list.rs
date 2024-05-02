use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::buckets::Bucket;
use crate::http::object_access_controls::Projection;

/// Request message for DeleteBucket.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListBucketsRequest {
    /// Required. A valid API project identifier.
    pub project: String,
    /// Maximum number of buckets to return in a single response. The service will
    /// use this parameter or 1,000 items, whichever is smaller.
    pub max_results: Option<i32>,
    /// A previously-returned page token representing part of the larger set of
    /// results to view.
    pub page_token: Option<String>,
    /// Filter results to buckets whose names begin with this prefix.
    pub prefix: Option<String>,
    /// Set of properties to return. Defaults to `NO_ACL`.
    pub projection: Option<Projection>,
    /// A glob pattern used to filter results (for example, foo*bar).
    pub match_glob: Option<String>,
}

/// The result of a call to Buckets.ListBuckets
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListBucketsResponse {
    /// The list of items.
    pub items: Vec<Bucket>,
    /// The continuation token, used to page through large result sets. Provide
    /// this value in a subsequent request to return the next page of results.
    pub next_page_token: Option<String>,
}

pub(crate) fn build(base_url: &str, client: &Client, req: &ListBucketsRequest) -> RequestBuilder {
    let url = format!("{base_url}/b");
    client.get(url).query(&req)
}
