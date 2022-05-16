use crate::http::object_access_controls::Projection;
use std::collections::HashMap;

pub mod stop;

/// An notification channel used to watch for resource changes.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WatchableChannel {
    /// A UUID or similar unique string that identifies this channel.
    pub id: String,
    /// An opaque ID that identifies the resource being watched on this channel.
    /// Stable across different API versions.
    pub resource_id: String,
    /// A version-specific identifier for the watched resource.
    pub resource_uri: String,
    /// An arbitrary string delivered to the target address with each notification
    /// delivered over this channel. Optional.
    pub token: String,
    /// Date and time of notification channel expiration. Optional.
    pub expiration: Option<chrono::DateTime<chrono::Utc>>,
    /// The type of delivery mechanism used for this channel.
    pub r#type: String,
    /// The address where notifications are delivered for this channel.
    pub address: String,
    /// Additional parameters controlling delivery channel behavior. Optional.
    pub params: HashMap<String, String>,
    /// A Boolean value to indicate whether payload is wanted. Optional.
    pub payload: bool,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Channel {
    /// User-specified name for a channel. Needed to unsubscribe.
    pub channel_id: String,
    /// Opaque value generated by GCS representing a bucket. Needed to
    /// unsubscribe.
    pub resource_id: String,
    /// Url used to identify where notifications are sent to.
    pub push_url: String,
    /// Email address of the subscriber.
    pub subscriber_email: String,
    /// Time when the channel was created.
    pub creation_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// Request message for WatchAllObjects.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WatchAllObjectsRequest {
    /// Name of the bucket in which to look for objects.
    pub bucket: String,
    /// If `true`, lists all versions of an object as distinct results.
    /// The default is `false`. For more information, see
    /// [Object
    /// Versioning](<https://cloud.google.com/storage/docs/object-versioning>).
    pub versions: bool,
    /// Returns results in a directory-like mode. `items` will contain
    /// only objects whose names, aside from the `prefix`, do not
    /// contain `delimiter`. Objects whose names, aside from the
    /// `prefix`, contain `delimiter` will have their name,
    /// truncated after the `delimiter`, returned in
    /// `prefixes`. Duplicate `prefixes` are omitted.
    pub delimiter: String,
    /// Maximum number of `items` plus `prefixes` to return
    /// in a single page of responses. As duplicate `prefixes` are
    /// omitted, fewer total results may be returned than requested. The service
    /// will use this parameter or 1,000 items, whichever is smaller.
    pub max_results: i32,
    /// Filter results to objects whose names begin with this prefix.
    pub prefix: String,
    /// If true, objects that end in exactly one instance of `delimiter`
    /// will have their metadata included in `items` in addition to
    /// `prefixes`.
    pub include_trailing_delimiter: bool,
    /// A previously-returned page token representing part of the larger set of
    /// results to view.
    pub page_token: String,
    /// Set of properties to return. Defaults to `NO_ACL`.
    pub projection: Projection,
    /// Properties of the channel to be inserted.
    pub channel: WatchableChannel,
}
