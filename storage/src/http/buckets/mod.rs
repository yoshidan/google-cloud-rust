use time::OffsetDateTime;

use crate::http::bucket_access_controls::BucketAccessControl;
use crate::http::object_access_controls::ObjectAccessControl;
use crate::http::objects::Owner;

pub mod delete;
pub mod get;
pub mod get_iam_policy;
pub mod insert;
pub mod list;
pub mod list_channels;
pub mod lock_retention_policy;
pub mod patch;
pub mod set_iam_policy;
pub mod test_iam_permissions;

/// A bucket.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Default, Debug)]
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
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub time_created: Option<OffsetDateTime>,
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
    #[serde(deserialize_with = "crate::http::from_str")]
    pub project_number: i64,
    /// The metadata generation of this bucket.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    #[serde(deserialize_with = "crate::http::from_str")]
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
    /// <https://developers.google.com/storage/docs/storage-classes>.
    pub storage_class: String,
    /// HTTP 1.1 \[<https://tools.ietf.org/html/rfc7232#section-2.3"\]Entity> tag]
    /// for the bucket.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    pub etag: String,
    /// The modification time of the bucket.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub updated: Option<OffsetDateTime>,
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
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Billing {
    /// When set to true, Requester Pays is enabled for this bucket.
    pub requester_pays: bool,
}
/// Cross-Origin Response sharing (CORS) properties for a bucket.
/// For more on GCS and CORS, see
/// <https://cloud.google.com/storage/docs/cross-origin>.
/// For more on CORS in general, see <https://tools.ietf.org/html/rfc6454>.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
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
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Encryption {
    /// A Cloud KMS key that will be used to encrypt objects inserted into this
    /// bucket, if no encryption method is specified.
    pub default_kms_key_name: String,
}
/// Bucket restriction options currently enforced on the bucket.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IamConfiguration {
    pub uniform_bucket_level_access: Option<iam_configuration::UniformBucketLevelAccess>,
    /// Whether IAM will enforce public access prevention.
    pub public_access_prevention: Option<iam_configuration::PublicAccessPrevention>,
}
/// Nested message and enum types in `IamConfiguration`.
pub mod iam_configuration {
    use time::OffsetDateTime;

