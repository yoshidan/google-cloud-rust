use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::object_access_controls::Projection;
use crate::http::objects::Object;
use crate::http::Escape;

/// Request message for GetNotification.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListObjectsRequest {
    /// Name of the bucket in which to look for objects.
    #[serde(skip_serializing)]
    pub bucket: String,
    /// Returns results in a directory-like mode, with / being a common value for the delimiter.
    /// items[] contains object metadata for objects whose names do not contain
    /// delimiter, or whose names only have instances of delimiter in their prefix.
    /// prefixes[] contains truncated object names for objects whose names contain
    /// delimiter after any prefix. Object names are truncated beyond the first applicable
    /// instance of the delimiter. If multiple objects have the same truncated name, duplicates are omitted.
    pub delimiter: Option<String>,
    /// Filter results to objects whose names are lexicographically before endOffset.
    /// If startOffset is also set, the objects listed have names between startOffset
    /// (inclusive) and endOffset (exclusive).
    pub end_offset: Option<String>,
    /// If true, objects that end in exactly one instance of delimiter have their metadata
    /// included in items[] in addition to the relevant part of the object name appearing in prefixes[].
    pub include_trailing_delimiter: Option<bool>,
    /// Maximum combined number of entries in items[] and prefixes[] to return in a
    /// single page of responses. The service may return fewer results than maxResults
    /// so the presence of nextPageToken should always be checked.
    /// The recommended upper value for maxResults is 1000 objects in a single response.
    pub max_results: Option<i32>,
    /// A previously-returned page token representing part of the larger set of results to view.
    /// The pageToken is an encoded field that marks the name and generation of
    /// the last object in the returned list. In a subsequent request using the pageToken,
    /// items that come after the pageToken are shown (up to maxResults).
    /// If you start a listing and then create an object in the bucket before using a pageToken
    /// to continue listing, you do not see the new object in subsequent listing results
    /// if it is in part of the object namespace already listed.
    pub page_token: Option<String>,
    /// Filter results to include only objects whose names begin with this prefix.
    pub prefix: Option<String>,
    /// Set of properties to return. Defaults to noAcl.
    /// Acceptable values are:
    /// full: Include all properties.
    /// noAcl: Omit the owner, acl property.
    pub projection: Option<Projection>,
    /// Filter results to objects whose names are lexicographically equal to or after startOffset.
    /// If endOffset is also set, the objects listed have names between startOffset
    /// (inclusive) and endOffset (exclusive).
    pub start_offset: Option<String>,
    /// If true, lists all versions of an object as distinct results in order of
    /// increasing generation number. The default value for versions is false.
    /// For more information, see Object Versioning.
    pub versions: Option<bool>,
    /// Filter results to objects and prefixes that match this glob pattern.
    /// For more information, see [List objects and prefixes using glob](<https://cloud.google.com/storage/docs/json_api/v1/objects/list#list-objects-and-prefixes-using-glob>)
    pub match_glob: Option<String>,
}

/// The result of a call to Objects.ListObjects
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
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

pub(crate) fn build(base_url: &str, client: &Client, req: &ListObjectsRequest) -> RequestBuilder {
    let url = format!("{}/b/{}/o", base_url, req.bucket.escape());
    client.get(url).query(&req)
}
