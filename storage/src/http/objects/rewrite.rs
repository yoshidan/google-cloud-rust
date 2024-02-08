use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::object_access_controls::{PredefinedObjectAcl, Projection};
use crate::http::objects::{Encryption, Object};
use crate::http::Escape;

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RewriteObjectRequest {
    /// Name of the bucket in which to store the new object. Overrides the provided
    /// object metadata's bucket value, if any.
    #[serde(skip_serializing)]
    pub destination_bucket: String,
    /// Name of the new object. Required when the object metadata is not otherwise provided.
    /// Overrides the object metadata's name value, if any. For information about how to
    /// URL encode object names to be path safe, see Encoding URI path parts.
    #[serde(skip_serializing)]
    pub destination_object: String,
    /// Name of the bucket in which to find the source object.
    #[serde(skip_serializing)]
    pub source_bucket: String,
    ///Name of the source object. For information about how to URL encode object names
    /// to be path safe, see Encoding URI path parts.
    #[serde(skip_serializing)]
    pub source_object: String,
    /// If set, only deletes the bucket if its metageneration matches this value.
    pub if_destination_metageneration_match: Option<i64>,
    /// If set, only deletes the bucket if its metageneration does not match this
    /// value.
    pub if_destination_metageneration_not_match: Option<i64>,
    /// If set, only deletes the bucket if its metageneration matches this value.
    pub if_source_metageneration_match: Option<i64>,
    /// If set, only deletes the bucket if its metageneration does not match this
    /// value.
    pub if_source_metageneration_not_match: Option<i64>,
    /// Resource name of the Cloud KMS key that will be used to encrypt the object. The Cloud KMS key must be located in same location as the object.
    /// If the parameter is not specified, the request uses the destination bucket's default encryption key,
    /// if any, or the Google-managed encryption key.
    pub destination_kms_key_name: Option<String>,
    /// Apply a predefined set of access controls to the destination object.
    /// Acceptable values are:
    /// authenticatedRead: Object owner gets OWNER access, and allAuthenticatedUsers get READER access.
    /// bucketOwnerFullControl: Object owner gets OWNER access, and project team owners get OWNER access.
    /// bucketOwnerRead: Object owner gets OWNER access, and project team owners get READER access.
    /// private: Object owner gets OWNER access.
    /// projectPrivate: Object owner gets OWNER access, and project team members get access according to their roles.
    /// publicRead: Object owner gets OWNER access, and allUsers get READER access.
    /// If iamConfiguration.uniformBucketLevelAccess.enabled is set to true,
    /// requests that include this parameter fail with a 400 Bad Request response.
    pub destination_predefined_object_acl: Option<PredefinedObjectAcl>,
    /// The maximum number of bytes that will be rewritten per rewrite request.
    /// Most callers shouldn't need to specify this parameter - it is primarily in place to
    /// support testing. If specified the value must be an integral multiple of 1 MiB (1048576).
    /// Also, this only applies to requests where the source and destination span
    /// locations and/or storage classes. Finally,
    /// this value must not change across rewrite calls else you'll get an error
    /// that the rewriteToken is invalid.
    pub max_bytes_rewritten_per_call: Option<i64>,
    /// Set of properties to return. Defaults to noAcl,
    /// unless the object resource specifies the acl property, when it defaults to full.
    /// Acceptable values are:
    /// full: Include all properties.
    /// noAcl: Omit the owner, acl property.
    pub projection: Option<Projection>,
    /// If present, selects a specific revision of the source object (as opposed to the latest version, the default).
    pub source_generation: Option<i64>,
    /// Include this field (from the previous rewrite response) on each rewrite request
    /// after the first one, until the rewrite response 'done' flag is true.
    /// Calls that provide a rewriteToken can omit all other request fields,
    /// but if included those fields must match the values provided in the first rewrite request.
    pub rewrite_token: Option<String>,
    /// Destination object metadata.
    #[serde(skip_serializing)]
    pub destination_metadata: Option<Object>,
    /// Source encryption setting
    #[serde(skip_serializing)]
    pub source_encryption: Option<Encryption>,
    /// Destination encryption setting
    #[serde(skip_serializing)]
    pub destination_encryption: Option<Encryption>,
}

/// A rewrite response.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RewriteObjectResponse {
    /// The total bytes written so far, which can be used to provide a waiting user
    /// with a progress indicator. This property is always present in the response.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub total_bytes_rewritten: i64,
    /// The total size of the object being copied in bytes. This property is always
    /// present in the response.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub object_size: i64,
    /// `true` if the copy is finished; otherwise, `false` if
    /// the copy is in progress. This property is always present in the response.
    pub done: bool,
    /// A token to use in subsequent requests to continue copying data. This token
    /// is present in the response only when there is more data to copy.
    pub rewrite_token: Option<String>,
    /// A resource containing the metadata for the copied-to object. This property
    /// is present in the response only when copying completes.
    pub resource: Option<Object>,
}

pub(crate) fn build(base_url: &str, client: &Client, req: &RewriteObjectRequest) -> RequestBuilder {
    let url = format!(
        "{}/b/{}/o/{}/rewriteTo/b/{}/o/{}",
        base_url,
        req.source_bucket.escape(),
        req.source_object.escape(),
        req.destination_bucket.escape(),
        req.destination_object.escape()
    );
    let mut builder = client.post(url).query(&req).json(&req.destination_metadata);
    if let Some(e) = &req.destination_encryption {
        builder = e.with_headers(builder)
    }
    if let Some(e) = &req.source_encryption {
        builder
            .header("X-Goog-Copy-Source-Encryption-Algorithm", &e.encryption_algorithm)
            .header("X-Goog-Copy-Source-Encryption-Key", &e.encryption_key)
            .header("X-Goog-Copy-Source-Encryption-Key-Sha256", &e.encryption_key_sha256)
    } else {
        builder
    }
}
