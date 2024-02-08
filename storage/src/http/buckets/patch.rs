use std::collections::HashMap;

use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::bucket_access_controls::{BucketAccessControl, PredefinedBucketAcl};
use crate::http::buckets::insert::RetentionPolicyCreationConfig;
use crate::http::buckets::{Billing, Cors, Encryption, IamConfiguration, Lifecycle, Logging, Versioning, Website};
use crate::http::object_access_controls::insert::ObjectAccessControlCreationConfig;
use crate::http::object_access_controls::{PredefinedObjectAcl, Projection};
use crate::http::Escape;

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BucketPatchConfig {
    /// Access controls on the bucket, containing one or more bucketAccessControls Resources.
    /// If iamConfiguration.uniformBucketLevelAccess.enabled is set to true,
    /// this field is omitted in responses, and requests that specify
    /// this field fail with a 400 Bad Request response.
    pub acl: Option<Vec<BucketAccessControl>>,
    /// Default access controls to apply to new objects when no ACL is provided.
    /// This list defines an entity and role for one or more defaultObjectAccessControls Resources.
    /// If iamConfiguration.uniformBucketLevelAccess.enabled is set to true,
    /// this field is omitted in responses, and requests that specify this field
    /// fail with a 400 Bad Request response.
    pub default_object_acl: Option<Vec<ObjectAccessControlCreationConfig>>,
    /// The bucket's lifecycle configuration. See lifecycle management for more information.
    pub lifecycle: Option<Lifecycle>,
    /// The bucket's Cross-Origin Resource Sharing (CORS) configuration.
    pub cors: Option<Vec<Cors>>,
    /// The bucket's default storage class, used whenever no storageClass is specified
    /// for a newly-created object. If storageClass is not specified when the bucket is created,
    /// it defaults to "STANDARD". For available storage classes, see Storage classes.
    pub storage_class: Option<String>,
    /// Default access controls to apply to new objects when no ACL is provided.
    /// This list defines an entity and role for one or more defaultObjectAccessControls Resources.
    /// If iamConfiguration.uniformBucketLevelAccess.enabled is set to true,
    /// this field is omitted in responses, and requests that specify this field fail with a 400 Bad Request
    /// response.
    pub default_event_based_hold: bool,
    /// User-provided bucket labels, in key/value pairs.
    pub labels: Option<HashMap<String, String>>,
    /// The bucket's website configuration, controlling how the service behaves
    /// when accessing bucket contents as a web site. See the Static Website Examples for more information.
    pub website: Option<Website>,
    /// The bucket's versioning configuration.
    pub versioning: Option<Versioning>,
    /// The bucket's logging configuration, which defines the destination bucket
    /// and optional name prefix for the current bucket's logs.
    pub logging: Option<Logging>,
    /// Encryption configuration for a bucket.
    pub encryption: Option<Encryption>,
    /// The bucket's billing configuration.
    pub billing: Option<Billing>,
    /// The bucket's retention policy, which defines the minimum age
    /// an object in the bucket must have to be deleted or replaced.
    pub retention_policy: Option<RetentionPolicyCreationConfig>,
    /// The bucket's IAM configuration.
    pub iam_configuration: Option<IamConfiguration>,
    /// The recovery point objective for cross-region replication of the bucket.
    /// Applicable only for dual- and multi-region buckets.
    /// "DEFAULT" uses default replication. "ASYNC_TURBO" enables turbo replication,
    /// valid for dual-region buckets only. If rpo is not specified when the bucket is created,
    /// it defaults to "DEFAULT". For more information, see Turbo replication.
    pub rpo: Option<String>,
}

/// Request for PatchBucket method.
#[derive(Clone, PartialEq, Eq, Default, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PatchBucketRequest {
    /// Required. Name of a bucket.
    #[serde(skip_serializing)]
    pub bucket: String,
    /// If set, only deletes the bucket if its metageneration matches this value.
    pub if_metageneration_match: Option<i64>,
    /// If set, only deletes the bucket if its metageneration does not match this
    /// value.
    pub if_metageneration_not_match: Option<i64>,
    /// Apply a predefined set of access controls to this bucket.
    pub predefined_acl: Option<PredefinedBucketAcl>,
    /// Apply a predefined set of default object access controls to this bucket.
    pub predefined_default_object_acl: Option<PredefinedObjectAcl>,
    /// Set of properties to return. Defaults to `FULL`.
    pub projection: Option<Projection>,
    /// The Bucket metadata for updating.
    #[serde(skip_serializing)]
    pub metadata: Option<BucketPatchConfig>,
}

pub(crate) fn build(base_url: &str, client: &Client, req: &PatchBucketRequest) -> RequestBuilder {
    let url = format!("{}/b/{}", base_url, req.bucket.escape());
    let builder = client.patch(url).query(&req);
    if let Some(body) = &req.metadata {
        builder.json(body)
    } else {
        builder
    }
}
