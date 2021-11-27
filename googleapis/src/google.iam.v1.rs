/// Encapsulates settings provided to GetIamPolicy.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetPolicyOptions {
    /// Optional. The policy format version to be returned.
    ///
    /// Valid values are 0, 1, and 3. Requests specifying an invalid value will be
    /// rejected.
    ///
    /// Requests for policies with any conditional bindings must specify version 3.
    /// Policies without any conditional bindings may specify any valid value or
    /// leave the field unset.
    #[prost(int32, tag = "1")]
    pub requested_policy_version: i32,
}
/// Defines an Identity and Access Management (IAM) policy. It is used to
/// specify access control policies for Cloud Platform resources.
///
///
/// A `Policy` is a collection of `bindings`. A `binding` binds one or more
/// `members` to a single `role`. Members can be user accounts, service accounts,
/// Google groups, and domains (such as G Suite). A `role` is a named list of
/// permissions (defined by IAM or configured by users). A `binding` can
/// optionally specify a `condition`, which is a logic expression that further
/// constrains the role binding based on attributes about the request and/or
/// target resource.
///
/// **JSON Example**
///
///     {
///       "bindings": [
///         {
///           "role": "roles/resourcemanager.organizationAdmin",
///           "members": [
///             "user:mike@example.com",
///             "group:admins@example.com",
///             "domain:google.com",
///             "serviceAccount:my-project-id@appspot.gserviceaccount.com"
///           ]
///         },
///         {
///           "role": "roles/resourcemanager.organizationViewer",
///           "members": \["user:eve@example.com"\],
///           "condition": {
///             "title": "expirable access",
///             "description": "Does not grant access after Sep 2020",
///             "expression": "request.time <
///             timestamp('2020-10-01T00:00:00.000Z')",
///           }
///         }
///       ]
///     }
///
/// **YAML Example**
///
///     bindings:
///     - members:
///       - user:mike@example.com
///       - group:admins@example.com
///       - domain:google.com
///       - serviceAccount:my-project-id@appspot.gserviceaccount.com
///       role: roles/resourcemanager.organizationAdmin
///     - members:
///       - user:eve@example.com
///       role: roles/resourcemanager.organizationViewer
///       condition:
///         title: expirable access
///         description: Does not grant access after Sep 2020
///         expression: request.time < timestamp('2020-10-01T00:00:00.000Z')
///
/// For a description of IAM and its features, see the
/// [IAM developer's guide](<https://cloud.google.com/iam/docs>).
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Policy {
    /// Specifies the format of the policy.
    ///
    /// Valid values are 0, 1, and 3. Requests specifying an invalid value will be
    /// rejected.
    ///
    /// Operations affecting conditional bindings must specify version 3. This can
    /// be either setting a conditional policy, modifying a conditional binding,
    /// or removing a binding (conditional or unconditional) from the stored
    /// conditional policy.
    /// Operations on non-conditional policies may specify any valid value or
    /// leave the field unset.
    ///
    /// If no etag is provided in the call to `setIamPolicy`, version compliance
    /// checks against the stored policy is skipped.
    #[prost(int32, tag = "1")]
    pub version: i32,
    /// Associates a list of `members` to a `role`. Optionally may specify a
    /// `condition` that determines when binding is in effect.
    /// `bindings` with no members will result in an error.
    #[prost(message, repeated, tag = "4")]
    pub bindings: ::prost::alloc::vec::Vec<Binding>,
    /// `etag` is used for optimistic concurrency control as a way to help
    /// prevent simultaneous updates of a policy from overwriting each other.
    /// It is strongly suggested that systems make use of the `etag` in the
    /// read-modify-write cycle to perform policy updates in order to avoid race
    /// conditions: An `etag` is returned in the response to `getIamPolicy`, and
    /// systems are expected to put that etag in the request to `setIamPolicy` to
    /// ensure that their change will be applied to the same version of the policy.
    ///
    /// If no `etag` is provided in the call to `setIamPolicy`, then the existing
    /// policy is overwritten. Due to blind-set semantics of an etag-less policy,
    /// 'setIamPolicy' will not fail even if the incoming policy version does not
    /// meet the requirements for modifying the stored policy.
    #[prost(bytes = "vec", tag = "3")]
    pub etag: ::prost::alloc::vec::Vec<u8>,
}
/// Associates `members` with a `role`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Binding {
    /// Role that is assigned to `members`.
    /// For example, `roles/viewer`, `roles/editor`, or `roles/owner`.
    #[prost(string, tag = "1")]
    pub role: ::prost::alloc::string::String,
    /// Specifies the identities requesting access for a Cloud Platform resource.
    /// `members` can have the following values:
    ///
    /// * `allUsers`: A special identifier that represents anyone who is
    ///    on the internet; with or without a Google account.
    ///
    /// * `allAuthenticatedUsers`: A special identifier that represents anyone
    ///    who is authenticated with a Google account or a service account.
    ///
    /// * `user:{emailid}`: An email address that represents a specific Google
    ///    account. For example, `alice@example.com` .
    ///
    ///
    /// * `serviceAccount:{emailid}`: An email address that represents a service
    ///    account. For example, `my-other-app@appspot.gserviceaccount.com`.
    ///
    /// * `group:{emailid}`: An email address that represents a Google group.
    ///    For example, `admins@example.com`.
    ///
    ///
    /// * `domain:{domain}`: The G Suite domain (primary) that represents all the
    ///    users of that domain. For example, `google.com` or `example.com`.
    ///
    ///
    #[prost(string, repeated, tag = "2")]
    pub members: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// The condition that is associated with this binding.
    /// NOTE: An unsatisfied condition will not allow user access via current
    /// binding. Different bindings, including their conditions, are examined
    /// independently.
    #[prost(message, optional, tag = "3")]
    pub condition: ::core::option::Option<super::super::r#type::Expr>,
}
/// The difference delta between two policies.
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
    /// A single identity requesting access for a Cloud Platform resource.
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
}
/// One delta entry for AuditConfig. Each individual change (only one
/// exempted_member in each entry) to a AuditConfig will be a separate entry.
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
}
/// Request message for `SetIamPolicy` method.
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
}
/// Request message for `GetIamPolicy` method.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetIamPolicyRequest {
    /// REQUIRED: The resource for which the policy is being requested.
    /// See the operation documentation for the appropriate value for this field.
    #[prost(string, tag = "1")]
    pub resource: ::prost::alloc::string::String,
    /// OPTIONAL: A `GetPolicyOptions` object for specifying options to
    /// `GetIamPolicy`. This field is only used by Cloud IAM.
    #[prost(message, optional, tag = "2")]
    pub options: ::core::option::Option<GetPolicyOptions>,
}
/// Request message for `TestIamPermissions` method.
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
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TestIamPermissionsResponse {
    /// A subset of `TestPermissionsRequest.permissions` that the caller is
    /// allowed.
    #[prost(string, repeated, tag = "1")]
    pub permissions: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[doc = r" Generated client implementations."]
pub mod iam_policy_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[doc = " ## API Overview"]
    #[doc = ""]
    #[doc = " Manages Identity and Access Management (IAM) policies."]
    #[doc = ""]
    #[doc = " Any implementation of an API that offers access control features"]
    #[doc = " implements the google.iam.v1.IAMPolicy interface."]
    #[doc = ""]
    #[doc = " ## Data model"]
    #[doc = ""]
    #[doc = " Access control is applied when a principal (user or service account), takes"]
    #[doc = " some action on a resource exposed by a service. Resources, identified by"]
    #[doc = " URI-like names, are the unit of access control specification. Service"]
    #[doc = " implementations can choose the granularity of access control and the"]
    #[doc = " supported permissions for their resources."]
    #[doc = " For example one database service may allow access control to be"]
    #[doc = " specified only at the Table level, whereas another might allow access control"]
    #[doc = " to also be specified at the Column level."]
    #[doc = ""]
    #[doc = " ## Policy Structure"]
    #[doc = ""]
    #[doc = " See google.iam.v1.Policy"]
    #[doc = ""]
    #[doc = " This is intentionally not a CRUD style API because access control policies"]
    #[doc = " are created and deleted implicitly with the resources to which they are"]
    #[doc = " attached."]
    #[derive(Debug, Clone)]
    pub struct IamPolicyClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl IamPolicyClient<tonic::transport::Channel> {
        #[doc = r" Attempt to create a new client by connecting to a given endpoint."]
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> IamPolicyClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::ResponseBody: Body + Send + 'static,
        T::Error: Into<StdError>,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> IamPolicyClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
                Into<StdError> + Send + Sync,
        {
            IamPolicyClient::new(InterceptedService::new(inner, interceptor))
        }
        #[doc = r" Compress requests with `gzip`."]
        #[doc = r""]
        #[doc = r" This requires the server to support it otherwise it might respond with an"]
        #[doc = r" error."]
        pub fn send_gzip(mut self) -> Self {
            self.inner = self.inner.send_gzip();
            self
        }
        #[doc = r" Enable decompressing responses with `gzip`."]
        pub fn accept_gzip(mut self) -> Self {
            self.inner = self.inner.accept_gzip();
            self
        }
        #[doc = " Sets the access control policy on the specified resource. Replaces any"]
        #[doc = " existing policy."]
        pub async fn set_iam_policy(
            &mut self,
            request: impl tonic::IntoRequest<super::SetIamPolicyRequest>,
        ) -> Result<tonic::Response<super::Policy>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/google.iam.v1.IAMPolicy/SetIamPolicy");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Gets the access control policy for a resource."]
        #[doc = " Returns an empty policy if the resource exists and does not have a policy"]
        #[doc = " set."]
        pub async fn get_iam_policy(
            &mut self,
            request: impl tonic::IntoRequest<super::GetIamPolicyRequest>,
        ) -> Result<tonic::Response<super::Policy>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/google.iam.v1.IAMPolicy/GetIamPolicy");
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = " Returns permissions that a caller has on the specified resource."]
        #[doc = " If the resource does not exist, this will return an empty set of"]
        #[doc = " permissions, not a NOT_FOUND error."]
        #[doc = ""]
        #[doc = " Note: This operation is designed to be used for building permission-aware"]
        #[doc = " UIs and command-line tools, not for authorization checking. This operation"]
        #[doc = " may \"fail open\" without warning."]
        pub async fn test_iam_permissions(
            &mut self,
            request: impl tonic::IntoRequest<super::TestIamPermissionsRequest>,
        ) -> Result<tonic::Response<super::TestIamPermissionsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/google.iam.v1.IAMPolicy/TestIamPermissions");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
