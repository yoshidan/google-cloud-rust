use crate::apiv1::conn_pool::{AUDIENCE, SPANNER};

use google_cloud_gax::call_option::{Backoff, BackoffRetrySettings, BackoffRetryer};
use google_cloud_gax::retry::invoke_reuse;
use google_cloud_gax::util::create_request;
use google_cloud_googleapis::iam::v1::{
    GetIamPolicyRequest, Policy, SetIamPolicyRequest, TestIamPermissionsRequest,
    TestIamPermissionsResponse,
};

use google_cloud_googleapis::spanner::admin::instance::v1::instance_admin_client::InstanceAdminClient as InternalInstanceAdminClient;
use google_cloud_googleapis::{Code, Status};

use crate::admin::SCOPES;
use google_cloud_gax::conn::{Channel, ConnectionManager, Error};
use google_cloud_googleapis::spanner::admin::instance::v1::{
    CreateInstanceRequest, DeleteInstanceRequest, GetInstanceConfigRequest, GetInstanceRequest,
    Instance, InstanceConfig, ListInstanceConfigsRequest, ListInstancesRequest,
    UpdateInstanceRequest,
};
use google_cloud_longrunning::autogen::operations_client::OperationsClient;
use google_cloud_longrunning::longrunning::Operation;
use tonic::Response;

fn default_setting() -> BackoffRetrySettings {
    BackoffRetrySettings {
        retryer: BackoffRetryer {
            backoff: Backoff::default(),
            codes: vec![Code::Unavailable, Code::Unknown, Code::DeadlineExceeded],
        },
    }
}

#[derive(Clone)]
pub struct InstanceAdminClient {
    inner: InternalInstanceAdminClient<Channel>,
    lro_client: OperationsClient,
}

impl InstanceAdminClient {
    pub fn new(inner: InternalInstanceAdminClient<Channel>, lro_client: OperationsClient) -> Self {
        Self { inner, lro_client }
    }

    pub async fn default() -> Result<Self, Error> {
        let emulator_host = match std::env::var("SPANNER_EMULATOR_HOST") {
            Ok(s) => Some(s),
            Err(_) => None,
        };
        let conn_pool =
            ConnectionManager::new(1, SPANNER, AUDIENCE, Some(&SCOPES), emulator_host).await?;
        let conn = conn_pool.conn();
        let lro_client = OperationsClient::new(conn).await?;
        let conn = conn_pool.conn();
        Ok(Self::new(
            InternalInstanceAdminClient::new(conn),
            lro_client,
        ))
    }

    /// merge call setting
    fn get_call_setting(call_setting: Option<BackoffRetrySettings>) -> BackoffRetrySettings {
        match call_setting {
            Some(s) => s,
            None => default_setting(),
        }
    }

    /// list_instance_configs lists the supported instance configurations for a given project.
    pub async fn list_instance_configs(
        &mut self,
        mut req: ListInstanceConfigsRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Vec<InstanceConfig>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let parent = &req.parent;
        let mut all = vec![];
        //eager loading
        loop {
            let response = invoke_reuse(
                |client| async {
                    let request = create_request(format!("parent={}", parent), req.clone());
                    client
                        .list_instance_configs(request)
                        .await
                        .map_err(|e| (Status::from(e), client))
                        .map(|d| d.into_inner())
                },
                &mut self.inner,
                &mut setting,
            )
            .await?;
            all.extend(response.instance_configs.into_iter());
            if response.next_page_token.is_empty() {
                return Ok(all);
            }
            req.page_token = response.next_page_token;
        }
    }

    /// get_instance_config gets information about a particular instance configuration.
    pub async fn get_instance_config(
        &mut self,
        req: GetInstanceConfigRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<InstanceConfig>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let name = &req.name;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("name={}", name), req.clone());
                client
                    .get_instance_config(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// list_instances lists all instances in the given project.
    pub async fn list_instances(
        &mut self,
        mut req: ListInstancesRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Vec<Instance>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let parent = &req.parent;
        let mut all = vec![];
        //eager loading
        loop {
            let response = invoke_reuse(
                |client| async {
                    let request = create_request(format!("parent={}", parent), req.clone());
                    client
                        .list_instances(request)
                        .await
                        .map_err(|e| (Status::from(e), client))
                        .map(|d| d.into_inner())
                },
                &mut self.inner,
                &mut setting,
            )
            .await?;
            all.extend(response.instances.into_iter());
            if response.next_page_token.is_empty() {
                return Ok(all);
            }
            req.page_token = response.next_page_token;
        }
    }

    /// gets information about a particular instance.
    pub async fn get_instance(
        &mut self,
        req: GetInstanceRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<Instance>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let name = &req.name;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("name={}", name), req.clone());
                client
                    .get_instance(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
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
    pub async fn create_instance(
        &mut self,
        req: CreateInstanceRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Operation<Instance>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let parent = &req.parent;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("parent={}", parent), req.clone());
                client
                    .create_instance(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await
        .map(|d| Operation::new(self.lro_client.clone(), d.into_inner()));
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
    pub async fn update_instance(
        &mut self,
        req: UpdateInstanceRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Operation<Instance>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let instance_name = &req.instance.as_ref().unwrap().name;
        return invoke_reuse(
            |client| async {
                let request =
                    create_request(format!("instance.name={}", instance_name), req.clone());
                client
                    .update_instance(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await
        .map(|d| Operation::new(self.lro_client.clone(), d.into_inner()));
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
    pub async fn delete_instance(
        &mut self,
        req: DeleteInstanceRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<()>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let name = &req.name;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("parent={}", name), req.clone());
                client
                    .delete_instance(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// set_iam_policy sets the access control policy on an instance resource. Replaces any
    /// existing policy.
    ///
    /// Authorization requires spanner.instances.setIamPolicy on resource.
    pub async fn set_iam_policy(
        &mut self,
        req: SetIamPolicyRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<Policy>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let resource = &req.resource;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("resource={}", resource), req.clone());
                client
                    .set_iam_policy(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// get_iam_policy sets the access control policy on an instance resource. Replaces any
    /// existing policy.
    ///
    /// Authorization requires spanner.instances.setIamPolicy on resource.
    pub async fn get_iam_policy(
        &mut self,
        req: GetIamPolicyRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<Policy>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let resource = &req.resource;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("resource={}", resource), req.clone());
                client
                    .get_iam_policy(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// test_iam_permissions returns permissions that the caller has on the specified instance resource.
    ///
    /// Attempting this RPC on a non-existent Cloud Spanner instance resource will
    /// result in a NOT_FOUND error if the user has spanner.instances.list
    /// permission on the containing Google Cloud Project. Otherwise returns an
    /// empty set of permissions.
    pub async fn test_iam_permissions(
        &mut self,
        req: TestIamPermissionsRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<TestIamPermissionsResponse>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let resource = &req.resource;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("resource={}", resource), req.clone());
                client
                    .test_iam_permissions(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }
}
