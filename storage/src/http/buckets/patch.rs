use crate::http::bucket_access_controls::{BucketAccessControl, PredefinedBucketAcl};
use crate::http::buckets::insert::RetentionPolicyCreationConfig;
use crate::http::buckets::{Billing, Cors, Encryption, IamConfiguration, Lifecycle, Logging, Versioning, Website};
use crate::http::object_access_controls::insert::ObjectAccessControlCreationConfig;
use crate::http::object_access_controls::{PredefinedObjectAcl, Projection};
use crate::http::{Escape, BASE_URL};

use reqwest::{Client, RequestBuilder};
use std::collections::HashMap;

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BucketPatchConfig {
    pub acl: Option<Vec<BucketAccessControl>>,
    pub default_object_acl: Option<Vec<ObjectAccessControlCreationConfig>>,
    pub lifecycle: Option<Lifecycle>,
    pub cors: Option<Vec<Cors>>,
    pub storage_class: Option<String>,
    pub default_event_based_hold: Option<bool>,
    pub labels: Option<HashMap<String, String>>,
    pub website: Option<Website>,
    pub versioning: Option<Versioning>,
    pub logging: Option<Logging>,
    pub encryption: Option<Encryption>,
    pub billing: Option<Billing>,
    pub retention_policy: Option<RetentionPolicyCreationConfig>,
    pub iam_configuration: Option<IamConfiguration>,
    pub rpo: Option<String>,
}

/// Request for PatchBucket method.
#[derive(Clone, PartialEq, Default, serde::Deserialize, serde::Serialize, Debug)]
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

pub(crate) fn build(client: &Client, req: &PatchBucketRequest) -> RequestBuilder {
    let url = format!("{}/b/{}", BASE_URL, req.bucket.escape());
    let builder = client.patch(url).query(&req);
    if let Some(body) = &req.metadata {
        builder.json(body)
    } else {
        builder
    }
}