    #[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct UniformBucketLevelAccess {
        /// If set, access checks only use bucket-level IAM policies or above.
        pub enabled: bool,
        /// The deadline time for changing
        /// <code>iamConfiguration.uniformBucketLevelAccess.enabled</code> from
        /// true to false in \[<https://tools.ietf.org/html/rfc3339\][RFC> 3339]. After
        /// the deadline is passed the field is immutable.
        #[serde(default, with = "time::serde::rfc3339::option")]
        pub locked_time: Option<OffsetDateTime>,
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
/// For more information, see <https://cloud.google.com/storage/docs/lifecycle>.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
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
    #[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Rule {
        /// The action to take.
        pub action: Option<rule::Action>,
        /// The condition(s) under which the action will be taken.
        pub condition: Option<rule::Condition>,
    }
    /// Nested message and enum types in `Rule`.
    pub mod rule {
        use time::Date;

        // RFC3339 Date part, in format YYYY-MM-DD
        time::serde::format_description!(date_format, Date, "[year]-[month]-[day]");

        #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        pub enum ActionType {
            /// Deletes a Bucket.
            Delete,
            /// Sets the `storage_class` of a Bucket.
            SetStorageClass,
            /// Aborts an incomplete multipart upload and deletes the associated parts when the multipart upload meets the conditions specified in the lifecycle rule.
            AbortIncompleteMultipartUpload,
        }
        /// An action to take on an object.
        #[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
        #[serde(rename_all = "camelCase")]
        pub struct Action {
            pub r#type: ActionType,
            pub storage_class: Option<String>,
        }
        /// A condition of an object which triggers some action.
        #[derive(Clone, PartialEq, Eq, Default, serde::Deserialize, serde::Serialize, Debug)]
        #[serde(rename_all = "camelCase")]
        pub struct Condition {
            pub age: Option<i32>,
            #[serde(default, with = "date_format::option")]
            pub created_before: Option<Date>,
            #[serde(default, with = "date_format::option")]
            pub custom_time_before: Option<Date>,
            pub days_since_custom_time: Option<i32>,
            pub days_since_noncurrent_time: Option<i32>,
            pub is_live: Option<bool>,
            pub matches_storage_class: Option<Vec<String>>,
            #[serde(default, with = "date_format::option")]
            pub noncurrent_time_before: Option<Date>,
            pub num_newer_versions: Option<i32>,
        }
    }
}
/// Logging-related properties of a bucket.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Logging {
    /// The destination bucket where the current bucket's logs should be placed.
    pub log_bucket: String,
    /// A prefix for log object names.
    pub log_object_prefix: String,
}
/// Retention policy properties of a bucket.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RetentionPolicy {
    /// Server-determined value that indicates the time from which policy was
    /// enforced and effective. This value is in
    /// \[<https://tools.ietf.org/html/rfc3339\][RFC> 3339] format.
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub effective_time: Option<OffsetDateTime>,
    /// Once locked, an object retention policy cannot be modified.
    pub is_locked: Option<bool>,
    /// The duration in seconds that objects need to be retained. Retention
    /// duration must be greater than zero and less than 100 years. Note that
    /// enforcement of retention periods less than a day is not guaranteed. Such
    /// periods should only be used for testing purposes.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub retention_period: u64,
}
/// Properties of a bucket related to versioning.
/// For more on GCS versioning, see
/// <https://cloud.google.com/storage/docs/object-versioning>.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Versioning {
    /// While set to true, versioning is fully enabled for this bucket.
    pub enabled: bool,
}
/// Properties of a bucket related to accessing the contents as a static
/// website. For more on hosting a static website via GCS, see
/// <https://cloud.google.com/storage/docs/hosting-static-website>.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
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
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Autoclass {
    /// Enables Autoclass.
    pub enabled: bool,
    /// Latest instant at which the `enabled` bit was flipped.
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub toggle_time: Option<OffsetDateTime>,
}

