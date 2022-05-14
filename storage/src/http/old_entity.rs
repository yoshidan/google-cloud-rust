use crate::http::old_entity::common_enums::{PredefinedBucketAcl, PredefinedObjectAcl, Projection};
use serde::{de, Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;
use crate::http::entity::acl::{Generation, ObjectACLRole};

/// Metadata generation match parameter.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Default)]
pub struct MetadataGenerationMatch {
    /// If set, only deletes the bucket if its metageneration matches this value.
    pub if_metageneration_match: Option<i64>,
    /// If set, only deletes the bucket if its metageneration does not match this
    /// value.
    pub if_metageneration_not_match: Option<i64>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ObjectAccessControlsCreationConfig {
    pub entity: String,
    pub role: ObjectACLRole,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BucketAccessControlsCreationConfig {
    pub entity: String,
    pub role: ACLRole,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BucketCreationConfig {
    pub(crate) name: String,
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
    pub retention_policy: Option<RetentionPolicyCreationConfig>,
    pub iam_configuration: Option<bucket::IamConfiguration>,
    pub rpo: Option<String>,
}

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
    pub lifecycle: Option<bucket::Lifecycle>,
    pub cors: Option<Vec<bucket::Cors>>,
    pub storage_class: Option<String>,
    pub default_event_based_hold: Option<bool>,
    pub labels: Option<HashMap<String, String>>,
    pub website: Option<bucket::Website>,
    pub versioning: Option<bucket::Versioning>,
    pub logging: Option<bucket::Logging>,
    pub encryption: Option<bucket::Encryption>,
    pub billing: Option<bucket::Billing>,
    pub retention_policy: Option<RetentionPolicyCreationConfig>,
    pub iam_configuration: Option<bucket::IamConfiguration>,
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
    pub lifecycle: Option<bucket::Lifecycle>,
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
    #[serde(deserialize_with = "from_str")]
    pub project_number: i64,
    /// The metadata generation of this bucket.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    #[serde(deserialize_with = "from_str")]
    pub metageneration: i64,
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
    pub website: Option<bucket::Website>,
    /// The bucket's versioning configuration.
    pub versioning: Option<bucket::Versioning>,
    /// The bucket's logging configuration, which defines the destination bucket
    /// and optional name prefix for the current bucket's logs.
    pub logging: Option<bucket::Logging>,
    /// The owner of the bucket. This is always the project team's owner group.
    pub owner: Option<Owner>,
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
    pub iam_configuration: Option<bucket::IamConfiguration>,
}
/// Nested message and enum types in `Bucket`.
pub mod bucket {
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
}
/// An access-control entry.
#[derive(Clone, PartialEq, Default, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BucketAccessControl {
    pub bucket: String,
    pub domain: Option<String>,
    pub email: Option<String>,
    pub entity: String,
    pub entity_id: Option<String>,
    pub etag: String,
    pub id: Option<String>,
    pub kind: String,
    pub project_team: Option<ProjectTeam>,
    pub role: String,
    pub self_link: String,
}
/// The response to a call to BucketAccessControls.ListBucketAccessControls.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListBucketAccessControlsResponse {
    /// The list of items.
    pub items: Vec<BucketAccessControl>,
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
/// An notification channel used to watch for resource changes.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Channel {
    /// A UUID or similar unique string that identifies this channel.
    pub id: String,
    /// An opaque ID that identifies the resource being watched on this channel.
    /// Stable across different API versions.
    pub resource_id: String,
    /// A version-specific identifier for the watched resource.
    pub resource_uri: String,
    /// An arbitrary string delivered to the target address with each notification
    /// delivered over this channel. Optional.
    pub token: String,
    /// Date and time of notification channel expiration. Optional.
    pub expiration: Option<chrono::DateTime<chrono::Utc>>,
    /// The type of delivery mechanism used for this channel.
    pub r#type: String,
    /// The address where notifications are delivered for this channel.
    pub address: String,
    /// Additional parameters controlling delivery channel behavior. Optional.
    pub params: ::std::collections::HashMap<String, String>,
    /// A Boolean value to indicate whether payload is wanted. Optional.
    pub payload: bool,
}
/// The result of a call to Channels.ListChannels
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListChannelsResponse {
    /// The list of notification channels for a bucket.
    pub items: Vec<list_channels_response::Items>,
}
/// Nested message and enum types in `ListChannelsResponse`.
pub mod list_channels_response {
    #[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Items {
        /// User-specified name for a channel. Needed to unsubscribe.
        pub channel_id: String,
        /// Opaque value generated by GCS representing a bucket. Needed to
        /// unsubscribe.
        pub resource_id: String,
        /// Url used to identify where notifications are sent to.
        pub push_url: String,
        /// Email address of the subscriber.
        pub subscriber_email: String,
        /// Time when the channel was created.
        pub creation_time: Option<chrono::DateTime<chrono::Utc>>,
    }
}
/// Message used to convey content being read or written, along with its
/// checksum.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChecksummedData {
    /// The data.
    pub content: Vec<u8>,
    /// CRC32C digest of the contents.
    pub crc32c: Option<u32>,
}
/// Message used for storing full (not subrange) object checksums.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ObjectChecksums {
    /// CRC32C digest of the object data. Computed by the GCS service for
    /// all written objects, and validated by the GCS service against
    /// client-supplied values if present in an InsertObjectRequest.
    pub crc32c: Option<u32>,
    /// Hex-encoded MD5 hash of the object data (hexdigest). Whether/how this
    /// checksum is provided and validated is service-dependent.
    pub md5_hash: String,
}

/// A set of properties to return in a response.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Deserialize, serde::Serialize, Debug)]
pub enum ACLRole {
    OWNER,
    READER,
    WRITER,
}

