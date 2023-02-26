use reqwest::{Client, RequestBuilder};

use crate::http::object_access_controls::Projection;
use crate::http::objects::Encryption;
use crate::http::Escape;

/// Request message for GetObject.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetObjectRequest {
    /// Required. Name of the bucket in which the object resides.
    #[serde(skip_serializing)]
    pub bucket: String,
    /// Required. Name of the object.
    #[serde(skip_serializing)]
    pub object: String,
    /// If present, selects a specific revision of this object (as opposed to the
    /// latest version, the default).
    pub generation: Option<i64>,
    /// Makes the operation conditional on whether the object's current generation
    /// matches the given value. Setting to 0 makes the operation succeed only if
    /// there are no live versions of the object.
    pub if_generation_match: Option<i64>,
    /// Makes the operation conditional on whether the object's current generation
    /// does not match the given value. If no live object exists, the precondition
    /// fails. Setting to 0 makes the operation succeed only if there is a live
    /// version of the object.
    pub if_generation_not_match: Option<i64>,
    /// Makes the operation conditional on whether the object's current
    /// metageneration matches the given value.
    pub if_metageneration_match: Option<i64>,
    /// Makes the operation conditional on whether the object's current
    /// metageneration does not match the given value.
    pub if_metageneration_not_match: Option<i64>,
    /// Set of properties to return. Defaults to `NO_ACL`.
    pub projection: Option<Projection>,
    /// A set of parameters common to Storage API requests concerning an object.
    #[serde(skip_serializing)]
    pub encryption: Option<Encryption>,
}

pub(crate) fn build(base_url: &str, client: &Client, req: &GetObjectRequest) -> RequestBuilder {
    let url = format!("{}/b/{}/o/{}", base_url, req.bucket.escape(), req.object.escape());
    let builder = client.get(url).query(&req);
    if let Some(e) = &req.encryption {
        e.with_headers(builder)
    } else {
        builder
    }
}
