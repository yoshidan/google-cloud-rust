use std::collections::HashMap;
use crate::http::entity::acl::{BucketAccessControl, ObjectAccessControl, ObjectAccessControlsCreationConfig};
use crate::http::entity::{MaxResults, PageToken, Prefix, Project};
use crate::http::entity::common::{PredefinedBucketAcl, PredefinedObjectAcl, Projection};
use crate::http::entity::object::Owner;

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RetentionPolicyCreationConfig {
    pub retention_period: u64,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BucketPatchConfig {
    pub acl: Option<Vec<BucketAccessControl>>,
    pub default_object_acl: Option<Vec<ObjectAccessControlsCreationConfig>>,
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

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BucketCreationConfig {
    pub(crate) name: String,
    pub acl: Option<Vec<BucketAccessControl>>,
    pub default_object_acl: Option<Vec<ObjectAccessControlsCreationConfig>>,
    pub lifecycle: Option<Lifecycle>,
    pub cors: Option<Vec<Cors>>,
    pub location: String,
    pub storage_class: String,
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

/// A bucket.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Bucket {
    /// Access controls on the bucket.
    pub acl: Option<Vec<BucketAccessControl>>,
    /// Default access controls to apply to new objects when no ACL is provided.
    pub default_object_acl: Option<Vec<ObjectAccessControl>>,
    /// The bucket's lifecycle configuration. See
    /// \[<https://developers.google.com/storage/docs/lifecycle\]Lifecycle> Management]
    /// for more information.
    pub lifecycle: Option<Lifecycle>,
    /// The creation time of the bucket in
    /// \[<https://tools.ietf.org/html/rfc3339\][RFC> 3339] format.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    pub time_created: Option<chrono::DateTime<chrono::Utc>>,
    /// The ID of the bucket. For buckets, the `id` and `name` properties are the
    /// same.
    /// Attempting to update this field after the bucket is created will result in
    /// a \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    pub id: String,
    /// The name of the bucket.
    /// Attempting to update this field after the bucket is created will result in
    /// an error.
    pub name: String,
    /// The project number of the project the bucket belongs to.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    #[serde(deserialize_with = "crate::http::entity::from_str")]
    pub project_number: i64,
    /// The metadata generation of this bucket.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    #[serde(deserialize_with = "crate::http::entity::from_str")]
    pub metageneration: i64,
    /// The bucket's \[<https://www.w3.org/TR/cors/\][Cross-Origin> Resource Sharing]
    /// (CORS) configuration.
    pub cors: Option<Vec<Cors>>,
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
    /// HTTP 1.1 \[<https://tools.ietf.org/html/rfc7232#section-2.3"\]Entity> tag]
    /// for the bucket.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    pub etag: String,
    /// The modification time of the bucket.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    pub updated: Option<chrono::DateTime<chrono::Utc>>,
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
    pub default_event_based_hold: Option<bool>,
    /// User-provided labels, in key/value pairs.
    pub labels: Option<::std::collections::HashMap<String, String>>,
    /// The bucket's website configuration, controlling how the service behaves
    /// when accessing bucket contents as a web site. See the
    /// \[<https://cloud.google.com/storage/docs/static-website\][Static> Website
    /// Examples] for more information.
    pub website: Option<Website>,
    /// The bucket's versioning configuration.
    pub versioning: Option<Versioning>,
    /// The bucket's logging configuration, which defines the destination bucket
    /// and optional name prefix for the current bucket's logs.
    pub logging: Option<Logging>,
    /// The owner of the bucket. This is always the project team's owner group.
    pub owner: Option<Owner>,
    /// Encryption configuration for a bucket.
    pub encryption: Option<Encryption>,
    /// The bucket's billing configuration.
    pub billing: Option<Billing>,
    /// The bucket's retention policy. The retention policy enforces a minimum
    /// retention time for all objects contained in the bucket, based on their
    /// creation time. Any attempt to overwrite or delete objects younger than the
    /// retention period will result in a PERMISSION_DENIED error.  An unlocked
    /// retention policy can be modified or removed from the bucket via a
    /// storage.buckets.update operation. A locked retention policy cannot be
    /// removed or shortened in duration for the lifetime of the bucket.
    /// Attempting to remove or decrease period of a locked retention policy will
    /// result in a PERMISSION_DENIED error.
    pub retention_policy: Option<RetentionPolicy>,
    /// The location type of the bucket (region, dual-region, multi-region, etc).
    pub location_type: String,
    /// The recovery point objective for cross-region replication of the bucket.
    /// Applicable only for dual- and multi-region buckets.
    /// "DEFAULT" uses default replication.
    /// "ASYNC_TURBO" enables turbo replication, valid for dual-region buckets only.
    /// If rpo is not specified when the bucket is created, it defaults to "DEFAULT".
    /// For more information, see Turbo replication.
    pub rpo: Option<String>,
    /// The bucket's IAM configuration.
    pub iam_configuration: Option<IamConfiguration>,
}
    /// Billing properties of a bucket.
    #[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Billing {
        /// When set to true, Requester Pays is enabled for this bucket.
        pub requester_pays: bool,
    }
    /// Cross-Origin Response sharing (CORS) properties for a bucket.
    /// For more on GCS and CORS, see
    /// <https://cloud.google.com/storage/docs/cross-origin.>
    /// For more on CORS in general, see <https://tools.ietf.org/html/rfc6454.>
    #[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Cors {
        /// The list of Origins eligible to receive CORS response headers. See
        /// \[<https://tools.ietf.org/html/rfc6454\][RFC> 6454] for more on origins.
        /// Note: "*" is permitted in the list of origins, and means "any Origin".
        pub origin: Vec<String>,
        /// The list of HTTP methods on which to include CORS response headers,
        /// (`GET`, `OPTIONS`, `POST`, etc) Note: "*" is permitted in the list of
        /// methods, and means "any method".
        pub method: Vec<String>,
        /// The list of HTTP headers other than the
        /// \[<https://www.w3.org/TR/cors/#simple-response-header\][simple> response
        /// headers] to give permission for the user-agent to share across domains.
        pub response_header: Vec<String>,
        /// The value, in seconds, to return in the
        /// \[<https://www.w3.org/TR/cors/#access-control-max-age-response-header\][Access-Control-Max-Age>
        /// header] used in preflight responses.
        pub max_age_seconds: i32,
    }
    /// Encryption properties of a bucket.
    #[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Encryption {
        /// A Cloud KMS key that will be used to encrypt objects inserted into this
        /// bucket, if no encryption method is specified.
        pub default_kms_key_name: String,
    }
    /// Bucket restriction options currently enforced on the bucket.
    #[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct IamConfiguration {
        pub uniform_bucket_level_access: Option<iam_configuration::UniformBucketLevelAccess>,
        /// Whether IAM will enforce public access prevention.
        pub public_access_prevention: Option<iam_configuration::PublicAccessPrevention>,
    }
    /// Nested message and enum types in `IamConfiguration`.
    pub mod iam_configuration {
        #[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
        #[serde(rename_all = "camelCase")]
        pub struct UniformBucketLevelAccess {
            /// If set, access checks only use bucket-level IAM policies or above.
            pub enabled: bool,
            /// The deadline time for changing
            /// <code>iamConfiguration.uniformBucketLevelAccess.enabled</code> from
            /// true to false in \[<https://tools.ietf.org/html/rfc3339\][RFC> 3339]. After
            /// the deadline is passed the field is immutable.
            pub locked_time: Option<chrono::DateTime<chrono::Utc>>,
        }
        /// Public Access Prevention configuration values.
        #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Deserialize, serde::Serialize, Debug)]
        #[repr(i32)]
        pub enum PublicAccessPrevention {
            /// Prevents access from being granted to public members 'allUsers' and
            /// 'allAuthenticatedUsers'. Prevents attempts to grant new access to
            /// public members.
            #[serde(rename = "enforced")]
            Enforced = 1,
            /// This setting is inherited from Org Policy. Does not prevent access from
            /// being granted to public members 'allUsers' or 'allAuthenticatedUsers'.
            #[serde(rename = "inherited")]
            Inherited = 2,
        }
    }
    /// Lifecycle properties of a bucket.
    /// For more information, see <https://cloud.google.com/storage/docs/lifecycle.>
    #[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Lifecycle {
        /// A lifecycle management rule, which is made of an action to take and the
        /// condition(s) under which the action will be taken.
        pub rule: Vec<lifecycle::Rule>,
    }
    /// Nested message and enum types in `Lifecycle`.
    pub mod lifecycle {
        /// A lifecycle Rule, combining an action to take on an object and a
        /// condition which will trigger that action.
        #[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
        #[serde(rename_all = "camelCase")]
        pub struct Rule {
            /// The action to take.
            pub action: Option<rule::Action>,
            /// The condition(s) under which the action will be taken.
            pub condition: Option<rule::Condition>,
        }
        /// Nested message and enum types in `Rule`.
        pub mod rule {
            #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
            pub enum ActionType {
                /// Deletes a Bucket.
                Delete,
                /// Sets the `storage_class` of a Bucket.
                SetStorageClass,
            }
            /// An action to take on an object.
            #[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
            #[serde(rename_all = "camelCase")]
            pub struct Action {
                pub r#type: ActionType,
                pub storage_class: Option<String>,
            }
            /// A condition of an object which triggers some action.
            #[derive(Clone, PartialEq, Default, serde::Deserialize, serde::Serialize, Debug)]
            #[serde(rename_all = "camelCase")]
            pub struct Condition {
                pub age: i32,
                pub created_before: Option<chrono::DateTime<chrono::Utc>>,
                pub custom_time_before: Option<chrono::DateTime<chrono::Utc>>,
                pub days_since_custom_time: Option<i32>,
                pub days_since_noncurrent_time: Option<i32>,
                pub is_live: Option<bool>,
                pub matches_storage_class: Option<Vec<String>>,
                pub noncurrent_time_before: Option<chrono::DateTime<chrono::Utc>>,
                pub num_newer_versions: Option<i32>,
            }
        }
    }
    /// Logging-related properties of a bucket.
    #[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Logging {
        /// The destination bucket where the current bucket's logs should be placed.
        pub log_bucket: String,
        /// A prefix for log object names.
        pub log_object_prefix: String,
    }
    /// Retention policy properties of a bucket.
    #[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct RetentionPolicy {
        /// Server-determined value that indicates the time from which policy was
        /// enforced and effective. This value is in
        /// \[<https://tools.ietf.org/html/rfc3339\][RFC> 3339] format.
        pub effective_time: Option<chrono::DateTime<chrono::Utc>>,
        /// Once locked, an object retention policy cannot be modified.
        pub is_locked: Option<bool>,
        /// The duration in seconds that objects need to be retained. Retention
        /// duration must be greater than zero and less than 100 years. Note that
        /// enforcement of retention periods less than a day is not guaranteed. Such
        /// periods should only be used for testing purposes.
        #[serde(deserialize_with = "crate::http::entity::from_str")]
        pub retention_period: u64,
    }
    /// Properties of a bucket related to versioning.
    /// For more on GCS versioning, see
    /// <https://cloud.google.com/storage/docs/object-versioning.>
    #[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Versioning {
        /// While set to true, versioning is fully enabled for this bucket.
        pub enabled: bool,
    }
    /// Properties of a bucket related to accessing the contents as a static
    /// website. For more on hosting a static website via GCS, see
    /// <https://cloud.google.com/storage/docs/hosting-static-website.>
    #[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Website {
        /// If the requested object path is missing, the service will ensure the path
        /// has a trailing '/', append this suffix, and attempt to retrieve the
        /// resulting object. This allows the creation of `index.html`
        /// objects to represent directory pages.
        pub main_page_suffix: String,
        /// If the requested object path is missing, and any
        /// `mainPageSuffix` object is missing, if applicable, the service
        /// will return the named object from this bucket as the content for a
        /// \[<https://tools.ietf.org/html/rfc7231#section-6.5.4\][404> Not Found]
        /// result.
        pub not_found_page: String,
    }
    /// Configuration for a bucket's Autoclass feature.
    #[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Autoclass {
        /// Enables Autoclass.
        pub enabled: bool,
        /// Latest instant at which the `enabled` bit was flipped.
        pub toggle_time: Option<chrono::DateTime<chrono::Utc>>,
    }