/// A collection of enums used in multiple places throughout the API.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CommonEnums {}
/// Nested message and enum types in `CommonEnums`.
pub mod common_enums {
    /// A set of properties to return in a response.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Deserialize, serde::Serialize, Debug)]
    #[repr(i32)]
    pub enum Projection {
        /// Omit `owner`, `acl`, and `defaultObjectAcl` properties.
        NoAcl = 1,
        /// Include all properties.
        Full = 2,
    }
    /// Predefined or "canned" aliases for sets of specific bucket ACL entries.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Deserialize, serde::Serialize, Debug)]
    #[repr(i32)]
    pub enum PredefinedBucketAcl {
        /// Project team owners get `OWNER` access, and
        /// `allAuthenticatedUsers` get `READER` access.
        BucketAclAuthenticatedRead = 1,
        /// Project team owners get `OWNER` access.
        BucketAclPrivate = 2,
        /// Project team members get access according to their roles.
        BucketAclProjectPrivate = 3,
        /// Project team owners get `OWNER` access, and
        /// `allUsers` get `READER` access.
        BucketAclPublicRead = 4,
        /// Project team owners get `OWNER` access, and
        /// `allUsers` get `WRITER` access.
        BucketAclPublicReadWrite = 5,
    }
    /// Predefined or "canned" aliases for sets of specific object ACL entries.
    #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Deserialize, serde::Serialize, Debug)]
    #[repr(i32)]
    pub enum PredefinedObjectAcl {
        /// Object owner gets `OWNER` access, and
        /// `allAuthenticatedUsers` get `READER` access.
        ObjectAclAuthenticatedRead = 1,
        /// Object owner gets `OWNER` access, and project team owners get
        /// `OWNER` access.
        ObjectAclBucketOwnerFullControl = 2,
        /// Object owner gets `OWNER` access, and project team owners get
        /// `READER` access.
        ObjectAclBucketOwnerRead = 3,
        /// Object owner gets `OWNER` access.
        ObjectAclPrivate = 4,
        /// Object owner gets `OWNER` access, and project team members get
        /// access according to their roles.
        ObjectAclProjectPrivate = 5,
        /// Object owner gets `OWNER` access, and `allUsers`
        /// get `READER` access.
        ObjectAclPublicRead = 6,
    }
}
/// Specifies a requested range of bytes to download.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContentRange {
    /// The starting offset of the object data.
    pub start: i64,
    /// The ending offset of the object data.
    pub end: i64,
    /// The complete length of the object data.
    pub complete_length: i64,
}
/// Hmac Key Metadata, which includes all information other than the secret.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HmacKeyMetadata {
    /// Resource name ID of the key in the format <projectId>/<accessId>.
    pub id: String,
    /// Globally unique id for keys.
    pub access_id: String,
    /// The project ID that the hmac key is contained in.
    pub project_id: String,
    /// Email of the service account the key authenticates as.
    pub service_account_email: String,
    /// State of the key. One of ACTIVE, INACTIVE, or DELETED.
    pub state: String,
    /// The creation time of the HMAC key in RFC 3339 format.
    pub time_created: Option<chrono::DateTime<chrono::Utc>>,
    /// The last modification time of the HMAC key metadata in RFC 3339 format.
    pub updated: Option<chrono::DateTime<chrono::Utc>>,
    /// Tag updated with each key update.
    pub etag: String,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NotificationCreationConfig {
    /// The Cloud PubSub topic to which this subscription publishes. Formatted as:
    /// '//pubsub.googleapis.com/projects/{project-identifier}/topics/{my-topic}'
    pub topic: String,
    /// If present, only send notifications about listed event types. If empty,
    /// sent notifications for all event types.
    pub event_types: Option<Vec<String>>,
    /// An optional list of additional attributes to attach to each Cloud PubSub
    /// message published for this notification subscription.
    pub custom_attributes: HashMap<String, String>,
    /// If present, only apply this notification configuration to object names that
    /// begin with this prefix.
    pub object_name_prefix: Option<String>,
    /// The desired content of the Payload.
    pub payload_format: String,
}
/// A subscription to receive Google PubSub notifications.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    /// The Cloud PubSub topic to which this subscription publishes. Formatted as:
    /// '//pubsub.googleapis.com/projects/{project-identifier}/topics/{my-topic}'
    pub topic: String,
    /// If present, only send notifications about listed event types. If empty,
    /// sent notifications for all event types.
    pub event_types: Option<Vec<String>>,
    /// An optional list of additional attributes to attach to each Cloud PubSub
    /// message published for this notification subscription.
    pub custom_attributes: Option<HashMap<String, String>>,
    /// HTTP 1.1 \[<https://tools.ietf.org/html/rfc7232#section-2.3\][Entity> tag]
    /// for this subscription notification.
    pub etag: String,
    /// If present, only apply this notification configuration to object names that
    /// begin with this prefix.
    pub object_name_prefix: Option<String>,
    /// The desired content of the Payload.
    pub payload_format: String,
    /// The ID of the notification.
    pub id: String,
}
/// The result of a call to Notifications.ListNotifications
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListNotificationsResponse {
    /// The list of items.
    pub items: Vec<Notification>,
}
/// An object.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Object {
    /// Content-Encoding of the object data, matching
    /// \[<https://tools.ietf.org/html/rfc7231#section-3.1.2.2\][RFC> 7231 §3.1.2.2]
    pub content_encoding: String,
    /// Content-Disposition of the object data, matching
    /// \[<https://tools.ietf.org/html/rfc6266\][RFC> 6266].
    pub content_disposition: String,
    /// Cache-Control directive for the object data, matching
    /// \[<https://tools.ietf.org/html/rfc7234#section-5.2"\][RFC> 7234 §5.2].
    /// If omitted, and the object is accessible to all anonymous users, the
    /// default will be `public, max-age=3600`.
    pub cache_control: String,
    /// Access controls on the object.
    pub acl: Vec<ObjectAccessControl>,
    /// Content-Language of the object data, matching
    /// \[<https://tools.ietf.org/html/rfc7231#section-3.1.3.2\][RFC> 7231 §3.1.3.2].
    pub content_language: String,
    /// The version of the metadata for this object at this generation. Used for
    /// preconditions and for detecting changes in metadata. A metageneration
    /// number is only meaningful in the context of a particular generation of a
    /// particular object.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    pub metageneration: i64,
    /// The deletion time of the object. Will be returned if and only if this
    /// version of the object has been deleted.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    pub time_deleted: Option<chrono::DateTime<chrono::Utc>>,
    /// Content-Type of the object data, matching
    /// \[<https://tools.ietf.org/html/rfc7231#section-3.1.1.5\][RFC> 7231 §3.1.1.5].
    /// If an object is stored without a Content-Type, it is served as
    /// `application/octet-stream`.
    pub content_type: String,
    /// Content-Length of the object data in bytes, matching
    /// \[<https://tools.ietf.org/html/rfc7230#section-3.3.2\][RFC> 7230 §3.3.2].
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    pub size: i64,
    /// The creation time of the object.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    pub time_created: Option<chrono::DateTime<chrono::Utc>>,
    /// CRC32c checksum. For more information about using the CRC32c
    /// checksum, see
    /// \[<https://cloud.google.com/storage/docs/hashes-etags#json-api\][Hashes> and
    /// ETags: Best Practices]. This is a server determined value and should not be
    /// supplied by the user when sending an Object. The server will ignore any
    /// value provided. Users should instead use the object_checksums field on the
    /// InsertObjectRequest when uploading an object.
    pub crc32c: Option<u32>,
    /// Number of underlying components that make up this object. Components are
    /// accumulated by compose operations.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    pub component_count: i32,
    /// MD5 hash of the data; encoded using base64 as per
    /// \[<https://tools.ietf.org/html/rfc4648#section-4\][RFC> 4648 §4]. For more
    /// information about using the MD5 hash, see
    /// \[<https://cloud.google.com/storage/docs/hashes-etags#json-api\][Hashes> and
    /// ETags: Best Practices]. This is a server determined value and should not be
    /// supplied by the user when sending an Object. The server will ignore any
    /// value provided. Users should instead use the object_checksums field on the
    /// InsertObjectRequest when uploading an object.
    pub md5_hash: String,
    /// HTTP 1.1 Entity tag for the object. See
    /// \[<https://tools.ietf.org/html/rfc7232#section-2.3\][RFC> 7232 §2.3].
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    pub etag: String,
    /// The modification time of the object metadata.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    pub updated: Option<chrono::DateTime<chrono::Utc>>,
    /// Storage class of the object.
    pub storage_class: String,
    /// Cloud KMS Key used to encrypt this object, if the object is encrypted by
    /// such a key.
    pub kms_key_name: String,
    /// The time at which the object's storage class was last changed. When the
    /// object is initially created, it will be set to time_created.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    pub time_storage_class_updated: Option<chrono::DateTime<chrono::Utc>>,
    /// Whether an object is under temporary hold. While this flag is set to true,
    /// the object is protected against deletion and overwrites.  A common use case
    /// of this flag is regulatory investigations where objects need to be retained
    /// while the investigation is ongoing. Note that unlike event-based hold,
    /// temporary hold does not impact retention expiration time of an object.
    pub temporary_hold: bool,
    /// A server-determined value that specifies the earliest time that the
    /// object's retention period expires. This value is in
    /// \[<https://tools.ietf.org/html/rfc3339\][RFC> 3339] format.
    /// Note 1: This field is not provided for objects with an active event-based
    /// hold, since retention expiration is unknown until the hold is removed.
    /// Note 2: This value can be provided even when temporary hold is set (so that
    /// the user can reason about policy without having to first unset the
    /// temporary hold).
    pub retention_expiration_time: Option<chrono::DateTime<chrono::Utc>>,
    /// User-provided metadata, in key/value pairs.
    pub metadata: ::std::collections::HashMap<String, String>,
    /// Whether an object is under event-based hold. Event-based hold is a way to
    /// retain objects until an event occurs, which is signified by the
    /// hold's release (i.e. this value is set to false). After being released (set
    /// to false), such objects will be subject to bucket-level retention (if any).
    /// One sample use case of this flag is for banks to hold loan documents for at
    /// least 3 years after loan is paid in full. Here, bucket-level retention is 3
    /// years and the event is the loan being paid in full. In this example, these
    /// objects will be held intact for any number of years until the event has
    /// occurred (event-based hold on the object is released) and then 3 more years
    /// after that. That means retention duration of the objects begins from the
    /// moment event-based hold transitioned from true to false.
    pub event_based_hold: Option<bool>,
    /// The name of the object.
    /// Attempting to update this field after the object is created will result in
    /// an error.
    pub name: String,
    /// The ID of the object, including the bucket name, object name, and
    /// generation number.
    /// Attempting to update this field after the object is created will result in
    /// an error.
    pub id: String,
    /// The name of the bucket containing this object.
    /// Attempting to update this field after the object is created will result in
    /// an error.
    pub bucket: String,
    /// The content generation of this object. Used for object versioning.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    pub generation: i64,
    /// The owner of the object. This will always be the uploader of the object.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    pub owner: Option<Owner>,
    /// Metadata of customer-supplied encryption key, if the object is encrypted by
    /// such a key.
    pub customer_encryption: Option<object::CustomerEncryption>,
    /// A user-specified timestamp set on an object.
    pub custom_time: Option<chrono::DateTime<chrono::Utc>>,
}
/// Nested message and enum types in `Object`.
pub mod object {
    /// Describes the customer-specified mechanism used to store the data at rest.
    #[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct CustomerEncryption {
        /// The encryption algorithm.
        pub encryption_algorithm: String,
        /// SHA256 hash value of the encryption key.
        pub key_sha256: String,
    }
}
/// An access-control entry.
#[derive(Clone, PartialEq, Default, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ObjectAccessControl {
    pub bucket: Option<String>,
    pub domain: Option<String>,
    pub email: Option<String>,
    pub entity: String,
    pub entity_id: Option<String>,
    pub etag: String,
    pub generation: Option<i64>,
    pub id: Option<String>,
    pub kind: String,
    pub object: Option<String>,
    pub project_team: Option<ProjectTeam>,
    pub role: String,
    pub self_link: Option<String>,
}
/// The result of a call to ObjectAccessControls.ListObjectAccessControls.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListObjectAccessControlsResponse {
    /// The list of items.
    pub items: Vec<ObjectAccessControl>,
}
/// The result of a call to Objects.ListObjects
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListObjectsResponse {
    /// The list of prefixes of objects matching-but-not-listed up to and including
    /// the requested delimiter.
    pub prefixes: Vec<String>,
    /// The list of items.
    pub items: Vec<Object>,
    /// The continuation token, used to page through large result sets. Provide
    /// this value in a subsequent request to return the next page of results.
    pub next_page_token: String,
}
/// Represents the Viewers, Editors, or Owners of a given project.
#[derive(Clone, PartialEq, Default, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectTeam {
    /// The project number.
    #[serde(default)]
    pub project_number: String,
    /// The team.
    #[serde(default)]
    pub team: String,
}
/// A subscription to receive Google PubSub notifications.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ServiceAccount {
    /// The ID of the notification.
    pub email_address: String,
}
/// The owner of a specific resource.
#[derive(Clone, PartialEq, Default, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Owner {
    /// The entity, in the form `user-`*userId*.
    #[serde(default)]
    pub entity: String,
    /// The ID for the entity.
    pub entity_id: Option<String>,
}

