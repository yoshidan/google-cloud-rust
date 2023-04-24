use reqwest::{Client, RequestBuilder};

use crate::http::objects::{Encryption, Object};
use crate::http::{object_access_controls::Projection, Escape};
/// Request message for GetObject.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct CopyObjectRequest {
    /// Name of the new object. Required when the object metadata is not otherwise provided. Overrides the object metadata's name value, if any.
    pub destination_bucket: String,
    /// Name of the new object. Required when the object metadata is not otherwise provided. Overrides the object metadata's name value, if any.
    pub destination_object: String,
    // Name of the source object. For information about how to URL encode object names to be path safe, see Encoding URI path parts.
    pub source_object: String,
    /// Name of the bucket in which to find the source object.
    #[serde(skip_serializing)]
    pub source_bucket: String,

    /// Makes the operation conditional on there being a live destination object with a generation number that matches the given value. Setting ifGenerationMatch to 0 makes the operation succeed only if there is no live destination object.
    pub if_generation_match: Option<i64>,
    /// Makes the operation conditional on there being a live destination object with a generation number that does not match the given value. If no live destination object exists, the precondition fails. Setting ifGenerationNotMatch to 0 makes the operation succeed if there is a live version of the object.
    pub if_generation_not_match: Option<i64>,
    /// Makes the operation conditional on there being a live destination object with a metageneration number that matches the given value.
    pub if_metageneration_match: Option<i64>,
    /// Makes the operation conditional on there being a live destination object with a metageneration number that does not match the given value.
    pub if_metageneration_not_match: Option<i64>,
    /// Makes the operation conditional on whether the source object's generation matches the given value.
    pub if_source_generation_match: Option<i64>,
    /// Makes the operation conditional on whether the source object's generation does not match the given value.
    pub if_source_generation_not_match: Option<i64>,
    /// Makes the operation conditional on whether the source object's current metageneration matches the given value.
    pub if_source_metageneration_match: Option<i64>,
    /// Makes the operation conditional on whether the source object's current metageneration does not match the given value.
    pub if_source_metageneration_not_match: Option<i64>,
    /// Set of properties to return. Defaults to noAcl, unless the object resource specifies the acl property, when it defaults to full.
    ///
    /// Acceptable values are:
    /// full: Include all properties.
    /// noAcl: Omit the owner, acl property.
    pub projection: Option<Projection>,
    /// If present, selects a specific revision of the source object (as opposed to the latest version, the default)
    pub source_generation: Option<i64>,
    /// The Object metadata for updating.
    #[serde(skip_serializing)]
    pub metadata: Option<Object>,

    #[serde(skip_serializing)]
    pub encryption: Option<Encryption>,
}

pub(crate) fn build(base_url: &str, client: &Client, req: &CopyObjectRequest) -> RequestBuilder {
    let url = format!(
        "{}/b/{}/o/{}/copyTo/b/{}/o/{}",
        base_url,
        req.source_bucket.escape(),
        req.source_object.escape(),
        req.destination_bucket.escape(),
        req.destination_object.escape()
    );
    let builder = client.post(url).query(&req).json(&req.metadata);
    if let Some(e) = &req.encryption {
        e.with_headers(builder)
    } else {
        builder
    }
}
