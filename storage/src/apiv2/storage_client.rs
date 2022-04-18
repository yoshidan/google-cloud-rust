use google_cloud_gax::cancel::CancellationToken;
use std::time::Duration;

use google_cloud_gax::conn::Channel;
use google_cloud_gax::create_request;
use google_cloud_gax::grpc::{Code, IntoRequest, Response, Status, Streaming};
use google_cloud_gax::retry::{invoke, invoke_fn, RetrySetting};
use google_cloud_googleapis::iam::v1::Policy;
use google_cloud_googleapis::iam::v1::{
    GetIamPolicyRequest, SetIamPolicyRequest, TestIamPermissionsRequest, TestIamPermissionsResponse,
};
use google_cloud_googleapis::spanner::admin::database::v1::UpdateBackupRequest;
use google_cloud_googleapis::storage::v2 as internal;
use google_cloud_googleapis::storage::v2::storage_client::StorageClient as InternalStorageClient;
use google_cloud_googleapis::storage::v2::{
    Bucket, ComposeObjectRequest, CreateBucketRequest, CreateHmacKeyRequest, CreateHmacKeyResponse,
    CreateNotificationRequest, DeleteBucketRequest, DeleteHmacKeyRequest, DeleteNotificationRequest,
    DeleteObjectRequest, GetBucketRequest, GetHmacKeyRequest, GetNotificationRequest, GetObjectRequest,
    GetServiceAccountRequest, HmacKeyMetadata, ListBucketsRequest, ListHmacKeysRequest, ListNotificationsRequest,
    ListObjectsRequest, LockBucketRetentionPolicyRequest, Notification, Object, QueryWriteStatusRequest,
    QueryWriteStatusResponse, ReadObjectRequest, ReadObjectResponse, RewriteResponse, ServiceAccount,
    StartResumableWriteRequest, StartResumableWriteResponse, UpdateHmacKeyRequest, UpdateObjectRequest,
    WriteObjectResponse,
};

fn default_setting() -> RetrySetting {
    RetrySetting {
        from_millis: 50,
        max_delay: Some(Duration::from_secs(10)),
        factor: 1u64,
        take: 20,
        codes: vec![Code::DeadlineExceeded, Code::Unavailable, Code::Unknown],
    }
}

#[derive(Clone)]
pub struct StorageClient {
    inner: InternalStorageClient<Channel>,
}

impl StorageClient {
    /// create new storage client
    pub fn new(inner: InternalStorageClient<Channel>) -> Self {
        Self { inner }
    }

