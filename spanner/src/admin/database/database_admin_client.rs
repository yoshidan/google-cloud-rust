use crate::apiv1::conn_pool::{AUDIENCE, SPANNER};

use google_cloud_gax::call_option::{Backoff, BackoffRetrySettings, BackoffRetryer};
use google_cloud_gax::invoke::invoke_reuse;
use google_cloud_gax::util::create_request;
use google_cloud_googleapis::iam::v1::{
    GetIamPolicyRequest, Policy, SetIamPolicyRequest, TestIamPermissionsRequest,
    TestIamPermissionsResponse,
};
use google_cloud_googleapis::longrunning::Operation as InternalOperation;
use google_cloud_googleapis::spanner::admin::database::v1::database_admin_client::DatabaseAdminClient as InternalDatabaseAdminClient;
use google_cloud_googleapis::spanner::admin::database::v1::{
    Backup, CreateBackupRequest, CreateDatabaseRequest, Database, DeleteBackupRequest,
    DropDatabaseRequest, GetBackupRequest, GetDatabaseDdlRequest, GetDatabaseDdlResponse,
    GetDatabaseRequest, ListBackupOperationsRequest, ListBackupsRequest,
    ListDatabaseOperationsRequest, ListDatabasesRequest, RestoreDatabaseRequest,
    UpdateBackupRequest, UpdateDatabaseDdlRequest,
};
use google_cloud_googleapis::{Code, Status};

use crate::admin::SCOPES;
use google_cloud_grpc::conn::{ConnectionManager, Error, TokenSource};
use google_cloud_longrunning::autogen::operations_client::OperationsClient;
use google_cloud_longrunning::longrunning::Operation;
use tonic::transport::Channel;
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
pub struct DatabaseAdminClient {
    inner: InternalDatabaseAdminClient<Channel>,
    lro_client: OperationsClient,
    token_source: TokenSource,
}

impl DatabaseAdminClient {
    pub fn new(
        inner: InternalDatabaseAdminClient<Channel>,
        lro_client: OperationsClient,
        token_source: TokenSource,
    ) -> Self {
        Self {
            inner,
            lro_client,
            token_source,
        }
    }

    pub async fn default() -> Result<Self, Error> {
        let emulator_host = match std::env::var("SPANNER_EMULATOR_HOST") {
            Ok(s) => Some(s),
            Err(_) => None,
        };
        let conn_pool =
            ConnectionManager::new(1, SPANNER, AUDIENCE, Some(&SCOPES), emulator_host).await?;
        let (conn, token_source) = conn_pool.conn();
        let lro_client = OperationsClient::new(conn, token_source).await?;
        let (conn, token_source) = conn_pool.conn();
        Ok(Self::new(
            InternalDatabaseAdminClient::new(conn),
            lro_client,
            token_source,
        ))
    }

    /// merge call setting
    fn get_call_setting(call_setting: Option<BackoffRetrySettings>) -> BackoffRetrySettings {
        match call_setting {
            Some(s) => s,
            None => default_setting(),
        }
    }

    /// list_databases lists Cloud Spanner databases.
    pub async fn list_databases(
        &mut self,
        mut req: ListDatabasesRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Vec<Database>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let parent = &req.parent;
        let token = self.token_source.token().await?;
        let mut all_databases = vec![];
        //eager loading
        loop {
            let response = invoke_reuse(
                |database_admin_client| async {
                    let request = create_request(format!("parent={}", parent), &token, req.clone());
                    database_admin_client
                        .list_databases(request)
                        .await
                        .map_err(|e| (Status::from(e), database_admin_client))
                        .map(|d| d.into_inner())
                },
                &mut self.inner,
                &mut setting,
            )
            .await?;
            if response.databases.is_empty() {
                return Ok(all_databases);
            }
            all_databases.extend(response.databases.into_iter());
            req.page_token = response.next_page_token;
        }
    }

