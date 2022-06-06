use crate::http::object_access_controls::{PredefinedObjectAcl, Projection};

use crate::http::objects::Encryption;
use crate::http::{Escape, UPLOAD_BASE_URL};
use reqwest::header::{CONTENT_LENGTH, CONTENT_TYPE};
use reqwest::{Client, RequestBuilder};

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct UploadObjectRequest {
    /// Name of the bucket in which to store the new object.
    /// Overrides the provided object metadata's bucket value, if any.
    #[serde(skip_serializing)]
    pub bucket: String,
    /// Name of the object. Not required if the request body contains object metadata
    /// that includes a name value. Overrides the object metadata's name value, if any.
    /// For information about how to URL encode object names to be path safe, see Encoding URI path parts.
    pub name: String,
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
    /// Resource name of the Cloud KMS key that will be used to encrypt the object.
    /// If not specified, the request uses the bucket's default Cloud KMS key, if any,
    /// or a Google-managed encryption key.
    pub kms_key_name: Option<String>,
    ///Apply a predefined set of access controls to this object.
    /// Acceptable values are:
    /// authenticatedRead: Object owner gets OWNER access, and allAuthenticatedUsers get READER access.
    /// bucketOwnerFullControl: Object owner gets OWNER access, and project team owners get OWNER access.
    /// bucketOwnerRead: Object owner gets OWNER access, and project team owners get READER access.
    /// private: Object owner gets OWNER access.
    /// projectPrivate: Object owner gets OWNER access, and project team members get access according to their roles.
    /// publicRead: Object owner gets OWNER access, and allUsers get READER access.
    /// If iamConfiguration.uniformBucketLevelAccess.enabled is set to true,
    /// requests that include this parameter fail with a 400 Bad Request response.
    pub predefined_acl: Option<PredefinedObjectAcl>,
    /// Set of properties to return. Defaults to noAcl,
    /// unless the object resource specifies the acl property, when it defaults to full.
    /// Acceptable values are:
    /// full: Include all properties.
    /// noAcl: Omit the owner, acl property.
    pub projection: Option<Projection>,
    #[serde(skip_serializing)]
    pub encryption: Option<Encryption>,
}

pub(crate) fn build<T: Into<reqwest::Body>>(
    client: &Client,
    req: &UploadObjectRequest,
    content_length: Option<usize>,
    content_type: &str,
    body: T,
) -> RequestBuilder {
    let url = format!("{}/b/{}/o", UPLOAD_BASE_URL, req.bucket.escape());
    let mut builder = client
        .post(url)
        .query(&req)
        .body(body)
        .header(CONTENT_TYPE, content_type);

    if let Some(len) = content_length {
        builder = builder.header(CONTENT_LENGTH, len.to_string())
    }
    if let Some(e) = &req.encryption {
        e.with_headers(builder)
    } else {
        builder
    }
}
