use crate::apiv1::entity::common_enums::{PredefinedBucketAcl, PredefinedObjectAcl};
use crate::apiv1::entity::{bucket, Bucket, BucketAccessControl, ObjectAccessControl};
use std::collections::HashMap;

#[derive(Clone)]
pub struct BucketCreationConfig {
    pub predefined_acl: PredefinedBucketAcl,
    /// Apply a predefined set of default object access controls to this bucket.
    pub predefined_default_object_acl: PredefinedObjectAcl,
    /// Access controls on the bucket.
    pub acl: Option<Vec<BucketAccessControl>>,
    /// Default access controls to apply to new objects when no ACL is provided.
    pub default_object_acl: Option<Vec<ObjectAccessControl>>,
    /// The bucket's lifecycle configuration. See
    /// \[<https://developers.google.com/storage/docs/lifecycle\]Lifecycle> Management]
    /// for more information.
    pub lifecycle: Option<bucket::Lifecycle>,
    /// The bucket's \[<https://www.w3.org/TR/cors/\][Cross-Origin> Resource Sharing]
    /// (CORS) configuration.
    pub cors: Option<Vec<bucket::Cors>>,
    /// The location of the bucket. Object data for objects in the bucket resides
    /// in physical storage within this region.  Defaults to `US`. See the
    /// \[<https://developers.google.com/storage/docs/concepts-techniques#specifyinglocations"\][developer's>
    /// guide] for the authoritative list. Attempting to update this field after
    /// the bucket is created will result in an error.
    pub location: String,
    /// The bucket's default storage class, used whenever no storageClass is
    /// specified for a newly-created object. This defines how objects in the
    /// bucket are stored and determines the SLA and the cost of storage.
    /// If this value is not specified when the bucket is created, it will default
    /// to `STANDARD`. For more information, see
    /// <https://developers.google.com/storage/docs/storage-classes.>
    pub storage_class: String,
    /// The default value for event-based hold on newly created objects in this
    /// bucket.  Event-based hold is a way to retain objects indefinitely until an
    /// event occurs, signified by the
    /// hold's release. After being released, such objects will be subject to
    /// bucket-level retention (if any).  One sample use case of this flag is for
    /// banks to hold loan documents for at least 3 years after loan is paid in
    /// full. Here, bucket-level retention is 3 years and the event is loan being
    /// paid in full. In this example, these objects will be held intact for any
    /// number of years until the event has occurred (event-based hold on the
    /// object is released) and then 3 more years after that. That means retention
    /// duration of the objects begins from the moment event-based hold
    /// transitioned from true to false.  Objects under event-based hold cannot be
    /// deleted, overwritten or archived until the hold is removed.
    pub default_event_based_hold: bool,
    /// User-provided labels, in key/value pairs.
    pub labels: Option<HashMap<String, String>>,
    /// The bucket's website configuration, controlling how the service behaves
    /// when accessing bucket contents as a web site. See the
    /// \[<https://cloud.google.com/storage/docs/static-website\][Static> Website
    /// Examples] for more information.
    pub website: Option<bucket::Website>,
    /// The bucket's versioning configuration.
    pub versioning: Option<bucket::Versioning>,
    /// The bucket's logging configuration, which defines the destination bucket
    /// and optional name prefix for the current bucket's logs.
    pub logging: Option<bucket::Logging>,
    /// Encryption configuration for a bucket.
    pub encryption: Option<bucket::Encryption>,
    /// The bucket's billing configuration.
    pub billing: Option<bucket::Billing>,
    /// The bucket's retention policy. The retention policy enforces a minimum
    /// retention time for all objects contained in the bucket, based on their
    /// creation time. Any attempt to overwrite or delete objects younger than the
    /// retention period will result in a PERMISSION_DENIED error.  An unlocked
    /// retention policy can be modified or removed from the bucket via a
    /// storage.buckets.update operation. A locked retention policy cannot be
    /// removed or shortened in duration for the lifetime of the bucket.
    /// Attempting to remove or decrease period of a locked retention policy will
    /// result in a PERMISSION_DENIED error.
    pub retention_policy: Option<bucket::RetentionPolicy>,
    /// The bucket's IAM configuration.
    pub iam_configuration: Option<bucket::IamConfiguration>,
}

impl Default for BucketCreationConfig {
    fn default() -> Self {
        Self {
            predefined_acl: PredefinedBucketAcl::Unspecified,
            predefined_default_object_acl: PredefinedObjectAcl::Unspecified,
            acl: None,
            default_object_acl: None,
            lifecycle: None,
            cors: None,
            location: "".to_string(),
            storage_class: "".to_string(),
            default_event_based_hold: false,
            labels: Default::default(),
            website: None,
            versioning: None,
            logging: None,
            encryption: None,
            billing: None,
            retention_policy: None,
            iam_configuration: None,
        }
    }
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
            ..Default::default()
        }
    }
}
