use crate::http::bucket_access_controls::{BucketAccessControl, PredefinedBucketAcl};
use crate::http::buckets::{Billing, Cors, Encryption, IamConfiguration, Lifecycle, Logging, Versioning, Website};
use crate::http::object_access_controls::insert::ObjectAccessControlCreationConfig;
use crate::http::object_access_controls::{PredefinedObjectAcl, Projection};
use crate::http::BASE_URL;

use reqwest::{Client, RequestBuilder};
use std::collections::HashMap;

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BucketCreationConfig {
    pub acl: Option<Vec<BucketAccessControl>>,
    pub default_object_acl: Option<Vec<ObjectAccessControlCreationConfig>>,
    pub lifecycle: Option<Lifecycle>,
    pub cors: Option<Vec<Cors>>,
    pub location: String,
    pub storage_class: Option<String>,
    pub default_event_based_hold: bool,
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

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RetentionPolicyCreationConfig {
    pub retention_period: u64,
}

/// Request message for InsertBucket.
#[derive(Clone, PartialEq, Default, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertBucketParam {
    pub project: String,
    pub predefined_acl: Option<PredefinedBucketAcl>,
    pub predefined_default_object_acl: Option<PredefinedObjectAcl>,
    pub projection: Option<Projection>,
}
/// Request message for InsertBucket.
#[derive(Clone, PartialEq, Default, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertBucketRequest {
    pub name: String,
    #[serde(skip_serializing)]
    pub param: InsertBucketParam,
    #[serde(flatten)]
    pub bucket: BucketCreationConfig,
}

pub(crate) fn build(client: &Client, req: &InsertBucketRequest) -> RequestBuilder {
    let url = format!("{}/b", BASE_URL);
    client.post(url).query(&req.param).json(&req)
}
