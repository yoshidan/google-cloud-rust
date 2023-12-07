/// Encapsulates settings provided to GetIamPolicy.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetPolicyOptions {
    /// Optional. The maximum policy version that will be used to format the
    /// policy.
    ///
    /// Valid values are 0, 1, and 3. Requests specifying an invalid value will be
    /// rejected.
    ///
    /// Requests for policies with any conditional role bindings must specify
    /// version 3. Policies with no conditional role bindings may specify any valid
    /// value or leave the field unset.
    ///
    /// The policy in the response might use the policy version that you specified,
    /// or it might use a lower policy version. For example, if you specify version
    /// 3, but the policy has no conditional role bindings, the response uses
    /// version 1.
    ///
    /// To learn which resources support conditions in their IAM policies, see the
    /// [IAM
    /// documentation](<https://cloud.google.com/iam/help/conditions/resource-policies>).
    #[prost(int32, tag = "1")]
    pub requested_policy_version: i32,
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
/// [IAM
/// documentation](<https://cloud.google.com/iam/help/conditions/resource-policies>).
///
/// **JSON example:**
///
/// ```
///      {
///        "bindings": [
///          {
///            "role": "roles/resourcemanager.organizationAdmin",
///            "members": [
///              "user:mike@example.com",
///              "group:admins@example.com",
///              "domain:google.com",
///              "serviceAccount:my-project-id@appspot.gserviceaccount.com"
///            ]
///          },
///          {
///            "role": "roles/resourcemanager.organizationViewer",
///            "members": [
///              "user:eve@example.com"
///            ],
///            "condition": {
///              "title": "expirable access",
///              "description": "Does not grant access after Sep 2020",
///              "expression": "request.time <
///              timestamp('2020-10-01T00:00:00.000Z')",
///            }
///          }
///        ],
///        "etag": "BwWWja0YfJA=",
///        "version": 3
///      }
/// ```
///
/// **YAML example:**
///
/// ```
///      bindings:
///      - members:
///        - user:mike@example.com
///        - group:admins@example.com
///        - domain:google.com
///        - serviceAccount:my-project-id@appspot.gserviceaccount.com
///        role: roles/resourcemanager.organizationAdmin
///      - members:
///        - user:eve@example.com
///        role: roles/resourcemanager.organizationViewer
///        condition:
///          title: expirable access
///          description: Does not grant access after Sep 2020
///          expression: request.time < timestamp('2020-10-01T00:00:00.000Z')
///      etag: BwWWja0YfJA=
///      version: 3
/// ```
///
/// For a description of IAM and its features, see the
/// [IAM documentation](<https://cloud.google.com/iam/docs/>).
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
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
    ///    that includes conditions
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
    /// [IAM
    /// documentation](<https://cloud.google.com/iam/help/conditions/resource-policies>).
    #[prost(int32, tag = "1")]
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
    #[prost(message, repeated, tag = "4")]
    pub bindings: ::prost::alloc::vec::Vec<Binding>,
    /// Specifies cloud audit logging configuration for this policy.
    #[prost(message, repeated, tag = "6")]
    pub audit_configs: ::prost::alloc::vec::Vec<AuditConfig>,
    /// `etag` is used for optimistic concurrency control as a way to help
    /// prevent simultaneous updates of a policy from overwriting each other.
    /// It is strongly suggested that systems make use of the `etag` in the
    /// read-modify-write cycle to perform policy updates in order to avoid race
    /// conditions: An `etag` is returned in the response to `getIamPolicy`, and
    /// systems are expected to put that etag in the request to `setIamPolicy` to
    /// ensure that their change will be applied to the same version of the policy.
    ///
    /// **Important:** If you use IAM Conditions, you must include the `etag` field
    /// whenever you call `setIamPolicy`. If you omit this field, then IAM allows
    /// you to overwrite a version `3` policy with a version `1` policy, and all of
    /// the conditions in the version `3` policy are lost.
    #[prost(bytes = "vec", tag = "3")]
    pub etag: ::prost::alloc::vec::Vec<u8>,
}
/// Associates `members`, or principals, with a `role`.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Binding {
    /// Role that is assigned to the list of `members`, or principals.
    /// For example, `roles/viewer`, `roles/editor`, or `roles/owner`.
    #[prost(string, tag = "1")]
    pub role: ::prost::alloc::string::String,
    /// Specifies the principals requesting access for a Google Cloud resource.
    /// `members` can have the following values:
    ///
    /// * `allUsers`: A special identifier that represents anyone who is
    ///     on the internet; with or without a Google account.
    ///
    /// * `allAuthenticatedUsers`: A special identifier that represents anyone
    ///     who is authenticated with a Google account or a service account.
    ///
    /// * `user:{emailid}`: An email address that represents a specific Google
    ///     account. For example, `alice@example.com` .
    ///
    ///
    /// * `serviceAccount:{emailid}`: An email address that represents a service
    ///     account. For example, `my-other-app@appspot.gserviceaccount.com`.
    ///
    /// * `group:{emailid}`: An email address that represents a Google group.
    ///     For example, `admins@example.com`.
    ///
    /// * `deleted:user:{emailid}?uid={uniqueid}`: An email address (plus unique
    ///     identifier) representing a user that has been recently deleted. For
    ///     example, `alice@example.com?uid=123456789012345678901`. If the user is
    ///     recovered, this value reverts to `user:{emailid}` and the recovered user
    ///     retains the role in the binding.
    ///
    /// * `deleted:serviceAccount:{emailid}?uid={uniqueid}`: An email address (plus
    ///     unique identifier) representing a service account that has been recently
    ///     deleted. For example,
    ///     `my-other-app@appspot.gserviceaccount.com?uid=123456789012345678901`.
    ///     If the service account is undeleted, this value reverts to
    ///     `serviceAccount:{emailid}` and the undeleted service account retains the
    ///     role in the binding.
    ///
    /// * `deleted:group:{emailid}?uid={uniqueid}`: An email address (plus unique
    ///     identifier) representing a Google group that has been recently
    ///     deleted. For example, `admins@example.com?uid=123456789012345678901`. If
    ///     the group is recovered, this value reverts to `group:{emailid}` and the
    ///     recovered group retains the role in the binding.
    ///
    ///
    /// * `domain:{domain}`: The G Suite domain (primary) that represents all the
    ///     users of that domain. For example, `google.com` or `example.com`.
    ///
    ///
    #[prost(string, repeated, tag = "2")]
    pub members: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
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
    #[prost(message, optional, tag = "3")]
    pub condition: ::core::option::Option<super::super::r#type::Expr>,
}
/// Specifies the audit configuration for a service.
/// The configuration determines which permission types are logged, and what
/// identities, if any, are exempted from logging.
/// An AuditConfig must have one or more AuditLogConfigs.
///
/// If there are AuditConfigs for both `allServices` and a specific service,
/// the union of the two AuditConfigs is used for that service: the log_types
/// specified in each AuditConfig are enabled, and the exempted_members in each
/// AuditLogConfig are exempted.
///
/// Example Policy with multiple AuditConfigs:
///
///      {
///        "audit_configs": [
///          {
///            "service": "allServices",
///            "audit_log_configs": [
///              {
///                "log_type": "DATA_READ",
///                "exempted_members": [
///                  "user:jose@example.com"
///                ]
///              },
///              {
///                "log_type": "DATA_WRITE"
///              },
///              {
///                "log_type": "ADMIN_READ"
///              }
///            ]
///          },
///          {
///            "service": "sampleservice.googleapis.com",
///            "audit_log_configs": [
///              {
///                "log_type": "DATA_READ"
///              },
///              {
///                "log_type": "DATA_WRITE",
///                "exempted_members": [
///                  "user:aliya@example.com"
///                ]
///              }
///            ]
///          }
///        ]
///      }
///
/// For sampleservice, this policy enables DATA_READ, DATA_WRITE and ADMIN_READ
/// logging. It also exempts `jose@example.com` from DATA_READ logging, and
/// `aliya@example.com` from DATA_WRITE logging.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AuditConfig {
    /// Specifies a service that will be enabled for audit logging.
    /// For example, `storage.googleapis.com`, `cloudsql.googleapis.com`.
    /// `allServices` is a special value that covers all services.
    #[prost(string, tag = "1")]
    pub service: ::prost::alloc::string::String,
    /// The configuration for logging of each type of permission.
    #[prost(message, repeated, tag = "3")]
    pub audit_log_configs: ::prost::alloc::vec::Vec<AuditLogConfig>,
}
/// Provides the configuration for logging a type of permissions.
/// Example:
///
///      {
///        "audit_log_configs": [
///          {
///            "log_type": "DATA_READ",
///            "exempted_members": [
///              "user:jose@example.com"
///            ]
///          },
///          {
///            "log_type": "DATA_WRITE"
///          }
///        ]
///      }
///
/// This enables 'DATA_READ' and 'DATA_WRITE' logging, while exempting
/// jose@example.com from DATA_READ logging.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AuditLogConfig {
    /// The log type that this config enables.
    #[prost(enumeration = "audit_log_config::LogType", tag = "1")]
    pub log_type: i32,
    /// Specifies the identities that do not cause logging for this type of
    /// permission.
    /// Follows the same format of
    /// \[Binding.members][google.iam.v1.Binding.members\].
    #[prost(string, repeated, tag = "2")]
    pub exempted_members: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// Nested message and enum types in `AuditLogConfig`.
