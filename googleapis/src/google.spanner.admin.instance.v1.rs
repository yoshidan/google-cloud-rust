#[derive(Clone, PartialEq, Eq, ::prost::Message)]
pub struct ReplicaInfo {
    /// The location of the serving resources, e.g. "us-central1".
    #[prost(string, tag = "1")]
    pub location: ::prost::alloc::string::String,
    /// The type of replica.
    #[prost(enumeration = "replica_info::ReplicaType", tag = "2")]
    pub r#type: i32,
    /// If true, this location is designated as the default leader location where
    /// leader replicas are placed. See the [region types
    /// documentation](<https://cloud.google.com/spanner/docs/instances#region_types>)
    /// for more details.
    #[prost(bool, tag = "3")]
    pub default_leader_location: bool,
}
/// Nested message and enum types in `ReplicaInfo`.
pub mod replica_info {
    /// Indicates the type of replica.  See the [replica types
    /// documentation](<https://cloud.google.com/spanner/docs/replication#replica_types>)
    /// for more details.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum ReplicaType {
        /// Not specified.
        TypeUnspecified = 0,
        /// Read-write replicas support both reads and writes. These replicas:
        ///
        /// * Maintain a full copy of your data.
        /// * Serve reads.
        /// * Can vote whether to commit a write.
        /// * Participate in leadership election.
        /// * Are eligible to become a leader.
        ReadWrite = 1,
        /// Read-only replicas only support reads (not writes). Read-only replicas:
        ///
        /// * Maintain a full copy of your data.
        /// * Serve reads.
        /// * Do not participate in voting to commit writes.
        /// * Are not eligible to become a leader.
        ReadOnly = 2,
        /// Witness replicas don't support reads but do participate in voting to
        /// commit writes. Witness replicas:
        ///
        /// * Do not maintain a full copy of data.
        /// * Do not serve reads.
        /// * Vote whether to commit writes.
        /// * Participate in leader election but are not eligible to become leader.
        Witness = 3,
    }
}
/// A possible configuration for a Cloud Spanner instance. Configurations
/// define the geographic placement of nodes and their replication.
#[derive(Clone, PartialEq, Eq, ::prost::Message)]
pub struct InstanceConfig {
    /// A unique identifier for the instance configuration.  Values
    /// are of the form
    /// `projects/<project>/instanceConfigs/\[a-z][-a-z0-9\]*`.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// The name of this instance configuration as it appears in UIs.
    #[prost(string, tag = "2")]
    pub display_name: ::prost::alloc::string::String,
    /// The geographic placement of nodes in this instance configuration and their
    /// replication properties.
    #[prost(message, repeated, tag = "3")]
    pub replicas: ::prost::alloc::vec::Vec<ReplicaInfo>,
    /// Allowed values of the "default_leader" schema option for databases in
    /// instances that use this instance configuration.
    #[prost(string, repeated, tag = "4")]
    pub leader_options: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// An isolated set of Cloud Spanner resources on which databases can be hosted.
#[derive(Clone, PartialEq, Eq, ::prost::Message)]
pub struct Instance {
    /// Required. A unique identifier for the instance, which cannot be changed
    /// after the instance is created. Values are of the form
    /// `projects/<project>/instances/\[a-z][-a-z0-9]*[a-z0-9\]`. The final
    /// segment of the name must be between 2 and 64 characters in length.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// Required. The name of the instance's configuration. Values are of the form
    /// `projects/<project>/instanceConfigs/<configuration>`. See
    /// also \[InstanceConfig][google.spanner.admin.instance.v1.InstanceConfig\] and
    /// \[ListInstanceConfigs][google.spanner.admin.instance.v1.InstanceAdmin.ListInstanceConfigs\].
    #[prost(string, tag = "2")]
    pub config: ::prost::alloc::string::String,
    /// Required. The descriptive name for this instance as it appears in UIs.
    /// Must be unique per project and between 4 and 30 characters in length.
    #[prost(string, tag = "3")]
    pub display_name: ::prost::alloc::string::String,
    /// The number of nodes allocated to this instance. At most one of either
    /// node_count or processing_units should be present in the message. This
    /// may be zero in API responses for instances that are not yet in state
    /// `READY`.
    ///
    /// See [the
    /// documentation](<https://cloud.google.com/spanner/docs/compute-capacity>)
    /// for more information about nodes and processing units.
    #[prost(int32, tag = "5")]
    pub node_count: i32,
    /// The number of processing units allocated to this instance. At most one of
    /// processing_units or node_count should be present in the message. This may
    /// be zero in API responses for instances that are not yet in state `READY`.
    ///
    /// See [the
    /// documentation](<https://cloud.google.com/spanner/docs/compute-capacity>)
    /// for more information about nodes and processing units.
    #[prost(int32, tag = "9")]
    pub processing_units: i32,
    /// Output only. The current instance state. For
    /// \[CreateInstance][google.spanner.admin.instance.v1.InstanceAdmin.CreateInstance\], the state must be
    /// either omitted or set to `CREATING`. For
    /// \[UpdateInstance][google.spanner.admin.instance.v1.InstanceAdmin.UpdateInstance\], the state must be
    /// either omitted or set to `READY`.
    #[prost(enumeration = "instance::State", tag = "6")]
    pub state: i32,
    /// Cloud Labels are a flexible and lightweight mechanism for organizing cloud
    /// resources into groups that reflect a customer's organizational needs and
    /// deployment strategies. Cloud Labels can be used to filter collections of
    /// resources. They can be used to control how resource metrics are aggregated.
    /// And they can be used as arguments to policy management rules (e.g. route,
    /// firewall, load balancing, etc.).
    ///
    ///  * Label keys must be between 1 and 63 characters long and must conform to
    ///    the following regular expression: `\[a-z][a-z0-9_-\]{0,62}`.
    ///  * Label values must be between 0 and 63 characters long and must conform
    ///    to the regular expression `\[a-z0-9_-\]{0,63}`.
    ///  * No more than 64 labels can be associated with a given resource.
    ///
    /// See <https://goo.gl/xmQnxf> for more information on and examples of labels.
    ///
    /// If you plan to use labels in your own code, please note that additional
    /// characters may be allowed in the future. And so you are advised to use an
    /// internal label representation, such as JSON, which doesn't rely upon
    /// specific characters being disallowed.  For example, representing labels
    /// as the string:  name + "_" + value  would prove problematic if we were to
    /// allow "_" in a future release.
    #[prost(map = "string, string", tag = "7")]
    pub labels: ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
    /// Deprecated. This field is not populated.
    #[prost(string, repeated, tag = "8")]
    pub endpoint_uris: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Output only. The time at which the instance was created.
    #[prost(message, optional, tag = "11")]
    pub create_time: ::core::option::Option<::prost_types::Timestamp>,
    /// Output only. The time at which the instance was most recently updated.
    #[prost(message, optional, tag = "12")]
    pub update_time: ::core::option::Option<::prost_types::Timestamp>,
}
/// Nested message and enum types in `Instance`.
pub mod instance {
    /// Indicates the current state of the instance.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum State {
        /// Not specified.
        Unspecified = 0,
        /// The instance is still being created. Resources may not be
        /// available yet, and operations such as database creation may not
        /// work.
        Creating = 1,
        /// The instance is fully created and ready to do work such as
        /// creating databases.
        Ready = 2,
    }
}
/// The request for \[ListInstanceConfigs][google.spanner.admin.instance.v1.InstanceAdmin.ListInstanceConfigs\].
#[derive(Clone, PartialEq, Eq, ::prost::Message)]
pub struct ListInstanceConfigsRequest {
    /// Required. The name of the project for which a list of supported instance
    /// configurations is requested. Values are of the form
    /// `projects/<project>`.
    #[prost(string, tag = "1")]
    pub parent: ::prost::alloc::string::String,
    /// Number of instance configurations to be returned in the response. If 0 or
    /// less, defaults to the server's maximum allowed page size.
    #[prost(int32, tag = "2")]
    pub page_size: i32,
    /// If non-empty, `page_token` should contain a
    /// \[next_page_token][google.spanner.admin.instance.v1.ListInstanceConfigsResponse.next_page_token\]
    /// from a previous \[ListInstanceConfigsResponse][google.spanner.admin.instance.v1.ListInstanceConfigsResponse\].
    #[prost(string, tag = "3")]
    pub page_token: ::prost::alloc::string::String,
}
/// The response for \[ListInstanceConfigs][google.spanner.admin.instance.v1.InstanceAdmin.ListInstanceConfigs\].
#[derive(Clone, PartialEq, Eq, ::prost::Message)]
pub struct ListInstanceConfigsResponse {
    /// The list of requested instance configurations.
    #[prost(message, repeated, tag = "1")]
    pub instance_configs: ::prost::alloc::vec::Vec<InstanceConfig>,
    /// `next_page_token` can be sent in a subsequent
    /// \[ListInstanceConfigs][google.spanner.admin.instance.v1.InstanceAdmin.ListInstanceConfigs\] call to
    /// fetch more of the matching instance configurations.
    #[prost(string, tag = "2")]
    pub next_page_token: ::prost::alloc::string::String,
}
/// The request for
/// \[GetInstanceConfigRequest][google.spanner.admin.instance.v1.InstanceAdmin.GetInstanceConfig\].
#[derive(Clone, PartialEq, Eq, ::prost::Message)]
pub struct GetInstanceConfigRequest {
    /// Required. The name of the requested instance configuration. Values are of
    /// the form `projects/<project>/instanceConfigs/<config>`.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
/// The request for \[GetInstance][google.spanner.admin.instance.v1.InstanceAdmin.GetInstance\].
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetInstanceRequest {
    /// Required. The name of the requested instance. Values are of the form
    /// `projects/<project>/instances/<instance>`.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// If field_mask is present, specifies the subset of \[Instance][google.spanner.admin.instance.v1.Instance\] fields that
    /// should be returned.
    /// If absent, all \[Instance][google.spanner.admin.instance.v1.Instance\] fields are returned.
    #[prost(message, optional, tag = "2")]
    pub field_mask: ::core::option::Option<::prost_types::FieldMask>,
}
/// The request for \[CreateInstance][google.spanner.admin.instance.v1.InstanceAdmin.CreateInstance\].
#[derive(Clone, PartialEq, Eq, ::prost::Message)]
pub struct CreateInstanceRequest {
    /// Required. The name of the project in which to create the instance. Values
    /// are of the form `projects/<project>`.
    #[prost(string, tag = "1")]
    pub parent: ::prost::alloc::string::String,
    /// Required. The ID of the instance to create.  Valid identifiers are of the
    /// form `\[a-z][-a-z0-9]*[a-z0-9\]` and must be between 2 and 64 characters in
    /// length.
    #[prost(string, tag = "2")]
    pub instance_id: ::prost::alloc::string::String,
    /// Required. The instance to create.  The name may be omitted, but if
    /// specified must be `<parent>/instances/<instance_id>`.
    #[prost(message, optional, tag = "3")]
    pub instance: ::core::option::Option<Instance>,
}
/// The request for \[ListInstances][google.spanner.admin.instance.v1.InstanceAdmin.ListInstances\].
#[derive(Clone, PartialEq, Eq, ::prost::Message)]
pub struct ListInstancesRequest {
    /// Required. The name of the project for which a list of instances is
    /// requested. Values are of the form `projects/<project>`.
    #[prost(string, tag = "1")]
    pub parent: ::prost::alloc::string::String,
    /// Number of instances to be returned in the response. If 0 or less, defaults
    /// to the server's maximum allowed page size.
    #[prost(int32, tag = "2")]
    pub page_size: i32,
    /// If non-empty, `page_token` should contain a
    /// \[next_page_token][google.spanner.admin.instance.v1.ListInstancesResponse.next_page_token\] from a
    /// previous \[ListInstancesResponse][google.spanner.admin.instance.v1.ListInstancesResponse\].
    #[prost(string, tag = "3")]
    pub page_token: ::prost::alloc::string::String,
    /// An expression for filtering the results of the request. Filter rules are
    /// case insensitive. The fields eligible for filtering are:
    ///
    ///   * `name`
    ///   * `display_name`
    ///   * `labels.key` where key is the name of a label
    ///
    /// Some examples of using filters are:
    ///
    ///   * `name:*` --> The instance has a name.
    ///   * `name:Howl` --> The instance's name contains the string "howl".
    ///   * `name:HOWL` --> Equivalent to above.
    ///   * `NAME:howl` --> Equivalent to above.
    ///   * `labels.env:*` --> The instance has the label "env".
    ///   * `labels.env:dev` --> The instance has the label "env" and the value of
    ///                        the label contains the string "dev".
    ///   * `name:howl labels.env:dev` --> The instance's name contains "howl" and
    ///                                  it has the label "env" with its value
    ///                                  containing "dev".
    #[prost(string, tag = "4")]
    pub filter: ::prost::alloc::string::String,
}
/// The response for \[ListInstances][google.spanner.admin.instance.v1.InstanceAdmin.ListInstances\].
#[derive(Clone, PartialEq, Eq, ::prost::Message)]
pub struct ListInstancesResponse {
    /// The list of requested instances.
    #[prost(message, repeated, tag = "1")]
    pub instances: ::prost::alloc::vec::Vec<Instance>,
    /// `next_page_token` can be sent in a subsequent
    /// \[ListInstances][google.spanner.admin.instance.v1.InstanceAdmin.ListInstances\] call to fetch more
    /// of the matching instances.
    #[prost(string, tag = "2")]
    pub next_page_token: ::prost::alloc::string::String,
}
/// The request for \[UpdateInstance][google.spanner.admin.instance.v1.InstanceAdmin.UpdateInstance\].
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateInstanceRequest {
    /// Required. The instance to update, which must always include the instance
    /// name.  Otherwise, only fields mentioned in \[field_mask][google.spanner.admin.instance.v1.UpdateInstanceRequest.field_mask\] need be included.
    #[prost(message, optional, tag = "1")]
    pub instance: ::core::option::Option<Instance>,
    /// Required. A mask specifying which fields in \[Instance][google.spanner.admin.instance.v1.Instance\] should be updated.
    /// The field mask must always be specified; this prevents any future fields in
    /// \[Instance][google.spanner.admin.instance.v1.Instance\] from being erased accidentally by clients that do not know
    /// about them.
    #[prost(message, optional, tag = "2")]
    pub field_mask: ::core::option::Option<::prost_types::FieldMask>,
}
/// The request for \[DeleteInstance][google.spanner.admin.instance.v1.InstanceAdmin.DeleteInstance\].
#[derive(Clone, PartialEq, Eq, ::prost::Message)]
pub struct DeleteInstanceRequest {
    /// Required. The name of the instance to be deleted. Values are of the form
    /// `projects/<project>/instances/<instance>`
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
/// Metadata type for the operation returned by
/// \[CreateInstance][google.spanner.admin.instance.v1.InstanceAdmin.CreateInstance\].
#[derive(Clone, PartialEq, Eq, ::prost::Message)]
pub struct CreateInstanceMetadata {
    /// The instance being created.
    #[prost(message, optional, tag = "1")]
    pub instance: ::core::option::Option<Instance>,
    /// The time at which the
    /// \[CreateInstance][google.spanner.admin.instance.v1.InstanceAdmin.CreateInstance\] request was
    /// received.
    #[prost(message, optional, tag = "2")]
    pub start_time: ::core::option::Option<::prost_types::Timestamp>,
    /// The time at which this operation was cancelled. If set, this operation is
    /// in the process of undoing itself (which is guaranteed to succeed) and
    /// cannot be cancelled again.
    #[prost(message, optional, tag = "3")]
    pub cancel_time: ::core::option::Option<::prost_types::Timestamp>,
    /// The time at which this operation failed or was completed successfully.
    #[prost(message, optional, tag = "4")]
    pub end_time: ::core::option::Option<::prost_types::Timestamp>,
}
/// Metadata type for the operation returned by
/// \[UpdateInstance][google.spanner.admin.instance.v1.InstanceAdmin.UpdateInstance\].
#[derive(Clone, PartialEq, Eq, ::prost::Message)]
pub struct UpdateInstanceMetadata {
    /// The desired end state of the update.
    #[prost(message, optional, tag = "1")]
    pub instance: ::core::option::Option<Instance>,
    /// The time at which \[UpdateInstance][google.spanner.admin.instance.v1.InstanceAdmin.UpdateInstance\]
    /// request was received.
    #[prost(message, optional, tag = "2")]
    pub start_time: ::core::option::Option<::prost_types::Timestamp>,
    /// The time at which this operation was cancelled. If set, this operation is
    /// in the process of undoing itself (which is guaranteed to succeed) and
    /// cannot be cancelled again.
    #[prost(message, optional, tag = "3")]
    pub cancel_time: ::core::option::Option<::prost_types::Timestamp>,
    /// The time at which this operation failed or was completed successfully.
    #[prost(message, optional, tag = "4")]
    pub end_time: ::core::option::Option<::prost_types::Timestamp>,
}
/// Generated client implementations.
pub mod instance_admin_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Cloud Spanner Instance Admin API
    ///
    /// The Cloud Spanner Instance Admin API can be used to create, delete,
    /// modify and list instances. Instances are dedicated Cloud Spanner serving
    /// and storage resources to be used by Cloud Spanner databases.
    ///
    /// Each instance has a "configuration", which dictates where the
    /// serving resources for the Cloud Spanner instance are located (e.g.,
    /// US-central, Europe). Configurations are created by Google based on
    /// resource availability.
    ///
    /// Cloud Spanner billing is based on the instances that exist and their
    /// sizes. After an instance exists, there are no additional
    /// per-database or per-operation charges for use of the instance
    /// (though there may be additional network bandwidth charges).
    /// Instances offer isolation: problems with databases in one instance
    /// will not affect other instances. However, within an instance
    /// databases can affect each other. For example, if one database in an
    /// instance receives a lot of requests and consumes most of the
    /// instance resources, fewer resources are available for other
    /// databases in that instance, and their performance may suffer.
    #[derive(Debug, Clone)]
    pub struct InstanceAdminClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl InstanceAdminClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> InstanceAdminClient<T>
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
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> InstanceAdminClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<<T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody>,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error: Into<StdError> + Send + Sync,
        {
            InstanceAdminClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with `gzip`.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_gzip(mut self) -> Self {
            self.inner = self.inner.send_gzip();
            self
        }
        /// Enable decompressing responses with `gzip`.
        #[must_use]
        pub fn accept_gzip(mut self) -> Self {
            self.inner = self.inner.accept_gzip();
            self
        }
        /// Lists the supported instance configurations for a given project.
        pub async fn list_instance_configs(
            &mut self,
            request: impl tonic::IntoRequest<super::ListInstanceConfigsRequest>,
        ) -> Result<tonic::Response<super::ListInstanceConfigsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.admin.instance.v1.InstanceAdmin/ListInstanceConfigs",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Gets information about a particular instance configuration.
        pub async fn get_instance_config(
            &mut self,
            request: impl tonic::IntoRequest<super::GetInstanceConfigRequest>,
        ) -> Result<tonic::Response<super::InstanceConfig>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.admin.instance.v1.InstanceAdmin/GetInstanceConfig",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Lists all instances in the given project.
        pub async fn list_instances(
            &mut self,
            request: impl tonic::IntoRequest<super::ListInstancesRequest>,
        ) -> Result<tonic::Response<super::ListInstancesResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/google.spanner.admin.instance.v1.InstanceAdmin/ListInstances");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Gets information about a particular instance.
        pub async fn get_instance(
            &mut self,
            request: impl tonic::IntoRequest<super::GetInstanceRequest>,
        ) -> Result<tonic::Response<super::Instance>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/google.spanner.admin.instance.v1.InstanceAdmin/GetInstance");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Creates an instance and begins preparing it to begin serving. The
        /// returned [long-running operation][google.longrunning.Operation]
        /// can be used to track the progress of preparing the new
        /// instance. The instance name is assigned by the caller. If the
        /// named instance already exists, `CreateInstance` returns
        /// `ALREADY_EXISTS`.
        ///
        /// Immediately upon completion of this request:
        ///
        ///   * The instance is readable via the API, with all requested attributes
        ///     but no allocated resources. Its state is `CREATING`.
        ///
        /// Until completion of the returned operation:
        ///
        ///   * Cancelling the operation renders the instance immediately unreadable
        ///     via the API.
        ///   * The instance can be deleted.
        ///   * All other attempts to modify the instance are rejected.
        ///
        /// Upon completion of the returned operation:
        ///
        ///   * Billing for all successfully-allocated resources begins (some types
        ///     may have lower than the requested levels).
        ///   * Databases can be created in the instance.
        ///   * The instance's allocated resource levels are readable via the API.
        ///   * The instance's state becomes `READY`.
        ///
        /// The returned [long-running operation][google.longrunning.Operation] will
        /// have a name of the format `<instance_name>/operations/<operation_id>` and
        /// can be used to track creation of the instance.  The
        /// [metadata][google.longrunning.Operation.metadata] field type is
        /// [CreateInstanceMetadata][google.spanner.admin.instance.v1.CreateInstanceMetadata].
        /// The [response][google.longrunning.Operation.response] field type is
        /// [Instance][google.spanner.admin.instance.v1.Instance], if successful.
        pub async fn create_instance(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateInstanceRequest>,
        ) -> Result<tonic::Response<super::super::super::super::super::longrunning::Operation>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/google.spanner.admin.instance.v1.InstanceAdmin/CreateInstance");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Updates an instance, and begins allocating or releasing resources
        /// as requested. The returned [long-running
        /// operation][google.longrunning.Operation] can be used to track the
        /// progress of updating the instance. If the named instance does not
        /// exist, returns `NOT_FOUND`.
        ///
        /// Immediately upon completion of this request:
        ///
        ///   * For resource types for which a decrease in the instance's allocation
        ///     has been requested, billing is based on the newly-requested level.
        ///
        /// Until completion of the returned operation:
        ///
        ///   * Cancelling the operation sets its metadata's
        ///     [cancel_time][google.spanner.admin.instance.v1.UpdateInstanceMetadata.cancel_time], and begins
        ///     restoring resources to their pre-request values. The operation
        ///     is guaranteed to succeed at undoing all resource changes,
        ///     after which point it terminates with a `CANCELLED` status.
        ///   * All other attempts to modify the instance are rejected.
        ///   * Reading the instance via the API continues to give the pre-request
        ///     resource levels.
        ///
        /// Upon completion of the returned operation:
        ///
        ///   * Billing begins for all successfully-allocated resources (some types
        ///     may have lower than the requested levels).
        ///   * All newly-reserved resources are available for serving the instance's
        ///     tables.
        ///   * The instance's new resource levels are readable via the API.
        ///
        /// The returned [long-running operation][google.longrunning.Operation] will
        /// have a name of the format `<instance_name>/operations/<operation_id>` and
        /// can be used to track the instance modification.  The
        /// [metadata][google.longrunning.Operation.metadata] field type is
        /// [UpdateInstanceMetadata][google.spanner.admin.instance.v1.UpdateInstanceMetadata].
        /// The [response][google.longrunning.Operation.response] field type is
        /// [Instance][google.spanner.admin.instance.v1.Instance], if successful.
        ///
        /// Authorization requires `spanner.instances.update` permission on
        /// the resource [name][google.spanner.admin.instance.v1.Instance.name].
        pub async fn update_instance(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateInstanceRequest>,
        ) -> Result<tonic::Response<super::super::super::super::super::longrunning::Operation>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/google.spanner.admin.instance.v1.InstanceAdmin/UpdateInstance");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Deletes an instance.
        ///
        /// Immediately upon completion of the request:
        ///
        ///   * Billing ceases for all of the instance's reserved resources.
        ///
        /// Soon afterward:
        ///
        ///   * The instance and *all of its databases* immediately and
        ///     irrevocably disappear from the API. All data in the databases
        ///     is permanently deleted.
        pub async fn delete_instance(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteInstanceRequest>,
        ) -> Result<tonic::Response<()>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/google.spanner.admin.instance.v1.InstanceAdmin/DeleteInstance");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Sets the access control policy on an instance resource. Replaces any
        /// existing policy.
        ///
        /// Authorization requires `spanner.instances.setIamPolicy` on
        /// [resource][google.iam.v1.SetIamPolicyRequest.resource].
        pub async fn set_iam_policy(
            &mut self,
            request: impl tonic::IntoRequest<super::super::super::super::super::iam::v1::SetIamPolicyRequest>,
        ) -> Result<tonic::Response<super::super::super::super::super::iam::v1::Policy>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/google.spanner.admin.instance.v1.InstanceAdmin/SetIamPolicy");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Gets the access control policy for an instance resource. Returns an empty
        /// policy if an instance exists but does not have a policy set.
        ///
        /// Authorization requires `spanner.instances.getIamPolicy` on
        /// [resource][google.iam.v1.GetIamPolicyRequest.resource].
        pub async fn get_iam_policy(
            &mut self,
            request: impl tonic::IntoRequest<super::super::super::super::super::iam::v1::GetIamPolicyRequest>,
        ) -> Result<tonic::Response<super::super::super::super::super::iam::v1::Policy>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/google.spanner.admin.instance.v1.InstanceAdmin/GetIamPolicy");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Returns permissions that the caller has on the specified instance resource.
        ///
        /// Attempting this RPC on a non-existent Cloud Spanner instance resource will
        /// result in a NOT_FOUND error if the user has `spanner.instances.list`
        /// permission on the containing Google Cloud Project. Otherwise returns an
        /// empty set of permissions.
        pub async fn test_iam_permissions(
            &mut self,
            request: impl tonic::IntoRequest<super::super::super::super::super::iam::v1::TestIamPermissionsRequest>,
        ) -> Result<
            tonic::Response<super::super::super::super::super::iam::v1::TestIamPermissionsResponse>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.admin.instance.v1.InstanceAdmin/TestIamPermissions",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
