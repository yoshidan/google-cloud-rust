use google_cloud_gax::cancel::CancellationToken;
use google_cloud_gax::conn::Channel;
use google_cloud_gax::create_request;
use google_cloud_gax::grpc::{Response, Status};
use google_cloud_gax::retry::{invoke, RetrySetting};
use google_cloud_googleapis::iam::v1::{
    GetIamPolicyRequest, Policy, SetIamPolicyRequest, TestIamPermissionsRequest, TestIamPermissionsResponse,
};
use google_cloud_googleapis::spanner::admin::instance::v1::instance_admin_client::InstanceAdminClient as InternalInstanceAdminClient;
use google_cloud_googleapis::spanner::admin::instance::v1::{
    CreateInstanceRequest, DeleteInstanceRequest, GetInstanceConfigRequest, GetInstanceRequest, Instance,
    InstanceConfig, ListInstanceConfigsRequest, ListInstancesRequest, UpdateInstanceRequest,
};
use google_cloud_longrunning::autogen::operations_client::OperationsClient;
use google_cloud_longrunning::longrunning::Operation;

use crate::admin::default_retry_setting;

#[derive(Clone)]
pub struct InstanceAdminClient {
    inner: InternalInstanceAdminClient<Channel>,
    lro_client: OperationsClient,
}

impl InstanceAdminClient {
    pub fn new(inner: InternalInstanceAdminClient<Channel>, lro_client: OperationsClient) -> Self {
        Self { inner, lro_client }
    }

    /// list_instance_configs lists the supported instance configurations for a given project.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn list_instance_configs(
        &self,
        mut req: ListInstanceConfigsRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Vec<InstanceConfig>, Status> {
        let retry = Some(retry.unwrap_or_else(default_retry_setting));
        let parent = &req.parent;
        let mut all = vec![];
        //eager loading
        loop {
            let action = || async {
                let request = create_request(format!("parent={parent}"), req.clone());
                self.inner
                    .clone()
                    .list_instance_configs(request)
                    .await
                    .map(|d| d.into_inner())
            };
            let response = invoke(cancel.clone(), retry.clone(), action).await?;
            all.extend(response.instance_configs.into_iter());
            if response.next_page_token.is_empty() {
                return Ok(all);
            }
            req.page_token = response.next_page_token;
        }
    }

    /// get_instance_config gets information about a particular instance configuration.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_instance_config(
        &self,
        req: GetInstanceConfigRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<InstanceConfig, Status> {
        let retry = Some(retry.unwrap_or_else(default_retry_setting));
        let name = &req.name;
        let action = || async {
            let request = create_request(format!("name={name}"), req.clone());
            self.inner
                .clone()
                .get_instance_config(request)
                .await
                .map(|d| d.into_inner())
        };
        invoke(cancel, retry, action).await
    }

    /// list_instances lists all instances in the given project.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn list_instances(
        &self,
        mut req: ListInstancesRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Vec<Instance>, Status> {
        let retry = Some(retry.unwrap_or_else(default_retry_setting));
        let parent = &req.parent;
        let mut all = vec![];
        //eager loading
        loop {
            let action = || async {
                let request = create_request(format!("parent={parent}"), req.clone());
                self.inner.clone().list_instances(request).await.map(|d| d.into_inner())
            };
            let response = invoke(cancel.clone(), retry.clone(), action).await?;
            all.extend(response.instances.into_iter());
            if response.next_page_token.is_empty() {
                return Ok(all);
            }
            req.page_token = response.next_page_token;
        }
    }

    /// gets information about a particular instance.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_instance(
        &self,
        req: GetInstanceRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Instance>, Status> {
        let retry = Some(retry.unwrap_or_else(default_retry_setting));
        let name = &req.name;
        let action = || async {
            let request = create_request(format!("name={name}"), req.clone());
            self.inner.clone().get_instance(request).await
        };
        invoke(cancel, retry, action).await
    }

    /// create_instance creates an instance and begins preparing it to begin serving. The
    /// returned [long-running operation][google.longrunning.Operation]
    /// can be used to track the progress of preparing the new
    /// instance. The instance name is assigned by the caller. If the
    /// named instance already exists, CreateInstance returns
    /// ALREADY_EXISTS.
    ///
    /// Immediately upon completion of this request:
    ///
    ///   The instance is readable via the API, with all requested attributes
    ///   but no allocated resources. Its state is CREATING.
    ///
    /// Until completion of the returned operation:
    ///
    ///   Cancelling the operation renders the instance immediately unreadable
    ///   via the API.
    ///
    ///   The instance can be deleted.
    ///
    ///   All other attempts to modify the instance are rejected.
    ///
    /// Upon completion of the returned operation:
    ///
    ///   Billing for all successfully-allocated resources begins (some types
    ///   may have lower than the requested levels).
    ///
    ///   Databases can be created in the instance.
    ///
    ///   The instance’s allocated resource levels are readable via the API.
    ///
    ///   The instance’s state becomes READY.
    ///
    /// The returned [long-running operation][google.longrunning.Operation] will
    /// have a name of the format <instance_name>/operations/<operation_id> and
    /// can be used to track creation of the instance.  The
    /// metadata field type is
    /// CreateInstanceMetadata.
    /// The response field type is
    /// Instance, if successful.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn create_instance(
        &self,
        req: CreateInstanceRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Operation<Instance>, Status> {
        let retry = Some(retry.unwrap_or_else(default_retry_setting));
        let parent = &req.parent;
        let action = || async {
            let request = create_request(format!("parent={parent}"), req.clone());
            self.inner.clone().create_instance(request).await
        };
        invoke(cancel, retry, action)
            .await
            .map(|d| Operation::new(self.lro_client.clone(), d.into_inner()))
    }

