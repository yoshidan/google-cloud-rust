use crate::http::entity::common_enums::{PredefinedBucketAcl, PredefinedObjectAcl};
use crate::http::entity::{bucket, Bucket, BucketAccessControl, ObjectAccessControl};
use std::collections::HashMap;

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ObjectAccessControlsCreationConfig {
    pub entity: String,
    pub role: String,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BucketCreationConfig {
    pub name: String,
    pub predefined_acl: Option<PredefinedBucketAcl>,
    pub predefined_default_object_acl: Option<PredefinedObjectAcl>,
    pub acl: Option<Vec<BucketAccessControl>>,
    pub default_object_acl: Option<Vec<ObjectAccessControlsCreationConfig>>,
    pub lifecycle: Option<bucket::Lifecycle>,
    pub cors: Option<Vec<bucket::Cors>>,
    pub location: String,
    pub storage_class: String,
    pub default_event_based_hold: bool,
    pub labels: Option<HashMap<String, String>>,
    pub website: Option<bucket::Website>,
    pub versioning: Option<bucket::Versioning>,
    pub logging: Option<bucket::Logging>,
    pub encryption: Option<bucket::Encryption>,
    pub billing: Option<bucket::Billing>,
    pub retention_policy: Option<bucket::RetentionPolicy>,
    pub iam_configuration: Option<bucket::IamConfiguration>,
    pub rpo: Option<String>,
}