/// An Identity and Access Management (IAM) policy, which specifies access
/// controls for Google Cloud resources.
///
///
/// A `Policy` is a collection of `bindings`. A `binding` binds one or more
/// `members`, or principals, to a single `role`. Principals can be user
/// accounts, service accounts, Google groups, and domains (such as G Suite). A
/// `role` is a named list of permissions; each `role` can be an IAM predefined
/// role or a user-created custom role.
///
/// For some types of Google Cloud resources, a `binding` can also specify a
/// `condition`, which is a logical expression that allows access to a resource
/// only if the expression evaluates to `true`. A condition can add constraints
/// based on attributes of the request, the resource, or both. To learn which
/// resources support conditions in their IAM policies, see the
/// [IAM documentation](<https://cloud.google.com/iam/help/conditions/resource-policies>).
///
/// For a description of IAM and its features, see the
/// [IAM documentation](<https://cloud.google.com/iam/docs/>).
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Policy {
    /// Specifies the format of the policy.
    ///
    /// Valid values are `0`, `1`, and `3`. Requests that specify an invalid value
    /// are rejected.
    ///
    /// Any operation that affects conditional role bindings must specify version
    /// `3`. This requirement applies to the following operations:
    ///
    /// * Getting a policy that includes a conditional role binding
    /// * Adding a conditional role binding to a policy
    /// * Changing a conditional role binding in a policy
    /// * Removing any role binding, with or without a condition, from a policy
    ///   that includes conditions
    ///
    /// **Important:** If you use IAM Conditions, you must include the `etag` field
    /// whenever you call `setIamPolicy`. If you omit this field, then IAM allows
    /// you to overwrite a version `3` policy with a version `1` policy, and all of
    /// the conditions in the version `3` policy are lost.
    ///
    /// If a policy does not include any conditions, operations on that policy may
    /// specify any valid version or leave the field unset.
    ///
    /// To learn which resources support conditions in their IAM policies, see the
    /// [IAM documentation](<https://cloud.google.com/iam/help/conditions/resource-policies>).
    pub version: i32,
    /// Associates a list of `members`, or principals, with a `role`. Optionally,
    /// may specify a `condition` that determines how and when the `bindings` are
    /// applied. Each of the `bindings` must contain at least one principal.
    ///
    /// The `bindings` in a `Policy` can refer to up to 1,500 principals; up to 250
    /// of these principals can be Google groups. Each occurrence of a principal
    /// counts towards these limits. For example, if the `bindings` grant 50
    /// different roles to `user:alice@example.com`, and not to any other
    /// principal, then you can add another 1,450 principals to the `bindings` in
    /// the `Policy`.
    pub bindings: Vec<Binding>,
    pub etag: String,
}
/// Associates `members`, or principals, with a `role`.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Binding {
    /// Role that is assigned to the list of `members`, or principals.
    /// For example, `roles/viewer`, `roles/editor`, or `roles/owner`.
    pub role: String,
    /// Specifies the principals requesting access for a Cloud Platform resource.
    /// `members` can have the following values:
    ///
    /// * `allUsers`: A special identifier that represents anyone who is on the internet; with or without a Google account.
    ///
    /// * `allAuthenticatedUsers`: A special identifier that represents anyone who is authenticated with a Google account or a service account.
    ///
    /// * `user:{emailid}`: An email address that represents a specific Google account. For example, `alice@example.com` .
    ///
    ///
    /// * `serviceAccount:{emailid}`: An email address that represents a service account. For example, `my-other-app@appspot.gserviceaccount.com`.
    ///
    /// * `group:{emailid}`: An email address that represents a Google group. For example, `admins@example.com`.
    ///
    /// * `deleted:user:{emailid}?uid={uniqueid}`: An email address (plus unique identifier) representing a user that has been recently deleted. For example, `alice@example.com?uid=123456789012345678901`. If the user is recovered, this value reverts to `user:{emailid}` and the recovered user retains the role in the binding.
    ///
    /// * `deleted:serviceAccount:{emailid}?uid={uniqueid}`: An email address (plus unique identifier) representing a service account that has been recently deleted. For example, `my-other-app@appspot.gserviceaccount.com?uid=123456789012345678901`. If the service account is undeleted, this value reverts to `serviceAccount:{emailid}` and the undeleted service account retains the role in the binding.
    ///
    /// * `deleted:group:{emailid}?uid={uniqueid}`: An email address (plus unique identifier) representing a Google group that has been recently deleted. For example, `admins@example.com?uid=123456789012345678901`. If the group is recovered, this value reverts to `group:{emailid}` and the recovered group retains the role in the binding.
    ///
    ///
    /// * `domain:{domain}`: The G Suite domain (primary) that represents all the users of that domain. For example, `google.com` or `example.com`.
    ///
    pub members: Vec<String>,
    /// The condition that is associated with this binding.
    ///
    /// If the condition evaluates to `true`, then this binding applies to the
    /// current request.
    ///
    /// If the condition evaluates to `false`, then this binding does not apply to
    /// the current request. However, a different role binding might grant the same
    /// role to one or more of the principals in this binding.
    ///
    /// To learn which resources support conditions in their IAM policies, see the
    /// [IAM
    /// documentation](<https://cloud.google.com/iam/help/conditions/resource-policies>).
    pub condition: Option<Condition>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Condition {
    /// Textual representation of an expression in Common Expression Language
    /// syntax.
    pub expression: String,
    /// Optional. Title for the expression, i.e. a short string describing
    /// its purpose. This can be used e.g. in UIs which allow to enter the
    /// expression.
    pub title: String,
    /// Optional. Description of the expression. This is a longer text which
    /// describes the expression, e.g. when hovered over it in a UI.
    #[serde(default)]
    pub description: Option<String>,
}