    /// update_instance updates an instance, and begins allocating or releasing resources
    /// as requested. The returned [long-running
    /// operation][google.longrunning.Operation] can be used to track the
    /// progress of updating the instance. If the named instance does not
    /// exist, returns NOT_FOUND.
    ///
    /// Immediately upon completion of this request:
    ///
    ///   For resource types for which a decrease in the instance’s allocation
    ///   has been requested, billing is based on the newly-requested level.
    ///
    /// Until completion of the returned operation:
    ///
    ///   Cancelling the operation sets its metadata’s
    ///   cancel_time, and begins
    ///   restoring resources to their pre-request values. The operation
    ///   is guaranteed to succeed at undoing all resource changes,
    ///   after which point it terminates with a CANCELLED status.
    ///
    ///   All other attempts to modify the instance are rejected.
    ///
    ///   Reading the instance via the API continues to give the pre-request
    ///   resource levels.
    ///
    /// Upon completion of the returned operation:
    ///
    ///   Billing begins for all successfully-allocated resources (some types
    ///   may have lower than the requested levels).
    ///
    ///   All newly-reserved resources are available for serving the instance’s
    ///   tables.
    ///
    ///   The instance’s new resource levels are readable via the API.
    ///
    /// The returned [long-running operation][google.longrunning.Operation] will
    /// have a name of the format <instance_name>/operations/<operation_id> and
    /// can be used to track the instance modification.  The
    /// metadata field type is
    /// UpdateInstanceMetadata.
    /// The response field type is
    /// Instance, if successful.
    ///
    /// Authorization requires spanner.instances.update permission on
    /// resource [name][google.spanner.admin.instance.v1.Instance.name (at http://google.spanner.admin.instance.v1.Instance.name)].
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn update_instance(
        &self,
        req: UpdateInstanceRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Operation<Instance>, Status> {
        let retry = Some(retry.unwrap_or_else(default_retry_setting));
        let instance_name = &req.instance.as_ref().unwrap().name;
        let action = || async {
            let request = create_request(format!("instance.name={instance_name}"), req.clone());
            self.inner.clone().update_instance(request).await
        };
        invoke(cancel, retry, action)
            .await
            .map(|d| Operation::new(self.lro_client.clone(), d.into_inner()))
    }

    /// DeleteInstance deletes an instance.
    ///
    /// Immediately upon completion of the request:
    ///
    ///   Billing ceases for all of the instance’s reserved resources.
    ///
    /// Soon afterward:
    ///
    ///   The instance and all of its databases immediately and
    ///   irrevocably disappear from the API. All data in the databases
    ///   is permanently deleted.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn delete_instance(
        &self,
        req: DeleteInstanceRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<()>, Status> {
        let retry = Some(retry.unwrap_or_else(default_retry_setting));
        let name = &req.name;
        let action = || async {
            let request = create_request(format!("name={name}"), req.clone());
            self.inner.clone().delete_instance(request).await
        };
        invoke(cancel, retry, action).await
    }

    /// set_iam_policy sets the access control policy on an instance resource. Replaces any
    /// existing policy.
    ///
    /// Authorization requires spanner.instances.setIamPolicy on resource.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn set_iam_policy(
        &self,
        req: SetIamPolicyRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Policy>, Status> {
        let resource = &req.resource;
        let retry = Some(retry.unwrap_or_else(default_retry_setting));
        let action = || async {
            let request = create_request(format!("resource={resource}"), req.clone());
            self.inner.clone().set_iam_policy(request).await
        };
        invoke(cancel, retry, action).await
    }

    /// get_iam_policy sets the access control policy on an instance resource. Replaces any
    /// existing policy.
    ///
    /// Authorization requires spanner.instances.setIamPolicy on resource.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_iam_policy(
        &self,
        req: GetIamPolicyRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Policy>, Status> {
        let resource = &req.resource;
        let retry = Some(retry.unwrap_or_else(default_retry_setting));
        let action = || async {
            let request = create_request(format!("resource={resource}"), req.clone());
            self.inner.clone().get_iam_policy(request).await
        };
        invoke(cancel, retry, action).await
    }

    /// test_iam_permissions returns permissions that the caller has on the specified instance resource.
    ///
    /// Attempting this RPC on a non-existent Cloud Spanner instance resource will
    /// result in a NOT_FOUND error if the user has spanner.instances.list
    /// permission on the containing Google Cloud Project. Otherwise returns an
    /// empty set of permissions.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn test_iam_permissions(
        &self,
        req: TestIamPermissionsRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<TestIamPermissionsResponse>, Status> {
        let resource = &req.resource;
        let retry = Some(retry.unwrap_or_else(default_retry_setting));
        let action = || async {
            let request = create_request(format!("resource={resource}"), req.clone());
            self.inner.clone().test_iam_permissions(request).await
        };
        invoke(cancel, retry, action).await
    }
}