    /// create_database creates a new Cloud Spanner database and starts to prepare it for serving.
    /// The returned [long-running operation][google.longrunning.Operation] will
    /// have a name of the format <database_name>/operations/<operation_id> and
    /// can be used to track preparation of the database. The metadata field type is CreateDatabaseMetadata.
    /// The response field type is Database, if successful.
    pub async fn create_database(
        &mut self,
        req: CreateDatabaseRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Operation, Status> {
        let mut setting = Self::get_call_setting(opt);
        let parent = &req.parent;
        let token = self.token_source.token().await?;
        return invoke_reuse(
            |database_admin_client| async {
                let request = create_request(format!("parent={}", parent), &token, req.clone());
                database_admin_client
                    .create_database(request)
                    .await
                    .map_err(|e| (e.into(), database_admin_client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await
        .map(|d| Operation::new(self.lro_client.clone(), d.into_inner()));
    }

    /// get_database gets the state of a Cloud Spanner database.
    pub async fn get_database(
        &mut self,
        req: GetDatabaseRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<Database>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let name = &req.name;
        let token = self.token_source.token().await?;
        return invoke_reuse(
            |database_admin_client| async {
                let request = create_request(format!("name={}", name), &token, req.clone());
                database_admin_client
                    .get_database(request)
                    .await
                    .map_err(|e| (e.into(), database_admin_client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// update_database_ddl updates the schema of a Cloud Spanner database by
    /// creating/altering/dropping tables, columns, indexes, etc. The returned
    /// [long-running operation][google.longrunning.Operation] will have a name of
    /// the format <database_name>/operations/<operation_id> and can be used to
    /// track execution of the schema change(s). The
    /// metadata field type is
    /// UpdateDatabaseDdlMetadata.
    /// The operation has no response.
    pub async fn update_database_ddl(
        &mut self,
        req: UpdateDatabaseDdlRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Operation, Status> {
        let mut setting = Self::get_call_setting(opt);
        let database = &req.database;
        let token = self.token_source.token().await?;
        return invoke_reuse(
            |database_admin_client| async {
                let request = create_request(format!("database={}", database), &token, req.clone());
                database_admin_client
                    .update_database_ddl(request)
                    .await
                    .map_err(|e| (e.into(), database_admin_client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await
        .map(|d| Operation::new(self.lro_client.clone(), d.into_inner()));
    }

    /// drop_database drops (aka deletes) a Cloud Spanner database.
    /// Completed backups for the database will be retained according to their
    /// expire_time.
    pub async fn drop_database(
        &mut self,
        req: DropDatabaseRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<()>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let database = &req.database;
        let token = self.token_source.token().await?;
        return invoke_reuse(
            |database_admin_client| async {
                let request = create_request(format!("database={}", database), &token, req.clone());
                database_admin_client
                    .drop_database(request)
                    .await
                    .map_err(|e| (e.into(), database_admin_client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// get_database_ddl returns the schema of a Cloud Spanner database as a list of formatted
    /// DDL statements. This method does not show pending schema updates, those may
    /// be queried using the Operations API.
    pub async fn get_database_ddl(
        &mut self,
        req: GetDatabaseDdlRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<GetDatabaseDdlResponse>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let database = &req.database;
        let token = self.token_source.token().await?;
        return invoke_reuse(
            |database_admin_client| async {
                let request = create_request(format!("database={}", database), &token, req.clone());
                database_admin_client
                    .get_database_ddl(request)
                    .await
                    .map_err(|e| (e.into(), database_admin_client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// set_iam_policy sets the access control policy on a database or backup resource.
    /// Replaces any existing policy.
    ///
    /// Authorization requires spanner.databases.setIamPolicy
    /// permission on resource.
    /// For backups, authorization requires spanner.backups.setIamPolicy
    /// permission on resource.
    pub async fn set_iam_policy(
        &mut self,
        req: SetIamPolicyRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<Policy>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let resource = &req.resource;
        let token = self.token_source.token().await?;
        return invoke_reuse(
            |database_admin_client| async {
                let request = create_request(format!("resource={}", resource), &token, req.clone());
                database_admin_client
                    .set_iam_policy(request)
                    .await
                    .map_err(|e| (e.into(), database_admin_client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// get_iam_policy gets the access control policy for a database or backup resource.
    /// Returns an empty policy if a database or backup exists but does not have a
    /// policy set.
    ///
    /// Authorization requires spanner.databases.getIamPolicy permission on
    /// resource.
    /// For backups, authorization requires spanner.backups.getIamPolicy
    /// permission on resource.
    pub async fn get_iam_policy(
        &mut self,
        req: GetIamPolicyRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<Policy>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let resource = &req.resource;
        let token = self.token_source.token().await?;
        return invoke_reuse(
            |database_admin_client| async {
                let request = create_request(format!("resource={}", resource), &token, req.clone());
                database_admin_client
                    .get_iam_policy(request)
                    .await
                    .map_err(|e| (e.into(), database_admin_client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// TestIamPermissions returns permissions that the caller has on the specified database or backup
    /// resource.
    ///
    /// Attempting this RPC on a non-existent Cloud Spanner database will
    /// result in a NOT_FOUND error if the user has
    /// spanner.databases.list permission on the containing Cloud
    /// Spanner instance. Otherwise returns an empty set of permissions.
    /// Calling this method on a backup that does not exist will
    /// result in a NOT_FOUND error if the user has
    /// spanner.backups.list permission on the containing instance
    pub async fn test_iam_permissions(
        &mut self,
        req: TestIamPermissionsRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<TestIamPermissionsResponse>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let resource = &req.resource;
        let token = self.token_source.token().await?;
        return invoke_reuse(
            |database_admin_client| async {
                let request = create_request(format!("resource={}", resource), &token, req.clone());
                database_admin_client
                    .test_iam_permissions(request)
                    .await
                    .map_err(|e| (e.into(), database_admin_client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// create_backup starts creating a new Cloud Spanner Backup.
    /// The returned backup [long-running operation][google.longrunning.Operation]
    /// will have a name of the format
    /// projects/<project>/instances/<instance>/backups/<backup>/operations/<operation_id>
    /// and can be used to track creation of the backup. The
    /// metadata field type is
    /// CreateBackupMetadata.
    /// The response field type is
    /// Backup, if successful.
    /// Cancelling the returned operation will stop the creation and delete the
    /// backup. There can be only one pending backup creation per database. Backup
    /// creation of different databases can run concurrently.
    pub async fn create_backup(
        &mut self,
        req: CreateBackupRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Operation, Status> {
        let mut setting = Self::get_call_setting(opt);
        let parent = &req.parent;
        let token = self.token_source.token().await?;
        return invoke_reuse(
            |database_admin_client| async {
                let request = create_request(format!("parent={}", parent), &token, req.clone());
                database_admin_client
                    .create_backup(request)
                    .await
                    .map_err(|e| (e.into(), database_admin_client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await
        .map(|d| Operation::new(self.lro_client.clone(), d.into_inner()));
    }

    /// get_backup gets metadata on a pending or completed Backup.
    pub async fn get_backup(
        &mut self,
        req: GetBackupRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<Backup>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let name = &req.name;
        let token = self.token_source.token().await?;
        return invoke_reuse(
            |database_admin_client| async {
                let request = create_request(format!("name={}", name), &token, req.clone());
                database_admin_client
                    .get_backup(request)
                    .await
                    .map_err(|e| (e.into(), database_admin_client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// update_backup updates a pending or completed Backup.
    pub async fn update_backup(
        &mut self,
        req: UpdateBackupRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<Backup>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let name = &req.backup.as_ref().unwrap().name;
        let token = self.token_source.token().await?;
        return invoke_reuse(
            |database_admin_client| async {
                let request = create_request(format!("backup.name={}", name), &token, req.clone());
                database_admin_client
                    .update_backup(request)
                    .await
                    .map_err(|e| (e.into(), database_admin_client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// delete_backup deletes a pending or completed Backup.
    pub async fn delete_backup(
        &mut self,
        req: DeleteBackupRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<()>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let name = &req.name;
        let token = self.token_source.token().await?;
        return invoke_reuse(
            |database_admin_client| async {
                let request = create_request(format!("name={}", name), &token, req.clone());
                database_admin_client
                    .delete_backup(request)
                    .await
                    .map_err(|e| (e.into(), database_admin_client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// list_backups lists completed and pending backups.
    /// Backups returned are ordered by create_time in descending order,
    /// starting from the most recent create_time.
    pub async fn list_backups(
        &mut self,
        mut req: ListBackupsRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Vec<Backup>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let parent = &req.parent;
        let token = self.token_source.token().await?;
        let mut all_backups = vec![];
        //eager loading
        loop {
            let response = invoke_reuse(
                |client| async {
                    let request = create_request(format!("parent={}", parent), &token, req.clone());
                    client
                        .list_backups(request)
                        .await
                        .map_err(|e| (Status::from(e), client))
                        .map(|d| d.into_inner())
                },
                &mut self.inner,
                &mut setting,
            )
            .await?;
            if response.backups.is_empty() {
                return Ok(all_backups);
            }
            all_backups.extend(response.backups.into_iter());
            req.page_token = response.next_page_token;
        }
    }

    /// restore_database create a new database by restoring from a completed backup. The new
    /// database must be in the same project and in an instance with the same
    /// instance configuration as the instance containing
    /// the backup. The returned database [long-running
    /// operation][google.longrunning.Operation] has a name of the format
    /// projects/<project>/instances/<instance>/databases/<database>/operations/<operation_id>,
    /// and can be used to track the progress of the operation, and to cancel it.
    /// The metadata field type is
    /// RestoreDatabaseMetadata.
    /// The response type
    /// is Database, if
    /// successful. Cancelling the returned operation will stop the restore and
    /// delete the database.
    /// There can be only one database being restored into an instance at a time.
    /// Once the restore operation completes, a new restore operation can be
    /// initiated, without waiting for the optimize operation associated with the
    /// first restore to complete.
    pub async fn restore_database(
        &mut self,
        req: RestoreDatabaseRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Operation, Status> {
        let mut setting = Self::get_call_setting(opt);
        let parent = &req.parent;
        let token = self.token_source.token().await?;
        return invoke_reuse(
            |database_admin_client| async {
                let request = create_request(format!("parent={}", parent), &token, req.clone());
                database_admin_client
                    .restore_database(request)
                    .await
                    .map_err(|e| (e.into(), database_admin_client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await
        .map(|d| Operation::new(self.lro_client.clone(), d.into_inner()));
    }

    /// list_backup_operations lists the backup [long-running operations][google.longrunning.Operation] in
    /// the given instance. A backup operation has a name of the form
    /// projects/<project>/instances/<instance>/backups/<backup>/operations/<operation>.
    /// The long-running operation
    /// metadata field type
    /// metadata.type_url describes the type of the metadata. Operations returned
    /// include those that have completed/failed/canceled within the last 7 days,
    /// and pending operations. Operations returned are ordered by
    /// operation.metadata.value.progress.start_time in descending order starting
    /// from the most recently started operation.
    pub async fn list_backup_operations(
        &mut self,
        mut req: ListBackupOperationsRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Vec<InternalOperation>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let parent = &req.parent;
        let token = self.token_source.token().await?;
        let mut all_operations = vec![];
        //eager loading
        loop {
            let response = invoke_reuse(
                |client| async {
                    let request = create_request(format!("parent={}", parent), &token, req.clone());
                    client
                        .list_backup_operations(request)
                        .await
                        .map_err(|e| (Status::from(e), client))
                        .map(|d| d.into_inner())
                },
                &mut self.inner,
                &mut setting,
            )
            .await?;
            if response.operations.is_empty() {
                return Ok(all_operations);
            }
            all_operations.extend(response.operations.into_iter());
            req.page_token = response.next_page_token;
        }
    }

    /// list_database_operations lists database [longrunning-operations][google.longrunning.Operation].
    /// A database operation has a name of the form
    /// projects/<project>/instances/<instance>/databases/<database>/operations/<operation>.
    /// The long-running operation
    /// metadata field type
    /// metadata.type_url describes the type of the metadata. Operations returned
    /// include those that have completed/failed/canceled within the last 7 days,
    /// and pending operations.
    pub async fn list_database_operations(
        &mut self,
        mut req: ListDatabaseOperationsRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Vec<InternalOperation>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let parent = &req.parent;
        let token = self.token_source.token().await?;
        let mut all_operations = vec![];
        //eager loading
        loop {
            let response = invoke_reuse(
                |client| async {
                    let request = create_request(format!("parent={}", parent), &token, req.clone());
                    client
                        .list_database_operations(request)
                        .await
                        .map_err(|e| (Status::from(e), client))
                        .map(|d| d.into_inner())
                },
                &mut self.inner,
                &mut setting,
            )
            .await?;
            if response.operations.is_empty() {
                return Ok(all_operations);
            }
            all_operations.extend(response.operations.into_iter());
            req.page_token = response.next_page_token;
        }
    }
}
