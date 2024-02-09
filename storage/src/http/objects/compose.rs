use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::object_access_controls::PredefinedObjectAcl;
use crate::http::objects::{Encryption, Object, SourceObjects};
use crate::http::Escape;

/// Request message for ComposeObject.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ComposeObjectRequest {
    /// Required. Name of the bucket containing the source objects. The destination object is
    /// stored in this bucket.
    #[serde(skip_serializing)]
    pub bucket: String,
    /// Required. Name of the new object.
    #[serde(skip_serializing)]
    pub destination_object: String,
    /// Apply a predefined set of access controls to the destination object.
    pub destination_predefined_acl: Option<PredefinedObjectAcl>,
    #[serde(skip_serializing)]
    pub composing_targets: ComposingTargets,
    /// Makes the operation conditional on whether the object's current generation
    /// matches the given value. Setting to 0 makes the operation succeed only if
    /// there are no live versions of the object.
    pub if_generation_match: Option<i64>,
    /// Makes the operation conditional on whether the object's current
    /// metageneration matches the given value.
    pub if_metageneration_match: Option<i64>,
    /// Resource name of the Cloud KMS key, of the form
    /// `projects/my-project/locations/my-location/keyRings/my-kr/cryptoKeys/my-key`,
    /// that will be used to encrypt the object. Overrides the object
    /// metadata's `kms_key_name` value, if any.
    pub kms_key_name: Option<String>,
    /// A set of parameters common to Storage API requests concerning an object.
    #[serde(skip_serializing)]
    pub encryption: Option<Encryption>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ComposingTargets {
    /// Properties of the resulting object.
    pub destination: Option<Object>,
    /// The list of source objects that will be concatenated into a single object.
    pub source_objects: Vec<SourceObjects>,
}

pub(crate) fn build(base_url: &str, client: &Client, req: &ComposeObjectRequest) -> RequestBuilder {
    let url = format!(
        "{}/b/{}/o/{}/compose",
        base_url,
        req.bucket.escape(),
        req.destination_object.escape()
    );
    let builder = client.post(url).query(&req).json(&req.composing_targets);
    if let Some(e) = &req.encryption {
        e.with_headers(builder)
    } else {
        builder
    }
}