/// Request message for DeleteBucket.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DeleteBucketParam {
    /// If set, only deletes the bucket if its metageneration matches this value.
    pub if_metageneration_match: Option<i64>,
    /// If set, only deletes the bucket if its metageneration does not match this
    /// value.
    pub if_metageneration_not_match: Option<i64>,
}

/// Request message for DeleteBucket.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DeleteBucketRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// Parameter
    pub param: DeleteBucketParam
}

/// Request message for GetBucket.
#[derive(Clone, PartialEq, Default, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetBucketRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// If set, only deletes the bucket if its metageneration matches this value.
    pub if_metageneration_match: Option<i64>,
    /// If set, only deletes the bucket if its metageneration does not match this
    /// value.
    pub if_metageneration_not_match: Option<i64>,
    /// Set of properties to return. Defaults to `NO_ACL`.
    pub projection: Option<Projection>,
}

/// Request message for InsertBucket.
#[derive(Clone, PartialEq, Default, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertBucketParam {
    pub predefined_acl: Option<PredefinedBucketAcl>,
    pub predefined_default_object_acl: Option<PredefinedObjectAcl>,
    pub projection: Option<Projection>,
}
/// Request message for InsertBucket.
#[derive(Clone, PartialEq, Default, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertBucketRequest {
    pub project: Project,
    pub param: InsertBucketParam,
    pub bucket: BucketCreationConfig,
}

