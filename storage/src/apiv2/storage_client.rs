use async_stream;
use google_cloud_gax::cancel::CancellationToken;
use google_cloud_gax::conn::Channel;
use std::sync::Arc;
use std::time::Duration;

use crate::apiv2::conn_pool::ConnectionManager;
use google_cloud_gax::create_request;
use google_cloud_gax::grpc::{Code, IntoRequest, IntoStreamingRequest, Response, Status, Streaming};
use google_cloud_gax::retry::{invoke, invoke_fn, RetrySetting};
use google_cloud_googleapis::iam::v1::Policy;
use google_cloud_googleapis::iam::v1::{
    GetIamPolicyRequest, SetIamPolicyRequest, TestIamPermissionsRequest, TestIamPermissionsResponse,
};
use google_cloud_googleapis::storage::v2 as internal;
use google_cloud_googleapis::storage::v2::storage_client::StorageClient as InternalStorageClient;
use google_cloud_googleapis::storage::v2::{
    Bucket, ComposeObjectRequest, CreateBucketRequest, CreateHmacKeyRequest, CreateHmacKeyResponse,
    CreateNotificationRequest, DeleteBucketRequest, DeleteHmacKeyRequest, DeleteNotificationRequest,
    DeleteObjectRequest, GetBucketRequest, GetHmacKeyRequest, GetNotificationRequest, GetObjectRequest,
    GetServiceAccountRequest, HmacKeyMetadata, ListBucketsRequest, ListHmacKeysRequest, ListNotificationsRequest,
    ListObjectsRequest, LockBucketRetentionPolicyRequest, Notification, Object, QueryWriteStatusRequest,
    QueryWriteStatusResponse, ReadObjectRequest, ReadObjectResponse, RewriteObjectRequest, RewriteResponse,
    ServiceAccount, StartResumableWriteRequest, StartResumableWriteResponse, UpdateBucketRequest, UpdateHmacKeyRequest,
    UpdateObjectRequest, WriteObjectRequest, WriteObjectResponse,
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
    cm: Arc<ConnectionManager>,
}

impl StorageClient {
    /// create new storage client
    pub fn new(cm: ConnectionManager) -> Self {
        Self { cm: Arc::new(cm) }
    }

    fn client(&self) -> InternalStorageClient<Channel> {
        InternalStorageClient::new(self.cm.conn())
    }