pub mod audit_log_config {
    /// The list of valid permission types for which logging can be configured.
    /// Admin writes are always logged, and are not configurable.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum LogType {
        /// Default case. Should never be this.
        Unspecified = 0,
        /// Admin reads. Example: CloudIAM getIamPolicy
        AdminRead = 1,
        /// Data writes. Example: CloudSQL Users create
        DataWrite = 2,
        /// Data reads. Example: CloudSQL Users list
        DataRead = 3,
    }
    impl LogType {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                LogType::Unspecified => "LOG_TYPE_UNSPECIFIED",
                LogType::AdminRead => "ADMIN_READ",
                LogType::DataWrite => "DATA_WRITE",
                LogType::DataRead => "DATA_READ",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "LOG_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
                "ADMIN_READ" => Some(Self::AdminRead),
                "DATA_WRITE" => Some(Self::DataWrite),
                "DATA_READ" => Some(Self::DataRead),
                _ => None,
            }
        }
    }
}
/// The difference delta between two policies.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PolicyDelta {
    /// The delta for Bindings between two policies.
    #[prost(message, repeated, tag = "1")]
    pub binding_deltas: ::prost::alloc::vec::Vec<BindingDelta>,
    /// The delta for AuditConfigs between two policies.
    #[prost(message, repeated, tag = "2")]
    pub audit_config_deltas: ::prost::alloc::vec::Vec<AuditConfigDelta>,
}
/// One delta entry for Binding. Each individual change (only one member in each
/// entry) to a binding will be a separate entry.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BindingDelta {
    /// The action that was performed on a Binding.
    /// Required
    #[prost(enumeration = "binding_delta::Action", tag = "1")]
    pub action: i32,
    /// Role that is assigned to `members`.
    /// For example, `roles/viewer`, `roles/editor`, or `roles/owner`.
    /// Required
    #[prost(string, tag = "2")]
    pub role: ::prost::alloc::string::String,
    /// A single identity requesting access for a Google Cloud resource.
    /// Follows the same format of Binding.members.
    /// Required
    #[prost(string, tag = "3")]
    pub member: ::prost::alloc::string::String,
    /// The condition that is associated with this binding.
    #[prost(message, optional, tag = "4")]
    pub condition: ::core::option::Option<super::super::r#type::Expr>,
}
/// Nested message and enum types in `BindingDelta`.
pub mod binding_delta {
    /// The type of action performed on a Binding in a policy.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Action {
        /// Unspecified.
        Unspecified = 0,
        /// Addition of a Binding.
        Add = 1,
        /// Removal of a Binding.
        Remove = 2,
    }
    impl Action {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Action::Unspecified => "ACTION_UNSPECIFIED",
                Action::Add => "ADD",
                Action::Remove => "REMOVE",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "ACTION_UNSPECIFIED" => Some(Self::Unspecified),
                "ADD" => Some(Self::Add),
                "REMOVE" => Some(Self::Remove),
                _ => None,
            }
        }
    }
}
/// One delta entry for AuditConfig. Each individual change (only one
/// exempted_member in each entry) to a AuditConfig will be a separate entry.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AuditConfigDelta {
    /// The action that was performed on an audit configuration in a policy.
    /// Required
    #[prost(enumeration = "audit_config_delta::Action", tag = "1")]
    pub action: i32,
    /// Specifies a service that was configured for Cloud Audit Logging.
    /// For example, `storage.googleapis.com`, `cloudsql.googleapis.com`.
    /// `allServices` is a special value that covers all services.
    /// Required
    #[prost(string, tag = "2")]
    pub service: ::prost::alloc::string::String,
    /// A single identity that is exempted from "data access" audit
    /// logging for the `service` specified above.
    /// Follows the same format of Binding.members.
    #[prost(string, tag = "3")]
    pub exempted_member: ::prost::alloc::string::String,
    /// Specifies the log_type that was be enabled. ADMIN_ACTIVITY is always
    /// enabled, and cannot be configured.
    /// Required
    #[prost(string, tag = "4")]
    pub log_type: ::prost::alloc::string::String,
}
/// Nested message and enum types in `AuditConfigDelta`.
pub mod audit_config_delta {
    /// The type of action performed on an audit configuration in a policy.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Action {
        /// Unspecified.
        Unspecified = 0,
        /// Addition of an audit configuration.
        Add = 1,
        /// Removal of an audit configuration.
        Remove = 2,
    }
    impl Action {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Action::Unspecified => "ACTION_UNSPECIFIED",
                Action::Add => "ADD",
                Action::Remove => "REMOVE",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "ACTION_UNSPECIFIED" => Some(Self::Unspecified),
                "ADD" => Some(Self::Add),
                "REMOVE" => Some(Self::Remove),
                _ => None,
            }
        }
    }
}
/// Request message for `SetIamPolicy` method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetIamPolicyRequest {
    /// REQUIRED: The resource for which the policy is being specified.
    /// See the operation documentation for the appropriate value for this field.
    #[prost(string, tag = "1")]
    pub resource: ::prost::alloc::string::String,
    /// REQUIRED: The complete policy to be applied to the `resource`. The size of
    /// the policy is limited to a few 10s of KB. An empty policy is a
    /// valid policy but certain Cloud Platform services (such as Projects)
    /// might reject them.
    #[prost(message, optional, tag = "2")]
    pub policy: ::core::option::Option<Policy>,
    /// OPTIONAL: A FieldMask specifying which fields of the policy to modify. Only
    /// the fields in the mask will be modified. If no mask is provided, the
    /// following default mask is used:
    ///
    /// `paths: "bindings, etag"`
    #[prost(message, optional, tag = "3")]
    pub update_mask: ::core::option::Option<::prost_types::FieldMask>,
}
/// Request message for `GetIamPolicy` method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetIamPolicyRequest {
    /// REQUIRED: The resource for which the policy is being requested.
    /// See the operation documentation for the appropriate value for this field.
    #[prost(string, tag = "1")]
    pub resource: ::prost::alloc::string::String,
    /// OPTIONAL: A `GetPolicyOptions` object for specifying options to
    /// `GetIamPolicy`.
    #[prost(message, optional, tag = "2")]
    pub options: ::core::option::Option<GetPolicyOptions>,
}
/// Request message for `TestIamPermissions` method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TestIamPermissionsRequest {
    /// REQUIRED: The resource for which the policy detail is being requested.
    /// See the operation documentation for the appropriate value for this field.
    #[prost(string, tag = "1")]
    pub resource: ::prost::alloc::string::String,
    /// The set of permissions to check for the `resource`. Permissions with
    /// wildcards (such as '*' or 'storage.*') are not allowed. For more
    /// information see
    /// [IAM Overview](<https://cloud.google.com/iam/docs/overview#permissions>).
    #[prost(string, repeated, tag = "2")]
    pub permissions: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// Response message for `TestIamPermissions` method.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TestIamPermissionsResponse {
    /// A subset of `TestPermissionsRequest.permissions` that the caller is
    /// allowed.
    #[prost(string, repeated, tag = "1")]
    pub permissions: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// Generated client implementations.
