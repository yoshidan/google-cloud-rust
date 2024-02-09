use std::collections::HashMap;

use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::bucket_access_controls::{BucketAccessControl, PredefinedBucketAcl};
use crate::http::buckets::{Billing, Cors, Encryption, IamConfiguration, Lifecycle, Logging, Versioning, Website};
use crate::http::object_access_controls::insert::ObjectAccessControlCreationConfig;
use crate::http::object_access_controls::{PredefinedObjectAcl, Projection};

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BucketCreationConfig {
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
    /// The location of the bucket. Object data for objects in the bucket resides in physical storage
    /// within this region, dual-region, or multi-region. Defaults to "US".
    /// See Cloud Storage bucket locations for the authoritative list.
    pub location: String,
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

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RetentionPolicyCreationConfig {
    pub retention_period: u64,
}

/// Request message for InsertBucket.
#[derive(Clone, PartialEq, Eq, Default, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertBucketParam {
    pub project: String,
    pub predefined_acl: Option<PredefinedBucketAcl>,
    pub predefined_default_object_acl: Option<PredefinedObjectAcl>,
    pub projection: Option<Projection>,
}
/// Request message for InsertBucket.
#[derive(Clone, PartialEq, Eq, Default, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertBucketRequest {
    pub name: String,
    #[serde(skip_serializing)]
    pub param: InsertBucketParam,
    #[serde(flatten)]
    pub bucket: BucketCreationConfig,
}

pub(crate) fn build(base_url: &str, client: &Client, req: &InsertBucketRequest) -> RequestBuilder {
    let url = format!("{base_url}/b");
    client.post(url).query(&req.param).json(&req)
}