    pub async fn delete_bucket(
        &self,
        req: DeleteBucketRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<()>, Status> {
        let retry = retry.unwrap_or(default_setting());
        let action = || async {
            let mut client = self.client();
            let request = req.clone().into_request();
            client.delete_bucket(request).await.map_err(|e| e.into())
        };
        invoke(cancel, Some(retry), action).await
    }

    pub async fn get_bucket(
        &self,
        req: GetBucketRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Bucket>, Status> {
        let retry = retry.unwrap_or(default_setting());
        let action = || async {
            let mut client = self.client();
            let request = req.clone().into_request();
            client.get_bucket(request).await.map_err(|e| e.into())
        };
        invoke(cancel, Some(retry), action).await
    }

    pub async fn create_bucket(
        &self,
        req: CreateBucketRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Bucket>, Status> {
        let retry = retry.unwrap_or(default_setting());
        let action = || async {
            let mut client = self.client();
            client
                .create_bucket(req.clone().into_request())
                .await
                .map_err(|e| e.into())
        };
        invoke(cancel, Some(retry), action).await
    }

    /// list_sessions lists all sessions in a given database.
    pub async fn list_bucket(
        &self,
        mut req: ListBucketsRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Vec<Bucket>, Status> {
        let setting = retry.unwrap_or(default_setting());
        let mut all = vec![];
        //eager loading
        loop {
            let action = || async {
                let mut client = self.client();
                client
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
        &self,
        req: LockBucketRetentionPolicyRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Bucket>, Status> {
        let retry = retry.unwrap_or(default_setting());
        let action = || async {
            let mut client = self.client();
            client
                .lock_bucket_retention_policy(req.clone().into_request())
                .await
                .map_err(|e| e.into())
        };
        invoke(cancel, Some(retry), action).await
    }

    pub async fn get_iam_policy(
        &self,
        req: GetIamPolicyRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Policy>, Status> {
        let retry = retry.unwrap_or(default_setting());
        let action = || async {
            let mut client = self.client();
            client
                .get_iam_policy(req.clone().into_request())
                .await
                .map_err(|e| e.into())
        };
        invoke(cancel, Some(retry), action).await
    }

    pub async fn set_iam_policy(
        &self,
        req: SetIamPolicyRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Policy>, Status> {
        let retry = retry.unwrap_or(default_setting());
        let action = || async {
            let mut client = self.client();
            client
                .set_iam_policy(req.clone().into_request())
                .await
                .map_err(|e| e.into())
        };
        invoke(cancel, Some(retry), action).await
    }

    pub async fn test_iam_permissions(
        &self,
        req: TestIamPermissionsRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<TestIamPermissionsResponse>, Status> {
        let retry = retry.unwrap_or(default_setting());
        let action = || async {
            let mut client = self.client();
            client
                .test_iam_permissions(req.clone().into_request())
                .await
                .map_err(|e| e.into())
        };
        invoke(cancel, Some(retry), action).await
    }

    pub async fn update_bucket(
        &self,
        req: UpdateBucketRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Bucket>, Status> {
        let retry = retry.unwrap_or(default_setting());
        let action = || async {
            let mut client = self.client();
            client
                .update_bucket(req.clone().into_request())
                .await
                .map_err(|e| e.into())
        };
        invoke(cancel, Some(retry), action).await
    }

    pub async fn delete_notification(
        &self,
        req: DeleteNotificationRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<()>, Status> {
        let retry = retry.unwrap_or(default_setting());
        let action = || async {
            let mut client = self.client();
            client
                .delete_notification(req.clone().into_request())
                .await
                .map_err(|e| e.into())
        };
        invoke(cancel, Some(retry), action).await
    }

    /// BeginTransaction begins a new transaction. This step can often be skipped:
    /// Read, ExecuteSql and
    /// Commit can begin a new transaction as a
    /// side-effect.
    pub async fn get_notification(
        &self,
        req: GetNotificationRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Notification>, Status> {
        let retry = retry.unwrap_or(default_setting());
        let action = || async {
            let mut client = self.client();
            client
                .get_notification(req.clone().into_request())
                .await
                .map_err(|e| e.into())
        };
        invoke(cancel, Some(retry), action).await
    }

    pub async fn create_notification(
        &self,
        req: CreateNotificationRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Notification>, Status> {
        let retry = retry.unwrap_or(default_setting());
        let action = || async {
            let mut client = self.client();
            client
                .create_notification(req.clone().into_request())
                .await
                .map_err(|e| e.into())
        };
        invoke(cancel, Some(retry), action).await
    }

    pub async fn list_notifications(
        &self,
        mut req: ListNotificationsRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Vec<Notification>, Status> {
        let setting = retry.unwrap_or(default_setting());
        let mut all = vec![];
        //eager loading
        loop {
            let action = || async {
                let mut client = self.client();
                client
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
        &self,
        req: ComposeObjectRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Object>, Status> {
        let retry = retry.unwrap_or(default_setting());
        let action = || async {
            let mut client = self.client();
            client
                .compose_object(req.clone().into_request())
                .await
                .map_err(|e| e.into())
        };
        invoke(cancel, Some(retry), action).await
    }

    pub async fn delete_object(
        &self,
        req: DeleteObjectRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<()>, Status> {
        let retry = retry.unwrap_or(default_setting());
        let action = || async {
            let mut client = self.client();
            client
                .delete_object(req.clone().into_request())
                .await
                .map_err(|e| e.into())
        };
        invoke(cancel, Some(retry), action).await
    }

    pub async fn get_object(
        &self,
        req: GetObjectRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Object>, Status> {
        let retry = retry.unwrap_or(default_setting());
        let action = || async {
            let mut client = self.client();
            client
                .get_object(req.clone().into_request())
                .await
                .map_err(|e| e.into())
        };
        invoke(cancel, Some(retry), action).await
    }

    pub async fn read_object(
        &self,
        req: ReadObjectRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Streaming<ReadObjectResponse>>, Status> {
        let retry = retry.unwrap_or(default_setting());
        let action = || async {
            let mut client = self.client();
            client
                .read_object(req.clone().into_request())
                .await
                .map_err(|e| e.into())
        };
        invoke(cancel, Some(retry), action).await
    }

    pub async fn update_object(
        &self,
        req: UpdateObjectRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Object>, Status> {
        let retry = retry.unwrap_or(default_setting());
        let action = || async {
            let mut client = self.client();
            client
                .update_object(req.clone().into_request())
                .await
                .map_err(|e| e.into())
        };
        invoke(cancel, Some(retry), action).await
    }

    pub async fn write_object(
        &self,
        req: WriteObjectRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<WriteObjectResponse>, Status> {
        let retry = retry.unwrap_or(default_setting());
        let action = || async {
            let mut client = self.client();
            let base_req = req.clone();
            let request = Box::pin(async_stream::stream! {
                yield base_req.clone();
            });
            let v = request.into_streaming_request();
            client.write_object(v).await.map_err(|e| e.into())
        };
        invoke(cancel, Some(retry), action).await
    }

    pub async fn list_objects(
        &self,
        mut req: ListObjectsRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Vec<Object>, Status> {
        let setting = retry.unwrap_or(default_setting());
        let mut all = vec![];
        //eager loading
        loop {
            let action = || async {
                let mut client = self.client();
                client
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
        &self,
        req: RewriteObjectRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<RewriteResponse>, Status> {
        let retry = retry.unwrap_or(default_setting());
        let action = || async {
            let mut client = self.client();
            client
                .rewrite_object(req.clone().into_request())
                .await
                .map_err(|e| e.into())
        };
        invoke(cancel, Some(retry), action).await
    }

    pub async fn start_resumable_write(
        &self,
        req: StartResumableWriteRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<StartResumableWriteResponse>, Status> {
        let retry = retry.unwrap_or(default_setting());
        let action = || async {
            let mut client = self.client();
            client
                .start_resumable_write(req.clone().into_request())
                .await
                .map_err(|e| e.into())
        };
        invoke(cancel, Some(retry), action).await
    }

    pub async fn query_write_status(
        &self,
        req: QueryWriteStatusRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<QueryWriteStatusResponse>, Status> {
        let retry = retry.unwrap_or(default_setting());
        let action = || async {
            let mut client = self.client();
            client
                .query_write_status(req.clone().into_request())
                .await
                .map_err(|e| e.into())
        };
        invoke(cancel, Some(retry), action).await
    }

    pub async fn get_service_account(
        &self,
        req: GetServiceAccountRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<ServiceAccount>, Status> {
        let retry = retry.unwrap_or(default_setting());
        let action = || async {
            let mut client = self.client();
            client
                .get_service_account(req.clone().into_request())
                .await
                .map_err(|e| e.into())
        };
        invoke(cancel, Some(retry), action).await
    }

    pub async fn create_hmac_key(
        &self,
        req: CreateHmacKeyRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<CreateHmacKeyResponse>, Status> {
        let retry = retry.unwrap_or(default_setting());
        let action = || async {
            let mut client = self.client();
            client
                .create_hmac_key(req.clone().into_request())
                .await
                .map_err(|e| e.into())
        };
        invoke(cancel, Some(retry), action).await
    }

    pub async fn delete_hmac_key(
        &self,
        req: DeleteHmacKeyRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<()>, Status> {
        let retry = retry.unwrap_or(default_setting());
        let action = || async {
            let mut client = self.client();
            client
                .delete_hmac_key(req.clone().into_request())
                .await
                .map_err(|e| e.into())
        };
        invoke(cancel, Some(retry), action).await
    }

    pub async fn get_hmac_key(
        &self,
        req: GetHmacKeyRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<HmacKeyMetadata>, Status> {
        let retry = retry.unwrap_or(default_setting());
        let action = || async {
            let mut client = self.client();
            client
                .get_hmac_key(req.clone().into_request())
                .await
                .map_err(|e| e.into())
        };
        invoke(cancel, Some(retry), action).await
    }

    pub async fn list_hmac_keys(
        &self,
        mut req: ListHmacKeysRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Vec<HmacKeyMetadata>, Status> {
        let setting = retry.unwrap_or(default_setting());
        let mut all = vec![];
        //eager loading
        loop {
            let action = || async {
                let mut client = self.client();
                client
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
        &self,
        req: UpdateHmacKeyRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<HmacKeyMetadata>, Status> {
        let retry = retry.unwrap_or(default_setting());
        let action = || async {
            let mut client = self.client();
            client
                .update_hmac_key(req.clone().into_request())
                .await
                .map_err(|e| e.into())
        };
        invoke(cancel, Some(retry), action).await
    }
}
