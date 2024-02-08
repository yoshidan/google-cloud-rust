use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::channels::WatchableChannel;
use crate::http::object_access_controls::Projection;
use crate::http::Escape;

/// Request message for WatchAllObjects.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WatchAllObjectsRequest {
    /// Name of the bucket in which to look for objects.
    #[serde(skip_serializing)]
    pub bucket: String,
    /// If `true`, lists all versions of an object as distinct results.
    /// The default is `false`. For more information, see
    /// [Object
    /// Versioning](<https://cloud.google.com/storage/docs/object-versioning>).
    pub versions: Option<bool>,
    /// Returns results in a directory-like mode. `items` will contain
    /// only objects whose names, aside from the `prefix`, do not
    /// contain `delimiter`. Objects whose names, aside from the
    /// `prefix`, contain `delimiter` will have their name,
    /// truncated after the `delimiter`, returned in
    /// `prefixes`. Duplicate `prefixes` are omitted.
    pub delimiter: Option<String>,
    /// Maximum number of `items` plus `prefixes` to return
    /// in a single page of responses. As duplicate `prefixes` are
    /// omitted, fewer total results may be returned than requested. The service
    /// will use this parameter or 1,000 items, whichever is smaller.
    pub max_results: Option<i32>,
    /// Filter results to objects whose names begin with this prefix.
    pub prefix: Option<String>,
    /// If true, objects that end in exactly one instance of `delimiter`
    /// will have their metadata included in `items` in addition to
    /// `prefixes`.
    pub include_trailing_delimiter: Option<bool>,
    /// A previously-returned page token representing part of the larger set of
    /// results to view.
    pub page_token: Option<String>,
    /// Set of properties to return. Defaults to `NO_ACL`.
    pub projection: Option<Projection>,
    /// Properties of the channel to be inserted.
    #[serde(skip_serializing)]
    pub channel: Option<WatchableChannel>,
}

#[allow(dead_code)]
pub(crate) fn build(base_url: &str, client: &Client, req: &WatchAllObjectsRequest) -> RequestBuilder {
    let url = format!("{}/b/{}/o/watch", base_url, req.bucket.escape());
    client.post(url).query(&req).json(&req.channel)
}