/// Request message for ListBuckets.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListBucketsRequest {
    pub project: Project,
    /// Maximum number of buckets to return in a single response. The service will
    /// use this parameter or 1,000 items, whichever is smaller.
    pub max_results: Option<i32>,
    /// A previously-returned page token representing part of the larger set of
    /// results to view.
    pub page_token: Option<String>,
    /// Filter results to buckets whose names begin with this prefix.
    pub prefix: Option<String>,
    /// Set of properties to return. Defaults to `NO_ACL`.
    pub projection: Option<Projection>,
}
/// The result of a call to Buckets.ListBuckets
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListBucketsResponse {
    /// The list of items.
    pub items: Vec<Bucket>,
    /// The continuation token, used to page through large result sets. Provide
    /// this value in a subsequent request to return the next page of results.
    pub next_page_token: Option<String>,
}
/// Request message for LockRetentionPolicy.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LockRetentionPolicyRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// Makes the operation conditional on whether bucket's current metageneration
    /// matches the given value. Must be positive.
    pub if_metageneration_match: i64,
}
/// Request for PatchBucket method.
#[derive(Clone, PartialEq, Default, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PatchBucketRequest {
    /// Required. Name of a bucket.
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
    pub metadata: Option<BucketPatchConfig>,
}
/// Request for UpdateBucket method.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBucketRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// If set, only deletes the bucket if its metageneration matches this value.
    pub if_metageneration_match: Option<i64>,
    /// If set, only deletes the bucket if its metageneration does not match this
    /// value.
    pub if_metageneration_not_match: Option<i64>,
    /// Apply a predefined set of access controls to this bucket.
    pub predefined_acl: PredefinedBucketAcl,
    /// Apply a predefined set of default object access controls to this bucket.
    pub predefined_default_object_acl: PredefinedObjectAcl,
    /// Set of properties to return. Defaults to `FULL`.
    pub projection: Projection,
    /// The Bucket metadata for updating.
    pub metadata: Option<Bucket>
}