pub mod iam_policy_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::http::Uri;
    use tonic::codegen::*;
    /// API Overview
    ///
    ///
    /// Manages Identity and Access Management (IAM) policies.
    ///
    /// Any implementation of an API that offers access control features
    /// implements the google.iam.v1.IAMPolicy interface.
    ///
    /// ## Data model
    ///
    /// Access control is applied when a principal (user or service account), takes
    /// some action on a resource exposed by a service. Resources, identified by
    /// URI-like names, are the unit of access control specification. Service
    /// implementations can choose the granularity of access control and the
    /// supported permissions for their resources.
    /// For example one database service may allow access control to be
    /// specified only at the Table level, whereas another might allow access control
    /// to also be specified at the Column level.
    ///
    /// ## Policy Structure
    ///
    /// See google.iam.v1.Policy
    ///
    /// This is intentionally not a CRUD style API because access control policies
    /// are created and deleted implicitly with the resources to which they are
    /// attached.
    #[derive(Debug, Clone)]
    pub struct IamPolicyClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl IamPolicyClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> IamPolicyClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> IamPolicyClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<<T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody>,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error: Into<StdError> + Send + Sync,
        {
            IamPolicyClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        /// Sets the access control policy on the specified resource. Replaces any
        /// existing policy.
        ///
        /// Can return `NOT_FOUND`, `INVALID_ARGUMENT`, and `PERMISSION_DENIED` errors.
        pub async fn set_iam_policy(
            &mut self,
            request: impl tonic::IntoRequest<super::SetIamPolicyRequest>,
        ) -> std::result::Result<tonic::Response<super::Policy>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.iam.v1.IAMPolicy/SetIamPolicy");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("google.iam.v1.IAMPolicy", "SetIamPolicy"));
            self.inner.unary(req, path, codec).await
        }
        /// Gets the access control policy for a resource.
        /// Returns an empty policy if the resource exists and does not have a policy
        /// set.
        pub async fn get_iam_policy(
            &mut self,
            request: impl tonic::IntoRequest<super::GetIamPolicyRequest>,
        ) -> std::result::Result<tonic::Response<super::Policy>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.iam.v1.IAMPolicy/GetIamPolicy");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("google.iam.v1.IAMPolicy", "GetIamPolicy"));
            self.inner.unary(req, path, codec).await
        }
        /// Returns permissions that a caller has on the specified resource.
        /// If the resource does not exist, this will return an empty set of
        /// permissions, not a `NOT_FOUND` error.
        ///
        /// Note: This operation is designed to be used for building permission-aware
        /// UIs and command-line tools, not for authorization checking. This operation
        /// may "fail open" without warning.
        pub async fn test_iam_permissions(
            &mut self,
            request: impl tonic::IntoRequest<super::TestIamPermissionsRequest>,
        ) -> std::result::Result<tonic::Response<super::TestIamPermissionsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.iam.v1.IAMPolicy/TestIamPermissions");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("google.iam.v1.IAMPolicy", "TestIamPermissions"));
            self.inner.unary(req, path, codec).await
        }
    }
}
