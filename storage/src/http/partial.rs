use crate::http::entity::common_enums::{PredefinedBucketAcl, PredefinedObjectAcl};
use crate::http::entity::{bucket, Bucket, BucketAccessControl, ObjectAccessControl};
use std::collections::HashMap;

#[derive(Clone, Default)]
pub struct BucketCreationConfig {
    pub predefined_acl: Option<PredefinedBucketAcl>,
    pub predefined_default_object_acl: Option<PredefinedObjectAcl>,
    pub acl: Option<Vec<BucketAccessControl>>,
    pub default_object_acl: Option<Vec<ObjectAccessControl>>,
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

impl From<&BucketCreationConfig> for Bucket {
    fn from(attr: &BucketCreationConfig) -> Self {
        let attr = attr.clone();
        Self {
            acl: attr.acl,
            default_object_acl: attr.default_object_acl,
            lifecycle: attr.lifecycle,
            cors: attr.cors,
            location: attr.location,
            storage_class: attr.storage_class,
            default_event_based_hold: attr.default_event_based_hold,
            labels: attr.labels,
            website: attr.website,
            versioning: attr.versioning,
            logging: attr.logging,
            encryption: attr.encryption,
            billing: attr.billing,
            retention_policy: attr.retention_policy,
            iam_configuration: attr.iam_configuration,
            rpo: attr.rpo,
            ..Default::default()
        }
    }
}