    pub async fn delete_bucket(
        &mut self,
        req: DeleteBucketRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<()>, Status> {
        let setting = retry.unwrap_or(default_setting());
        return invoke_fn(
            cancel,
            Some(setting),
            |client| async {
                let request = req.clone().into_request();
                client.delete_bucket(request).await.map_err(|e| (e.into(), client))
            },
            &mut self.inner,
        )
        .await;
    }

    pub async fn get_bucket(
        &mut self,
        req: GetBucketRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Bucket>, Status> {
        let setting = retry.unwrap_or(default_setting());
        return invoke_fn(
            cancel,
            Some(setting),
            |client| async {
                let request = req.clone().into_request();
                client.get_bucket(request).await.map_err(|e| (e.into(), client))
            },
            &mut self.inner,
        )
        .await;
    }

    pub async fn create_bucket(
        &mut self,
        req: CreateBucketRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Bucket>, Status> {
        let setting = retry.unwrap_or(default_setting());
        return invoke_fn(
            cancel,
            Some(setting),
            |client| async {
                client
                    .create_bucket(req.clone().into_request())
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
        )
        .await;
    }

    /// list_sessions lists all sessions in a given database.
    pub async fn list_bucket(
        &mut self,
        mut req: ListBucketsRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Vec<Bucket>, Status> {
        let setting = retry.unwrap_or(default_setting());
        let mut all = vec![];
        //eager loading
        loop {
            let action = || async {
                self.inner
                    .list_buckets(req.clone().into_request())
                    .await
                    .map_err(|e| Status::from(e))
                    .map(|d| d.into_inner())
            };
            let response = invoke(cancel.clone(), Some(setting.clone()), action).await?;
            all.extend(response.buckets.into_iter());
            if response.next_page_token.is_empty() {
                return Ok(all);
            }
            req.page_token = response.next_page_token;
        }
    }

    pub async fn lock_bucket_retention_policy(
        &mut self,
        req: LockBucketRetentionPolicyRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Bucket>, Status> {
        let setting = retry.unwrap_or(default_setting());
        return invoke_fn(
            cancel,
            Some(setting),
            |client| async {
                client
                    .lock_bucket_retention_policy(req.clone().into_request())
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
        )
        .await;
    }

    pub async fn get_iam_policy(
        &mut self,
        req: GetIamPolicyRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Policy>, Status> {
        let setting = retry.unwrap_or(default_setting());
        return invoke_fn(
            cancel,
            Some(setting),
            |client| async {
                client
                    .get_iam_policy(req.clone().into_request())
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
        )
        .await;
    }

    pub async fn set_iam_policy(
        &mut self,
        req: SetIamPolicyRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Policy>, Status> {
        let setting = retry.unwrap_or(default_setting());
        return invoke_fn(
            cancel,
            Some(setting),
            |client| async {
                client
                    .set_iam_policy(req.clone().into_request())
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
        )
        .await;
    }

    pub async fn test_iam_permissions(
        &mut self,
        req: TestIamPermissionsRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<TestIamPermissionsResponse>, Status> {
        let setting = retry.unwrap_or(default_setting());
        return invoke_fn(
            cancel,
            Some(setting),
            |client| async {
                client
                    .test_iam_permissions(req.clone().into_request())
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
        )
        .await;
    }

    pub async fn update_bucket(
        &mut self,
        req: UpdateBackupRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Bucket>, Status> {
        let setting = retry.unwrap_or(default_setting());
        return invoke_fn(
            cancel,
            Some(setting),
            |client| async {
                client
                    .update_bucket(req.clone().into_request())
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
        )
        .await;
    }

    pub async fn delete_notification(
        &mut self,
        req: DeleteNotificationRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<()>, Status> {
        let setting = retry.unwrap_or(default_setting());
        return invoke_fn(
            cancel,
            Some(setting),
            |client| async {
                client
                    .delete_notification(req.clone().into_request())
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
        )
        .await;
    }

    /// BeginTransaction begins a new transaction. This step can often be skipped:
    /// Read, ExecuteSql and
    /// Commit can begin a new transaction as a
    /// side-effect.
    pub async fn get_notification(
        &mut self,
        req: GetNotificationRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Notification>, Status> {
        let setting = retry.unwrap_or(default_setting());
        return invoke_fn(
            cancel,
            Some(setting),
            |client| async {
                client
                    .get_notification(req.clone().into_request())
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
        )
        .await;
    }

    pub async fn create_notification(
        &mut self,
        req: CreateNotificationRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Notification>, Status> {
        let setting = retry.unwrap_or(default_setting());
        return invoke_fn(
            cancel,
            Some(setting),
            |client| async {
                client
                    .create_notification(req.clone().into_request())
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
        )
        .await;
    }

    pub async fn list_notifications(
        &mut self,
        mut req: ListNotificationsRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Vec<Notification>, Status> {
        let setting = retry.unwrap_or(default_setting());
        let mut all = vec![];
        //eager loading
        loop {
            let action = || async {
                self.inner
                    .list_notifications(req.clone().into_request())
                    .await
                    .map_err(|e| Status::from(e))
                    .map(|d| d.into_inner())
            };
            let response = invoke(cancel.clone(), Some(setting.clone()), action).await?;
            all.extend(response.notifications.into_iter());
            if response.next_page_token.is_empty() {
                return Ok(all);
            }
            req.page_token = response.next_page_token;
        }
    }

    pub async fn compose_object(
        &mut self,
        req: ComposeObjectRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Object>, Status> {
        let setting = retry.unwrap_or(default_setting());
        return invoke_fn(
            cancel,
            Some(setting),
            |client| async {
                client
                    .compose_object(req.clone().into_request())
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
        )
        .await;
    }

    pub async fn delete_object(
        &mut self,
        req: DeleteObjectRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<()>, Status> {
        let setting = retry.unwrap_or(default_setting());
        return invoke_fn(
            cancel,
            Some(setting),
            |client| async {
                client
                    .delete_object(req.clone().into_request())
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
        )
        .await;
    }

    pub async fn get_object(
        &mut self,
        req: GetObjectRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Object>, Status> {
        let setting = retry.unwrap_or(default_setting());
        return invoke_fn(
            cancel,
            Some(setting),
            |client| async {
                client
                    .get_object(req.clone().into_request())
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
        )
        .await;
    }

    pub async fn read_object(
        &mut self,
        req: ReadObjectRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Streaming<ReadObjectResponse>>, Status> {
        let setting = retry.unwrap_or(default_setting());
        return invoke_fn(
            cancel,
            Some(setting),
            |client| async {
                client
                    .read_object(req.clone().into_request())
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
        )
        .await;
    }

    pub async fn update_object(
        &mut self,
        req: UpdateObjectRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Object>, Status> {
        let setting = retry.unwrap_or(default_setting());
        return invoke_fn(
            cancel,
            Some(setting),
            |client| async {
                client
                    .update_object(req.clone().into_request())
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
        )
        .await;
    }

    pub async fn write_object(
        &mut self,
        req: WriterObjectRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<WriteObjectResponse>, Status> {
        let setting = retry.unwrap_or(default_setting());
        return invoke_fn(
            cancel,
            Some(setting),
            |client| async {
                client
                    .write_object(req.clone().into_request())
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
        )
        .await;
    }

    pub async fn list_objects(
        &mut self,
        mut req: ListObjectsRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Vec<Object>, Status> {
        let setting = retry.unwrap_or(default_setting());
        let mut all = vec![];
        //eager loading
        loop {
            let action = || async {
                self.inner
                    .list_objects(req.clone().into_request())
                    .await
                    .map_err(|e| Status::from(e))
                    .map(|d| d.into_inner())
            };
            let response = invoke(cancel.clone(), Some(setting.clone()), action).await?;
            all.extend(response.objects.into_iter());
            if response.next_page_token.is_empty() {
                return Ok(all);
            }
            req.page_token = response.next_page_token;
        }
    }

    pub async fn rewrite_object(
        &mut self,
        req: WriterObjectRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<RewriteResponse>, Status> {
        let setting = retry.unwrap_or(default_setting());
        return invoke_fn(
            cancel,
            Some(setting),
            |client| async {
                client
                    .rewrite_object(req.clone().into_request())
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
        )
        .await;
    }

    pub async fn start_resumable_write(
        &mut self,
        req: StartResumableWriteRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<StartResumableWriteResponse>, Status> {
        let setting = retry.unwrap_or(default_setting());
        return invoke_fn(
            cancel,
            Some(setting),
            |client| async {
                client
                    .start_resumable_write(req.clone().into_request())
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
        )
        .await;
    }

    pub async fn query_write_status(
        &mut self,
        req: QueryWriteStatusRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<QueryWriteStatusResponse>, Status> {
        let setting = retry.unwrap_or(default_setting());
        return invoke_fn(
            cancel,
            Some(setting),
            |client| async {
                client
                    .query_write_status(req.clone().into_request())
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
        )
        .await;
    }

    pub async fn get_service_account(
        &mut self,
        req: GetServiceAccountRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<ServiceAccount>, Status> {
        let setting = retry.unwrap_or(default_setting());
        return invoke_fn(
            cancel,
            Some(setting),
            |client| async {
                client
                    .get_service_account(req.clone().into_request())
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
        )
        .await;
    }

    pub async fn create_hmac_key(
        &mut self,
        req: CreateHmacKeyRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<CreateHmacKeyResponse>, Status> {
        let setting = retry.unwrap_or(default_setting());
        return invoke_fn(
            cancel,
            Some(setting),
            |client| async {
                client
                    .create_hmac_key(req.clone().into_request())
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
        )
        .await;
    }

    pub async fn delete_hmac_key(
        &mut self,
        req: DeleteHmacKeyRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<()>, Status> {
        let setting = retry.unwrap_or(default_setting());
        return invoke_fn(
            cancel,
            Some(setting),
            |client| async {
                client
                    .delete_hmac_key(req.clone().into_request())
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
        )
        .await;
    }

    pub async fn get_hmac_key(
        &mut self,
        req: GetHmacKeyRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<HmacKeyMetadata>, Status> {
        let setting = retry.unwrap_or(default_setting());
        return invoke_fn(
            cancel,
            Some(setting),
            |client| async {
                client
                    .get_hmac_key(req.clone().into_request())
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
        )
        .await;
    }

    pub async fn list_hmac_keys(
        &mut self,
        mut req: ListHmacKeysRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Vec<HmacKeyMetadata>, Status> {
        let setting = retry.unwrap_or(default_setting());
        let mut all = vec![];
        //eager loading
        loop {
            let action = || async {
                self.inner
                    .list_hmac_keys(req.clone().into_request())
                    .await
                    .map_err(|e| Status::from(e))
                    .map(|d| d.into_inner())
            };
            let response = invoke(cancel.clone(), Some(setting.clone()), action).await?;
            all.extend(response.hmac_keys.into_iter());
            if response.next_page_token.is_empty() {
                return Ok(all);
            }
            req.page_token = response.next_page_token;
        }
    }

    pub async fn update_hmac_key(
        &mut self,
        req: UpdateHmacKeyRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<HmacKeyMetadata>, Status> {
        let setting = retry.unwrap_or(default_setting());
        return invoke_fn(
            cancel,
            Some(setting),
            |client| async {
                client
                    .update_hmac_key(req.clone().into_request())
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
        )
        .await;
    }
}