/// Request message for InsertBucketAccessControl.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertBucketAccessControlRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// Properties of the new bucket access control being inserted.
    pub bucket_access_control: Option<BucketAccessControl>,
}

/// Request message for DeleteBucket.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DeleteBucketRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// If set, only deletes the bucket if its metageneration matches this value.
    pub if_metageneration_match: Option<i64>,
    /// If set, only deletes the bucket if its metageneration does not match this
    /// value.
    pub if_metageneration_not_match: Option<i64>,
}


/// Request message for GetBucket.
#[derive(Clone, PartialEq, Default, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetBucketRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// Makes the return of the bucket metadata conditional on whether the bucket's
    /// current metageneration matches the given value.
    pub if_metageneration_match: Option<i64>,
    /// Makes the return of the bucket metadata conditional on whether the bucket's
    /// current metageneration does not match the given value.
    pub if_metageneration_not_match: Option<i64>,
    /// Set of properties to return. Defaults to `NO_ACL`.
    pub projection: Option<Projection>,
}
/// Request message for InsertBucket.
#[derive(Clone, PartialEq, Default, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertBucketRequest {
    pub predefined_acl: Option<PredefinedBucketAcl>,
    pub predefined_default_object_acl: Option<PredefinedObjectAcl>,
    pub projection: Option<Projection>,
    pub bucket: BucketCreationConfig,
}
/// Request message for ListChannels.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListChannelsRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
}
/// Request message for ListBuckets.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListBucketsRequest {
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
    /// Makes the return of the bucket metadata conditional on whether the bucket's
    /// current metageneration matches the given value.
    pub if_metageneration_match: Option<i64>,
    /// Makes the return of the bucket metadata conditional on whether the bucket's
    /// current metageneration does not match the given value.
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
    /// Makes the return of the bucket metadata conditional on whether the bucket's
    /// current metageneration matches the given value.
    pub if_metageneration_match: Option<i64>,
    /// Makes the return of the bucket metadata conditional on whether the bucket's
    /// current metageneration does not match the given value.
    pub if_metageneration_not_match: Option<i64>,
    /// Apply a predefined set of access controls to this bucket.
    pub predefined_acl: PredefinedBucketAcl,
    /// Apply a predefined set of default object access controls to this bucket.
    pub predefined_default_object_acl: PredefinedObjectAcl,
    /// Set of properties to return. Defaults to `FULL`.
    pub projection: Projection,
    /// The Bucket metadata for updating.
    pub metadata: Option<Bucket>,
}
/// Request message for StopChannel.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StopChannelRequest {
    /// The channel to be stopped.
    pub channel: Option<Channel>,
}
/// Request message for DeleteDefaultObjectAccessControl.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteDefaultObjectAccessControlRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// Required. The entity holding the permission. Can be one of:
    /// * `user-`*userId*
    /// * `user-`*emailAddress*
    /// * `group-`*groupId*
    /// * `group-`*emailAddress*
    /// * `allUsers`
    /// * `allAuthenticatedUsers`
    pub entity: String,
}
/// Request message for GetDefaultObjectAccessControl.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetDefaultObjectAccessControlRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// Required. The entity holding the permission. Can be one of:
    /// * `user-`*userId*
    /// * `user-`*emailAddress*
    /// * `group-`*groupId*
    /// * `group-`*emailAddress*
    /// * `allUsers`
    /// * `allAuthenticatedUsers`
    pub entity: String,
}
/// Request message for InsertDefaultObjectAccessControl.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertDefaultObjectAccessControlRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// Properties of the object access control being inserted.
    pub object_access_control: ObjectAccessControlsCreationConfig,
}
/// Request message for ListDefaultObjectAccessControls.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListDefaultObjectAccessControlsRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// If present, only return default ACL listing if the bucket's current
    /// metageneration matches this value.
    pub if_metageneration_match: Option<i64>,
    /// If present, only return default ACL listing if the bucket's current
    /// metageneration does not match the given value.
    pub if_metageneration_not_match: Option<i64>,
}
/// Request message for PatchDefaultObjectAccessControl.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PatchDefaultObjectAccessControlRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// Required. The entity holding the permission. Can be one of:
    /// * `user-`*userId*
    /// * `user-`*emailAddress*
    /// * `group-`*groupId*
    /// * `group-`*emailAddress*
    /// * `allUsers`
    /// * `allAuthenticatedUsers`
    pub entity: String,
    /// The ObjectAccessControl for updating.
    pub object_access_control: Option<ObjectAccessControl>,
    /// List of fields to be updated.
    ///
    /// To specify ALL fields, equivalent to the JSON API's "update" function,
    /// specify a single field with the value `*`. Note: not recommended. If a new
    /// field is introduced at a later time, an older client updating with the `*`
    /// may accidentally reset the new field's value.
    ///
    /// Not specifying any fields is an error.
    /// Not specifying a field while setting that field to a non-default value is
    /// an error.
    pub update_mask: Option<()>, //TODO
}
/// Request message for UpdateDefaultObjectAccessControl.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDefaultObjectAccessControlRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// Required. The entity holding the permission. Can be one of:
    /// * `user-`*userId*
    /// * `user-`*emailAddress*
    /// * `group-`*groupId*
    /// * `group-`*emailAddress*
    /// * `allUsers`
    /// * `allAuthenticatedUsers`
    pub entity: String,
    /// The ObjectAccessControl for updating.
    pub object_access_control: Option<ObjectAccessControl>,
}
/// Request message for DeleteNotification.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteNotificationRequest {
    /// Required. The parent bucket of the notification.
    pub bucket: String,
    /// Required. ID of the notification to delete.
    pub notification: String,
}
/// Request message for GetNotification.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetNotificationRequest {
    /// Required. The parent bucket of the notification.
    pub bucket: String,
    /// Required. Notification ID.
    /// Required.
    pub notification: String,
}
/// Request message for InsertNotification.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertNotificationRequest {
    /// Required. The parent bucket of the notification.
    pub bucket: String,
    /// Properties of the notification to be inserted.
    pub notification: Option<Notification>,
}
/// Request message for ListNotifications.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListNotificationsRequest {
    /// Required. Name of a Google Cloud Storage bucket.
    pub bucket: String,
}
/// Request message for DeleteObjectAccessControl.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteObjectAccessControlRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// Required. The entity holding the permission. Can be one of:
    /// * `user-`*userId*
    /// * `user-`*emailAddress*
    /// * `group-`*groupId*
    /// * `group-`*emailAddress*
    /// * `allUsers`
    /// * `allAuthenticatedUsers`
    pub entity: String,
    /// Required. Name of the object.
    pub object: String,
    /// If present, selects a specific revision of this object (as opposed to the
    /// latest version, the default).
    pub generation: i64,
}
/// Request message for GetObjectAccessControl.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetObjectAccessControlRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// Required. The entity holding the permission. Can be one of:
    /// * `user-`*userId*
    /// * `user-`*emailAddress*
    /// * `group-`*groupId*
    /// * `group-`*emailAddress*
    /// * `allUsers`
    /// * `allAuthenticatedUsers`
    pub entity: String,
    /// Required. Name of the object.
    pub object: String,
    /// If present, selects a specific revision of this object (as opposed to the
    /// latest version, the default).
    pub generation: Option<Generation>,
}
/// Request message for InsertObjectAccessControl.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertObjectAccessControlRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// Required. Name of the object.
    pub object: String,
    /// If present, selects a specific revision of this object (as opposed to the
    /// latest version, the default).
    pub generation: i64,
    /// Properties of the object access control to be inserted.
    pub object_access_control: Option<ObjectAccessControl>,
}
/// Request message for ListObjectAccessControls.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListObjectAccessControlsRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// Required. Name of the object.
    pub object: String,
    /// If present, selects a specific revision of this object (as opposed to the
    /// latest version, the default).
    pub generation: i64,
}
/// Request message for PatchObjectAccessControl.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PatchObjectAccessControlRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// Required. The entity holding the permission. Can be one of:
    /// * `user-`*userId*
    /// * `user-`*emailAddress*
    /// * `group-`*groupId*
    /// * `group-`*emailAddress*
    /// * `allUsers`
    /// * `allAuthenticatedUsers`
    pub entity: String,
    /// Required. Name of the object.
    /// Required.
    pub object: String,
    /// If present, selects a specific revision of this object (as opposed to the
    /// latest version, the default).
    pub generation: i64,
    /// The ObjectAccessControl for updating.
    pub object_access_control: Option<ObjectAccessControl>,

    /// List of fields to be updated.
    ///
    /// To specify ALL fields, equivalent to the JSON API's "update" function,
    /// specify a single field with the value `*`. Note: not recommended. If a new
    /// field is introduced at a later time, an older client updating with the `*`
    /// may accidentally reset the new field's value.
    ///
    /// Not specifying any fields is an error.
    /// Not specifying a field while setting that field to a non-default value is
    /// an error.
    pub update_mask: Option<()>, //TODO
}
/// Request message for UpdateObjectAccessControl.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateObjectAccessControlRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// Required. The entity holding the permission. Can be one of:
    /// * `user-`*userId*
    /// * `user-`*emailAddress*
    /// * `group-`*groupId*
    /// * `group-`*emailAddress*
    /// * `allUsers`
    /// * `allAuthenticatedUsers`
    pub entity: String,
    /// Required. Name of the object.
    /// Required.
    pub object: String,
    /// If present, selects a specific revision of this object (as opposed to the
    /// latest version, the default).
    pub generation: i64,
    /// The ObjectAccessControl for updating.
    pub object_access_control: Option<ObjectAccessControl>,

    /// List of fields to be updated.
    ///
    /// To specify ALL fields, equivalent to the JSON API's "update" function,
    /// specify a single field with the value `*`. Note: not recommended. If a new
    /// field is introduced at a later time, an older client updating with the `*`
    /// may accidentally reset the new field's value.
    ///
    /// Not specifying any fields is an error.
    /// Not specifying a field while setting that field to a non-default value is
    /// an error.
    pub update_mask: Option<()>, //TODO
}
/// Request message for ComposeObject.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ComposeObjectRequest {
    /// Required. Name of the bucket containing the source objects. The destination object is
    /// stored in this bucket.
    pub destination_bucket: String,
    /// Required. Name of the new object.
    pub destination_object: String,
    /// Apply a predefined set of access controls to the destination object.
    pub destination_predefined_acl: PredefinedBucketAcl,
    /// Properties of the resulting object.
    pub destination: Option<Object>,
    /// The list of source objects that will be concatenated into a single object.
    pub source_objects: Vec<compose_object_request::SourceObjects>,
    /// Makes the operation conditional on whether the object's current generation
    /// matches the given value. Setting to 0 makes the operation succeed only if
    /// there are no live versions of the object.
    pub if_generation_match: Option<i64>,
    /// Makes the operation conditional on whether the object's current
    /// metageneration matches the given value.
    pub if_metageneration_match: Option<i64>,
    /// Resource name of the Cloud KMS key, of the form
    /// `projects/my-project/locations/my-location/keyRings/my-kr/cryptoKeys/my-key`,
    /// that will be used to encrypt the object. Overrides the object
    /// metadata's `kms_key_name` value, if any.
    pub kms_key_name: String,
    /// A set of parameters common to Storage API requests concerning an object.
    pub common_object_request_params: Option<CommonObjectRequestParams>,
}
/// Nested message and enum types in `ComposeObjectRequest`.
pub mod compose_object_request {
    /// Description of a source object for a composition request.
    #[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct SourceObjects {
        /// The source object's name. All source objects must reside in the same
        /// bucket.
        pub name: String,
        /// The generation of this object to use as the source.
        pub generation: i64,
        /// Conditions that must be met for this operation to execute.
        pub object_preconditions: Option<source_objects::ObjectPreconditions>,
    }
    /// Nested message and enum types in `SourceObjects`.
    pub mod source_objects {
        /// Preconditions for a source object of a composition request.
        #[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
        #[serde(rename_all = "camelCase")]
        pub struct ObjectPreconditions {
            /// Only perform the composition if the generation of the source object
            /// that would be used matches this value.  If this value and a generation
            /// are both specified, they must be the same value or the call will fail.
            pub if_generation_match: Option<i64>,
        }
    }
}
/// Request message for CopyObject.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CopyObjectRequest {
    /// Required. Name of the bucket in which to store the new object. Overrides the provided
    /// object
    /// metadata's `bucket` value, if any.
    pub destination_bucket: String,
    /// Required. Name of the new object.
    /// Required when the object metadata is not otherwise provided. Overrides the
    /// object metadata's `name` value, if any.
    pub destination_object: String,
    /// Apply a predefined set of access controls to the destination object.
    pub destination_predefined_acl: PredefinedBucketAcl,
    /// Makes the operation conditional on whether the destination object's current
    /// generation matches the given value. Setting to 0 makes the operation
    /// succeed only if there are no live versions of the object.
    pub if_generation_match: Option<i64>,
    /// Makes the operation conditional on whether the destination object's current
    /// generation does not match the given value. If no live object exists, the
    /// precondition fails. Setting to 0 makes the operation succeed only if there
    /// is a live version of the object.
    pub if_generation_not_match: Option<i64>,
    /// Makes the operation conditional on whether the destination object's current
    /// metageneration matches the given value.
    pub if_metageneration_match: Option<i64>,
    /// Makes the operation conditional on whether the destination object's current
    /// metageneration does not match the given value.
    pub if_metageneration_not_match: Option<i64>,
    /// Makes the operation conditional on whether the source object's current
    /// generation matches the given value.
    pub if_source_generation_match: Option<i64>,
    /// Makes the operation conditional on whether the source object's current
    /// generation does not match the given value.
    pub if_source_generation_not_match: Option<i64>,
    /// Makes the operation conditional on whether the source object's current
    /// metageneration matches the given value.
    pub if_source_metageneration_match: Option<i64>,
    /// Makes the operation conditional on whether the source object's current
    /// metageneration does not match the given value.
    pub if_source_metageneration_not_match: Option<i64>,
    /// Set of properties to return. Defaults to `NO_ACL`, unless the
    /// object resource specifies the `acl` property, when it defaults
    /// to `full`.
    pub projection: Projection,
    /// Required. Name of the bucket in which to find the source object.
    pub source_bucket: String,
    /// Required. Name of the source object.
    pub source_object: String,
    /// If present, selects a specific revision of the source object (as opposed to
    /// the latest version, the default).
    pub source_generation: i64,
    /// Properties of the resulting object. If not set, duplicate properties of
    /// source object.
    pub destination: Option<Object>,
    /// Resource name of the Cloud KMS key, of the form
    /// `projects/my-project/locations/my-location/keyRings/my-kr/cryptoKeys/my-key`,
    /// that will be used to encrypt the object. Overrides the object
    /// metadata's `kms_key_name` value, if any.
    pub destination_kms_key_name: String,
    /// A set of parameters common to Storage API requests concerning an object.
    pub common_object_request_params: Option<CommonObjectRequestParams>,
}
/// Message for deleting an object.
/// Either `bucket` and `object` *or* `upload_id` **must** be set (but not both).
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteObjectRequest {
    /// Required. Name of the bucket in which the object resides.
    pub bucket: String,
    /// Required. The name of the object to delete (when not using a resumable write).
    pub object: String,
    /// The resumable upload_id of the object to delete (when using a
    /// resumable write). This should be copied from the `upload_id` field of
    /// `StartResumableWriteResponse`.
    pub upload_id: String,
    /// If present, permanently deletes a specific revision of this object (as
    /// opposed to the latest version, the default).
    pub generation: i64,
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
    /// A set of parameters common to Storage API requests concerning an object.
    pub common_object_request_params: Option<CommonObjectRequestParams>,
}
/// Request message for GetObjectMedia.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetObjectMediaRequest {
    /// The name of the bucket containing the object to read.
    pub bucket: String,
    /// The name of the object to read.
    pub object: String,
    /// If present, selects a specific revision of this object (as opposed
    /// to the latest version, the default).
    pub generation: i64,
    /// The offset for the first byte to return in the read, relative to the start
    /// of the object.
    ///
    /// A negative `read_offset` value will be interpreted as the number of bytes
    /// back from the end of the object to be returned. For example, if an object's
    /// length is 15 bytes, a GetObjectMediaRequest with `read_offset` = -5 and
    /// `read_limit` = 3 would return bytes 10 through 12 of the object. Requesting
    /// a negative offset whose magnitude is larger than the size of the object
    /// will result in an error.
    pub read_offset: i64,
    /// The maximum number of `data` bytes the server is allowed to return in the
    /// sum of all `Object` messages. A `read_limit` of zero indicates that there
    /// is no limit, and a negative `read_limit` will cause an error.
    ///
    /// If the stream returns fewer bytes than allowed by the `read_limit` and no
    /// error occurred, the stream includes all data from the `read_offset` to the
    /// end of the resource.
    pub read_limit: i64,
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
    /// A set of parameters common to Storage API requests concerning an object.
    pub common_object_request_params: Option<CommonObjectRequestParams>,
}
/// Request message for GetObject.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetObjectRequest {
    /// Required. Name of the bucket in which the object resides.
    pub bucket: String,
    /// Required. Name of the object.
    pub object: String,
    /// If present, selects a specific revision of this object (as opposed to the
    /// latest version, the default).
    pub generation: i64,
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
    /// Set of properties to return. Defaults to `NO_ACL`.
    pub projection: Projection,
    /// A set of parameters common to Storage API requests concerning an object.
    pub common_object_request_params: Option<CommonObjectRequestParams>,
}
/// Response message for GetObject.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetObjectMediaResponse {
    /// A portion of the data for the object. The service **may** leave `data`
    /// empty for any given `ReadResponse`. This enables the service to inform the
    /// client that the request is still live while it is running an operation to
    /// generate more data.
    pub checksummed_data: Option<ChecksummedData>,
    /// The checksums of the complete object. The client should compute one of
    /// these checksums over the downloaded object and compare it against the value
    /// provided here.
    pub object_checksums: Option<ObjectChecksums>,
    /// If read_offset and or read_limit was specified on the
    /// GetObjectMediaRequest, ContentRange will be populated on the first
    /// GetObjectMediaResponse message of the read stream.
    pub content_range: Option<ContentRange>,
    /// Metadata of the object whose media is being returned.
    /// Only populated in the first response in the stream.
    pub metadata: Option<Object>,
}
/// Describes an attempt to insert an object, possibly over multiple requests.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertObjectSpec {
    /// Destination object, including its name and its metadata.
    pub resource: Option<Object>,
    /// Apply a predefined set of access controls to this object.
    pub predefined_acl: PredefinedBucketAcl,
    /// Makes the operation conditional on whether the object's current
    /// generation matches the given value. Setting to 0 makes the operation
    /// succeed only if there are no live versions of the object.
    pub if_generation_match: Option<i64>,
    /// Makes the operation conditional on whether the object's current
    /// generation does not match the given value. If no live object exists, the
    /// precondition fails. Setting to 0 makes the operation succeed only if
    /// there is a live version of the object.
    pub if_generation_not_match: Option<i64>,
    /// Makes the operation conditional on whether the object's current
    /// metageneration matches the given value.
    pub if_metageneration_match: Option<i64>,
    /// Makes the operation conditional on whether the object's current
    /// metageneration does not match the given value.
    pub if_metageneration_not_match: Option<i64>,
    /// Set of properties to return. Defaults to `NO_ACL`, unless the
    /// object resource specifies the `acl` property, when it defaults
    /// to `full`.
    pub projection: Projection,
}
/// Message for writing an object.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertObjectRequest {
    /// Required. The offset from the beginning of the object at which the data should be
    /// written.
    ///
    /// In the first `InsertObjectRequest` of a `InsertObject()` action, it
    /// indicates the initial offset for the `Insert()` call. The value **must** be
    /// equal to the `committed_size` that a call to `QueryWriteStatus()` would
    /// return (0 if this is the first write to the object).
    ///
    /// On subsequent calls, this value **must** be no larger than the sum of the
    /// first `write_offset` and the sizes of all `data` chunks sent previously on
    /// this stream.
    ///
    /// An incorrect value will cause an error.
    pub write_offset: i64,
    /// Checksums for the complete object. If the checksums computed by the service
    /// don't match the specifified checksums the call will fail. May only be
    /// provided in the first or last request (either with first_message, or
    /// finish_write set).
    pub object_checksums: Option<ObjectChecksums>,
    /// If `true`, this indicates that the write is complete. Sending any
    /// `InsertObjectRequest`s subsequent to one in which `finish_write` is `true`
    /// will cause an error.
    /// For a non-resumable write (where the upload_id was not set in the first
    /// message), it is an error not to set this field in the final message of the
    /// stream.
    pub finish_write: bool,
    /// A set of parameters common to Storage API requests concerning an object.
    pub common_object_request_params: Option<CommonObjectRequestParams>,

    /// The first message of each stream should set one of the following.
    pub first_message: Option<insert_object_request::FirstMessage>,
    /// A portion of the data for the object.
    pub data: Option<insert_object_request::Data>,
}
/// Nested message and enum types in `InsertObjectRequest`.
pub mod insert_object_request {
    /// The first message of each stream should set one of the following.
    #[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
    pub enum FirstMessage {
        /// For resumable uploads. This should be the `upload_id` returned from a
        /// call to `StartResumableWriteResponse`.
        UploadId(String),
        /// For non-resumable uploads. Describes the overall upload, including the
        /// destination bucket and object name, preconditions, etc.
        InsertObjectSpec(super::InsertObjectSpec),
    }
    /// A portion of the data for the object.
    #[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
    pub enum Data {
        /// The data to insert. If a crc32c checksum is provided that doesn't match
        /// the checksum computed by the service, the request will fail.
        ChecksummedData(super::ChecksummedData),
        /// A reference to an existing object. This can be used to support
        /// several use cases:
        ///   - Writing a sequence of data buffers supports the basic use case of
        ///     uploading a complete object, chunk by chunk.
        ///   - Writing a sequence of references to existing objects allows an
        ///     object to be composed from a collection of objects, which can be
        ///     used to support parallel object writes.
        ///   - Writing a single reference with a given offset and size can be used
        ///     to create an object from a slice of an existing object.
        ///   - Writing an object referencing a object slice (created as noted
        ///     above) followed by a data buffer followed by another object
        ///     slice can be used to support delta upload functionality.
        Reference(super::GetObjectMediaRequest),
    }
}
/// Request message for ListObjects.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListObjectsRequest {
    /// Required. Name of the bucket in which to look for objects.
    pub bucket: String,
    /// Returns results in a directory-like mode. `items` will contain
    /// only objects whose names, aside from the `prefix`, do not
    /// contain `delimiter`. Objects whose names, aside from the
    /// `prefix`, contain `delimiter` will have their name,
    /// truncated after the `delimiter`, returned in
    /// `prefixes`. Duplicate `prefixes` are omitted.
    pub delimiter: String,
    /// If true, objects that end in exactly one instance of `delimiter`
    /// will have their metadata included in `items` in addition to
    /// `prefixes`.
    pub include_trailing_delimiter: bool,
    /// Maximum number of `items` plus `prefixes` to return
    /// in a single page of responses. As duplicate `prefixes` are
    /// omitted, fewer total results may be returned than requested. The service
    /// will use this parameter or 1,000 items, whichever is smaller.
    pub max_results: i32,
    /// A previously-returned page token representing part of the larger set of
    /// results to view.
    pub page_token: String,
    /// Filter results to objects whose names begin with this prefix.
    pub prefix: String,
    /// Set of properties to return. Defaults to `NO_ACL`.
    pub projection: Projection,
    /// If `true`, lists all versions of an object as distinct results.
    /// The default is `false`. For more information, see
    /// [Object
    /// Versioning](<https://cloud.google.com/storage/docs/object-versioning>).
    pub versions: bool,
    /// Filter results to objects whose names are lexicographically equal to or
    /// after lexicographic_start. If lexicographic_end is also set, the objects
    /// listed have names between lexicographic_start (inclusive) and
    /// lexicographic_end (exclusive).
    pub lexicographic_start: String,
    /// Filter results to objects whose names are lexicographically before
    /// lexicographic_end. If lexicographic_start is also set, the objects listed
    /// have names between lexicographic_start (inclusive) and lexicographic_end
    /// (exclusive).
    pub lexicographic_end: String,
}
/// Request object for `QueryWriteStatus`.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QueryWriteStatusRequest {
    /// Required. The name of the resume token for the object whose write status is being
    /// requested.
    pub upload_id: String,
    /// A set of parameters common to Storage API requests concerning an object.
    pub common_object_request_params: Option<CommonObjectRequestParams>,
}
/// Response object for `QueryWriteStatus`.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QueryWriteStatusResponse {
    /// The number of bytes that have been processed for the given object.
    pub committed_size: i64,
    /// `complete` is `true` only if the client has sent a `InsertObjectRequest`
    /// with `finish_write` set to true, and the server has processed that request.
    pub complete: bool,
    /// The metadata for the uploaded object. Only set if `complete` is `true`.
    pub resource: Option<Object>,
}
/// Request message for RewriteObject.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RewriteObjectRequest {
    /// Required. Name of the bucket in which to store the new object. Overrides the provided
    /// object metadata's `bucket` value, if any.
    #[serde(skip_serializing)]
    pub destination_bucket: Option<&'a str>,
    /// Required. Name of the new object.
    /// Required when the object metadata is not otherwise provided. Overrides the
    /// object metadata's `name` value, if any.
    #[serde(skip_serializing)]
    pub destination_object: String,
    /// Resource name of the Cloud KMS key, of the form
    /// `projects/my-project/locations/my-location/keyRings/my-kr/cryptoKeys/my-key`,
    /// that will be used to encrypt the object. Overrides the object
    /// metadata's `kms_key_name` value, if any.
    #[serde(skip_serializing)]
    pub destination_kms_key_name: String,
    /// Apply a predefined set of access controls to the destination object.
    #[serde(skip_serializing)]
    pub destination_predefined_acl: PredefinedBucketAcl,
    /// Makes the operation conditional on whether the object's current generation
    /// matches the given value. Setting to 0 makes the operation succeed only if
    /// there are no live versions of the object.
    pub if_generation_match: Option<i64>,
    /// Makes the operation conditional on whether the object's current generation
    /// does not match the given value. If no live object exists, the precondition
    /// fails. Setting to 0 makes the operation succeed only if there is a live
    /// version of the object.
    pub if_generation_not_match: Option<i64>,
    /// Makes the operation conditional on whether the destination object's current
    /// metageneration matches the given value.
    pub if_metageneration_match: Option<i64>,
    /// Makes the operation conditional on whether the destination object's current
    /// metageneration does not match the given value.
    pub if_metageneration_not_match: Option<i64>,
    /// Makes the operation conditional on whether the source object's current
    /// generation matches the given value.
    pub if_source_generation_match: Option<i64>,
    /// Makes the operation conditional on whether the source object's current
    /// generation does not match the given value.
    pub if_source_generation_not_match: Option<i64>,
    /// Makes the operation conditional on whether the source object's current
    /// metageneration matches the given value.
    pub if_source_metageneration_match: Option<i64>,
    /// Makes the operation conditional on whether the source object's current
    /// metageneration does not match the given value.
    pub if_source_metageneration_not_match: Option<i64>,
    /// The maximum number of bytes that will be rewritten per rewrite request.
    /// Most callers
    /// shouldn't need to specify this parameter - it is primarily in place to
    /// support testing. If specified the value must be an integral multiple of
    /// 1 MiB (1048576). Also, this only applies to requests where the source and
    /// destination span locations and/or storage classes. Finally, this value must
    /// not change across rewrite calls else you'll get an error that the
    /// `rewriteToken` is invalid.
    pub max_bytes_rewritten_per_call: i64,
    /// Set of properties to return. Defaults to `NO_ACL`, unless the
    /// object resource specifies the `acl` property, when it defaults
    /// to `full`.
    pub projection: Projection,
    /// Include this field (from the previous rewrite response) on each rewrite
    /// request after the first one, until the rewrite response 'done' flag is
    /// true. Calls that provide a rewriteToken can omit all other request fields,
    /// but if included those fields must match the values provided in the first
    /// rewrite request.
    pub rewrite_token: String,
    /// Required. Name of the bucket in which to find the source object.
    pub source_bucket: String,
    /// Required. Name of the source object.
    pub source_object: String,
    /// If present, selects a specific revision of the source object (as opposed to
    /// the latest version, the default).
    pub source_generation: i64,
    /// Properties of the destination, post-rewrite object.
    pub object: Option<Object>,
    /// The algorithm used to encrypt the source object, if any.
    pub copy_source_encryption_algorithm: String,
    /// The encryption key used to encrypt the source object, if any.
    pub copy_source_encryption_key: String,
    /// The SHA-256 hash of the key used to encrypt the source object, if any.
    pub copy_source_encryption_key_sha256: String,
    /// A set of parameters common to Storage API requests concerning an object.
    pub common_object_request_params: Option<CommonObjectRequestParams>,
}
/// A rewrite response.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RewriteResponse {
    /// The total bytes written so far, which can be used to provide a waiting user
    /// with a progress indicator. This property is always present in the response.
    pub total_bytes_rewritten: i64,
    /// The total size of the object being copied in bytes. This property is always
    /// present in the response.
    pub object_size: i64,
    /// `true` if the copy is finished; otherwise, `false` if
    /// the copy is in progress. This property is always present in the response.
    pub done: bool,
    /// A token to use in subsequent requests to continue copying data. This token
    /// is present in the response only when there is more data to copy.
    pub rewrite_token: String,
    /// A resource containing the metadata for the copied-to object. This property
    /// is present in the response only when copying completes.
    pub resource: Option<Object>,
}
/// Request message StartResumableWrite.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartResumableWriteRequest {
    /// The destination bucket, object, and metadata, as well as any preconditions.
    pub insert_object_spec: Option<InsertObjectSpec>,
    /// A set of parameters common to Storage API requests concerning an object.
    pub common_object_request_params: Option<CommonObjectRequestParams>,
}
/// Response object for `StartResumableWrite`.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartResumableWriteResponse {
    /// The upload_id of the newly started resumable write operation. This
    /// value should be copied into the `InsertObjectRequest.upload_id` field.
    pub upload_id: String,
}
/// Request message for PatchObject.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PatchObjectRequest {
    /// Required. Name of the bucket in which the object resides.
    pub bucket: String,
    /// Required. Name of the object.
    pub object: String,
    /// If present, selects a specific revision of this object (as opposed to the
    /// latest version, the default).
    pub generation: i64,
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
    /// Apply a predefined set of access controls to this object.
    pub predefined_acl: PredefinedBucketAcl,
    /// Set of properties to return. Defaults to `FULL`.
    pub projection: Projection,
    /// The Object metadata for updating.
    pub metadata: Option<Object>,
    /// List of fields to be updated.
    ///
    /// To specify ALL fields, equivalent to the JSON API's "update" function,
    /// specify a single field with the value `*`. Note: not recommended. If a new
    /// field is introduced at a later time, an older client updating with the `*`
    /// may accidentally reset the new field's value.
    ///
    /// Not specifying any fields is an error.
    /// Not specifying a field while setting that field to a non-default value is
    /// an error.
    pub update_mask: Option<()>, //TODO
    /// A set of parameters common to Storage API requests concerning an object.
    pub common_object_request_params: Option<CommonObjectRequestParams>,
}
/// Request message for UpdateObject.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateObjectRequest {
    /// Required. Name of the bucket in which the object resides.
    pub bucket: String,
    /// Required. Name of the object.
    pub object: String,
    /// If present, selects a specific revision of this object (as opposed to the
    /// latest version, the default).
    pub generation: i64,
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
    /// Apply a predefined set of access controls to this object.
    pub predefined_acl: PredefinedBucketAcl,
    /// Set of properties to return. Defaults to `FULL`.
    pub projection: Projection,
    /// The Object metadata for updating.
    pub metadata: Option<Object>,
    /// A set of parameters common to Storage API requests concerning an object.
    pub common_object_request_params: Option<CommonObjectRequestParams>,
}
/// Request message for WatchAllObjects.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WatchAllObjectsRequest {
    /// Name of the bucket in which to look for objects.
    pub bucket: String,
    /// If `true`, lists all versions of an object as distinct results.
    /// The default is `false`. For more information, see
    /// [Object
    /// Versioning](<https://cloud.google.com/storage/docs/object-versioning>).
    pub versions: bool,
    /// Returns results in a directory-like mode. `items` will contain
    /// only objects whose names, aside from the `prefix`, do not
    /// contain `delimiter`. Objects whose names, aside from the
    /// `prefix`, contain `delimiter` will have their name,
    /// truncated after the `delimiter`, returned in
    /// `prefixes`. Duplicate `prefixes` are omitted.
    pub delimiter: String,
    /// Maximum number of `items` plus `prefixes` to return
    /// in a single page of responses. As duplicate `prefixes` are
    /// omitted, fewer total results may be returned than requested. The service
    /// will use this parameter or 1,000 items, whichever is smaller.
    pub max_results: i32,
    /// Filter results to objects whose names begin with this prefix.
    pub prefix: String,
    /// If true, objects that end in exactly one instance of `delimiter`
    /// will have their metadata included in `items` in addition to
    /// `prefixes`.
    pub include_trailing_delimiter: bool,
    /// A previously-returned page token representing part of the larger set of
    /// results to view.
    pub page_token: String,
    /// Set of properties to return. Defaults to `NO_ACL`.
    pub projection: Projection,
    /// Properties of the channel to be inserted.
    pub channel: Option<Channel>,
}
/// Request message for GetProjectServiceAccount.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetProjectServiceAccountRequest {
    /// Required. Project ID.
    pub project_id: String,
}
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateHmacKeyRequest {
    /// Required. The project that the HMAC-owning service account lives in.
    pub project_id: String,
    /// Required. The service account to create the HMAC for.
    pub service_account_email: String,
}
/// Create hmac response.  The only time the secret for an HMAC will be returned.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateHmacKeyResponse {
    /// Key metadata.
    pub metadata: Option<HmacKeyMetadata>,
    /// HMAC key secret material.
    pub secret: String,
}
/// Request object to delete a given HMAC key.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteHmacKeyRequest {
    /// Required. The identifying key for the HMAC to delete.
    pub access_id: String,
    /// Required. The project id the HMAC key lies in.
    pub project_id: String,
}
/// Request object to get metadata on a given HMAC key.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetHmacKeyRequest {
    /// Required. The identifying key for the HMAC to delete.
    pub access_id: String,
    /// Required. The project id the HMAC key lies in.
    pub project_id: String,
}
/// Request to fetch a list of HMAC keys under a given project.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListHmacKeysRequest {
    /// Required. The project id to list HMAC keys for.
    pub project_id: String,
    /// An optional filter to only return HMAC keys for one service account.
    pub service_account_email: String,
    /// An optional bool to return deleted keys that have not been wiped out yet.
    pub show_deleted_keys: bool,
    /// The maximum number of keys to return.
    pub max_results: i32,
    /// A previously returned token from ListHmacKeysResponse to get the next page.
    pub page_token: String,
}
/// Hmac key list response with next page information.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListHmacKeysResponse {
    /// The continuation token, used to page through large result sets. Provide
    /// this value in a subsequent request to return the next page of results.
    pub next_page_token: String,
    /// The list of items.
    pub items: Vec<HmacKeyMetadata>,
}
/// Request object to update an HMAC key state.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateHmacKeyRequest {
    /// Required. The id of the HMAC key.
    pub access_id: String,
    /// Required. The project id the HMAC's service account lies in.
    pub project_id: String,
    /// Required. The service account owner of the HMAC key.
    pub metadata: Option<HmacKeyMetadata>,
}
/// Parameters that can be passed to any object request.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CommonObjectRequestParams {
    /// Encryption algorithm used with Customer-Supplied Encryption Keys feature.
    pub encryption_algorithm: String,
    /// Encryption key used with Customer-Supplied Encryption Keys feature.
    pub encryption_key: String,
    /// SHA256 hash of encryption key used with Customer-Supplied Encryption Keys
    /// feature.
    pub encryption_key_sha256: String,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_empty(v: &str) -> bool {
    v.is_empty()
}

fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: de::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}

fn from_str_opt<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
    D: serde::Deserializer<'de>,
{
    let s: Result<serde_json::Value, _> = serde::Deserialize::deserialize(deserializer);
    match s {
        Ok(serde_json::Value::String(s)) => T::from_str(&s).map_err(serde::de::Error::custom).map(Option::from),
        Ok(serde_json::Value::Number(num)) => T::from_str(&num.to_string())
            .map_err(serde::de::Error::custom)
            .map(Option::from),
        Ok(_value) => Err(serde::de::Error::custom("Incorrect type")),
        Err(_) => Ok(None),
    }
}

impl From<Projection> for &'static str {
    fn from(v: Projection) -> Self {
        match v {
            Projection::NoAcl => "noAcl",
            Projection::Full => "full",
        }
    }
}

impl From<PredefinedBucketAcl> for &'static str {
    fn from(v: PredefinedBucketAcl) -> Self {
        match v {
            PredefinedBucketAcl::BucketAclAuthenticatedRead => "authenticatedRead",
            PredefinedBucketAcl::BucketAclPrivate => "private",
            PredefinedBucketAcl::BucketAclProjectPrivate => "projectPrivate",
            PredefinedBucketAcl::BucketAclPublicRead => "publicRead",
            PredefinedBucketAcl::BucketAclPublicReadWrite => "publicReadWrite",
        }
    }
}

impl From<PredefinedObjectAcl> for &'static str {
    fn from(v: PredefinedObjectAcl) -> Self {
        match v {
            PredefinedObjectAcl::ObjectAclAuthenticatedRead => "authenticatedRead",
            PredefinedObjectAcl::ObjectAclBucketOwnerFullControl => "bucketOwnerFullControl",
            PredefinedObjectAcl::ObjectAclBucketOwnerRead => "bucketOwnerRead",
            PredefinedObjectAcl::ObjectAclPrivate => "private",
            PredefinedObjectAcl::ObjectAclProjectPrivate => "projectPrivate",
            PredefinedObjectAcl::ObjectAclPublicRead => "publicRead",
        }
    }
}
