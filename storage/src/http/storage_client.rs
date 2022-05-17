use crate::http::bucket_access_controls::delete::DeleteBucketAccessControlRequest;
use crate::http::bucket_access_controls::get::GetBucketAccessControlRequest;
use crate::http::bucket_access_controls::insert::InsertBucketAccessControlRequest;
use crate::http::bucket_access_controls::list::{ListBucketAccessControlsRequest, ListBucketAccessControlsResponse};
use crate::http::bucket_access_controls::patch::PatchBucketAccessControlRequest;
use crate::http::bucket_access_controls::BucketAccessControl;
use crate::http::buckets::delete::DeleteBucketRequest;
use crate::http::buckets::get::GetBucketRequest;
use crate::http::buckets::get_iam_policy::GetIamPolicyRequest;
use crate::http::buckets::insert::InsertBucketRequest;
use crate::http::buckets::list::{ListBucketsRequest, ListBucketsResponse};

use crate::http::buckets::patch::PatchBucketRequest;
use crate::http::buckets::set_iam_policy::SetIamPolicyRequest;
use crate::http::buckets::test_iam_permissions::{TestIamPermissionsRequest, TestIamPermissionsResponse};
use crate::http::buckets::{Bucket, Policy};


use crate::http::default_object_access_controls::delete::DeleteDefaultObjectAccessControlRequest;
use crate::http::default_object_access_controls::get::GetDefaultObjectAccessControlRequest;
use crate::http::default_object_access_controls::insert::InsertDefaultObjectAccessControlRequest;
use crate::http::default_object_access_controls::list::{
    ListDefaultObjectAccessControlsRequest, ListDefaultObjectAccessControlsResponse,
};
use crate::http::default_object_access_controls::patch::PatchDefaultObjectAccessControlRequest;
use crate::http::hmac_keys::create::{CreateHmacKeyRequest, CreateHmacKeyResponse};
use crate::http::hmac_keys::delete::DeleteHmacKeyRequest;
use crate::http::hmac_keys::get::GetHmacKeyRequest;
use crate::http::hmac_keys::list::{ListHmacKeysRequest, ListHmacKeysResponse};
use crate::http::hmac_keys::update::UpdateHmacKeyRequest;
use crate::http::hmac_keys::HmacKeyMetadata;
use crate::http::notifications::delete::DeleteNotificationRequest;
use crate::http::notifications::get::GetNotificationRequest;
use crate::http::notifications::insert::InsertNotificationRequest;
use crate::http::notifications::list::{ListNotificationsRequest, ListNotificationsResponse};
use crate::http::notifications::Notification;
use crate::http::object_access_controls::delete::DeleteObjectAccessControlRequest;
use crate::http::object_access_controls::get::GetObjectAccessControlRequest;
use crate::http::object_access_controls::insert::InsertObjectAccessControlRequest;
use crate::http::object_access_controls::list::ListObjectAccessControlsRequest;
use crate::http::object_access_controls::patch::PatchObjectAccessControlRequest;
use crate::http::object_access_controls::ObjectAccessControl;
use crate::http::objects::compose::ComposeObjectRequest;
use crate::http::objects::delete::DeleteObjectRequest;
use crate::http::objects::get::GetObjectRequest;
use crate::http::objects::list::{ListObjectsRequest, ListObjectsResponse};
use crate::http::objects::patch::PatchObjectRequest;
use crate::http::objects::rewrite::{RewriteObjectRequest, RewriteObjectResponse};
use crate::http::objects::upload::UploadObjectRequest;

use crate::http::objects::Object;
use crate::http::{
    bucket_access_controls, buckets, default_object_access_controls, hmac_keys, notifications,
    object_access_controls, objects, CancellationToken, Error,
};
use futures_util::{Stream, StreamExt, TryStream};
use google_cloud_auth::token_source::TokenSource;

use reqwest::{Body, Client, RequestBuilder, Response};



use std::future::Future;



use std::sync::Arc;


pub const SCOPES: [&str; 2] = [
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/devstorage.full_control",
];

#[derive(Clone)]
pub struct StorageClient {
    ts: Arc<dyn TokenSource>,
}

impl StorageClient {
    pub(crate) fn new(ts: Arc<dyn TokenSource>) -> Self {
        Self { ts }
    }

    /// Deletes the bucket.
    pub async fn delete_bucket(
        &self,
        req: &DeleteBucketRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let action = async {
            let builder = buckets::delete::build(&Client::new(), req);
            self.send_get_empty(builder).await
        };
        invoke(cancel, action).await
    }

    /// Inserts the bucket.
    pub async fn insert_bucket(
        &self,
        req: &InsertBucketRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Bucket, Error> {
        let action = async {
            let builder = buckets::insert::build(&Client::new(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Gets the bucket.
    pub async fn get_bucket(&self, req: &GetBucketRequest, cancel: Option<CancellationToken>) -> Result<Bucket, Error> {
        let action = async {
            let builder = buckets::get::build(&Client::new(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Update the bucket.
    pub async fn patch_bucket(
        &self,
        req: &PatchBucketRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Bucket, Error> {
        let action = async {
            let builder = buckets::patch::build(&Client::new(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Lists the bucket.
    pub async fn list_buckets(
        &self,
        req: &ListBucketsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListBucketsResponse, Error> {
        let action = async {
            let builder = buckets::list::build(&Client::new(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Sets the iam policy.
    pub async fn set_iam_policy(
        &self,
        req: &SetIamPolicyRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Policy, Error> {
        let action = async {
            let builder = buckets::set_iam_policy::build(&Client::new(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Gets the iam policy.
    pub async fn get_iam_policy(
        &self,
        req: &GetIamPolicyRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Policy, Error> {
        let action = async {
            let builder = buckets::get_iam_policy::build(&Client::new(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Tests the iam permissions.
    pub async fn test_iam_permissions(
        &self,
        req: &TestIamPermissionsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<TestIamPermissionsResponse, Error> {
        let action = async {
            let builder = buckets::test_iam_permissions::build(&Client::new(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Lists the default object ACL.
    pub async fn list_default_object_access_controls(
        &self,
        req: &ListDefaultObjectAccessControlsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListDefaultObjectAccessControlsResponse, Error> {
        let action = async {
            let builder = default_object_access_controls::list::build(&Client::new(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Gets the default object ACL.
    pub async fn get_default_object_access_control(
        &self,
        req: &GetDefaultObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        let action = async {
            let builder = default_object_access_controls::get::build(&Client::new(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Inserts the default object ACL.
    pub async fn insert_default_object_access_control(
        &self,
        req: &InsertDefaultObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        let action = async {
            let builder = default_object_access_controls::insert::build(&Client::new(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Patchs the default object ACL.
    pub async fn patch_default_object_access_control(
        &self,
        req: &PatchDefaultObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        let action = async {
            let builder = default_object_access_controls::patch::build(&Client::new(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Deletes the default object ACL.
    pub async fn delete_default_object_access_control(
        &self,
        req: &DeleteDefaultObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let action = async {
            let builder = default_object_access_controls::delete::build(&Client::new(), req);
            self.send_get_empty(builder).await
        };
        invoke(cancel, action).await
    }

    /// Lists the bucket ACL.
    pub async fn list_bucket_access_controls(
        &self,
        req: &ListBucketAccessControlsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListBucketAccessControlsResponse, Error> {
        let action = async {
            let builder = bucket_access_controls::list::build(&Client::new(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Gets the bucket ACL.
    pub async fn get_bucket_access_control(
        &self,
        req: &GetBucketAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<BucketAccessControl, Error> {
        let action = async {
            let builder = bucket_access_controls::get::build(&Client::new(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Inserts the default object ACL.
    pub async fn insert_bucket_access_control(
        &self,
        req: &InsertBucketAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<BucketAccessControl, Error> {
        let action = async {
            let builder = bucket_access_controls::insert::build(&Client::new(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Patchs the bucket ACL.
    pub async fn patch_bucket_access_control(
        &self,
        req: &PatchBucketAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<BucketAccessControl, Error> {
        let action = async {
            let builder = bucket_access_controls::patch::build(&Client::new(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Deletes the bucket ACL.
    pub async fn delete_bucket_access_control(
        &self,
        req: &DeleteBucketAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let action = async {
            let builder = bucket_access_controls::delete::build(&Client::new(), req);
            self.send_get_empty(builder).await
        };
        invoke(cancel, action).await
    }

    /// Lists the object ACL.
    pub async fn list_object_access_controls(
        &self,
        req: &ListObjectAccessControlsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListBucketAccessControlsResponse, Error> {
        let action = async {
            let builder = object_access_controls::list::build(&Client::new(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Gets the object ACL.
    pub async fn get_object_access_control(
        &self,
        req: &GetObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        let action = async {
            let builder = object_access_controls::get::build(&Client::new(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Inserts the object ACL.
    pub async fn insert_object_access_control(
        &self,
        req: &InsertObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        let action = async {
            let builder = object_access_controls::insert::build(&Client::new(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Patchs the bucket ACL.
    pub async fn patch_object_access_control(
        &self,
        req: &PatchObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        let action = async {
            let builder = object_access_controls::patch::build(&Client::new(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Deletes the bucket ACL.
    pub async fn delete_object_access_control(
        &self,
        req: &DeleteObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let action = async {
            let builder = object_access_controls::delete::build(&Client::new(), req);
            self.send_get_empty(builder).await
        };
        invoke(cancel, action).await
    }

    /// Lists the notification.
    pub async fn list_notifications(
        &self,
        req: &ListNotificationsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListNotificationsResponse, Error> {
        let action = async {
            let builder = notifications::list::build(&Client::new(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Gets the notification.
    pub async fn get_notification(
        &self,
        req: &GetNotificationRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Notification, Error> {
        let action = async {
            let builder = notifications::get::build(&Client::new(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Inserts the notification.
    pub async fn insert_notification(
        &self,
        req: &InsertNotificationRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Notification, Error> {
        let action = async {
            let builder = notifications::insert::build(&Client::new(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Deletes the notification.
    pub async fn delete_notification(
        &self,
        req: &DeleteNotificationRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let action = async {
            let builder = notifications::delete::build(&Client::new(), req);
            self.send_get_empty(builder).await
        };
        invoke(cancel, action).await
    }

    /// Lists the hmac keys.
    pub async fn list_hmac_keys(
        &self,
        req: &ListHmacKeysRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListHmacKeysResponse, Error> {
        let action = async {
            let builder = hmac_keys::list::build(&Client::new(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Gets the hmac keys.
    pub async fn get_hmac_key(
        &self,
        req: &GetHmacKeyRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<HmacKeyMetadata, Error> {
        let action = async {
            let builder = hmac_keys::get::build(&Client::new(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Creates the hmac key.
    pub async fn create_hmac_key(
        &self,
        req: &CreateHmacKeyRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<CreateHmacKeyResponse, Error> {
        let action = async {
            let builder = hmac_keys::create::build(&Client::new(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Updates the hmac key.
    pub async fn update_hmac_key(
        &self,
        req: &UpdateHmacKeyRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<HmacKeyMetadata, Error> {
        let action = async {
            let builder = hmac_keys::update::build(&Client::new(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Deletes the hmac key.
    pub async fn delete_hmac_key(
        &self,
        req: &DeleteHmacKeyRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let action = async {
            let builder = hmac_keys::delete::build(&Client::new(), req);
            self.send_get_empty(builder).await
        };
        invoke(cancel, action).await
    }

    /// Lists the objects.
    pub async fn list_objects(
        &self,
        req: &ListObjectsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListObjectsResponse, Error> {
        let action = async {
            let builder = objects::list::build(&Client::new(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Gets the object.
    pub async fn get_object(&self, req: &GetObjectRequest, cancel: Option<CancellationToken>) -> Result<Object, Error> {
        let action = async {
            let builder = objects::get::build(&Client::new(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Download the object.
    pub async fn download_object(
        &self,
        req: &GetObjectRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Vec<u8>, Error> {
        let action = async {
            let builder = objects::download::build(&Client::new(), req);
            let request = self.with_headers(builder).await?;
            let response = request.send().await?;
            if response.status().is_success() {
                Ok(response.bytes().await?.to_vec())
            } else {
                Err(map_error(response).await)
            }
        };
        invoke(cancel, action).await
    }

    /// Download the object.
    pub async fn download_streamed_object(
        &self,
        req: &GetObjectRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<impl Stream<Item = reqwest::Result<bytes::Bytes>>, Error> {
        let action = async {
            let builder = objects::download::build(&Client::new(), req);
            let request = self.with_headers(builder).await?;
            let response = request.send().await?;
            if response.status().is_success() {
                Ok(response.bytes_stream())
            } else {
                Err(map_error(response).await)
            }
        };
        invoke(cancel, action).await
    }

    /// Uploads the object.
    pub async fn upload_object(
        &self,
        req: &UploadObjectRequest,
        data: Vec<u8>,
        content_type: &str,
        cancel: Option<CancellationToken>,
    ) -> Result<Object, Error> {
        let action = async {
            let builder = objects::upload::build(&Client::new(), req, data.len(), content_type, data);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Uploads the streamed object.
    pub async fn upload_streamed_object<S>(
        &self,
        req: &UploadObjectRequest,
        data: S,
        content_type: &str,
        content_length: usize,
        cancel: Option<CancellationToken>,
    ) -> Result<Object, Error>
    where
        S: TryStream + Send + Sync + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        bytes::Bytes: From<S::Ok>,
    {
        let action = async {
            let builder =
                objects::upload::build(&Client::new(), req, content_length, content_type, Body::wrap_stream(data));
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Updates the object.
    pub async fn patch_object(
        &self,
        req: &PatchObjectRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Object, Error> {
        let action = async {
            let builder = objects::patch::build(&Client::new(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Deletes the object.
    pub async fn delete_object(
        &self,
        req: &DeleteObjectRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let action = async {
            let builder = objects::delete::build(&Client::new(), req);
            self.send_get_empty(builder).await
        };
        invoke(cancel, action).await
    }

    /// Rewrites the object.
    pub async fn rewrite_object(
        &self,
        req: &RewriteObjectRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<RewriteObjectResponse, Error> {
        let action = async {
            let builder = objects::rewrite::build(&Client::new(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Composes the object.
    pub async fn compose_object(
        &self,
        req: &ComposeObjectRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Object, Error> {
        let action = async {
            let builder = objects::compose::build(&Client::new(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    async fn with_headers(&self, builder: RequestBuilder) -> Result<RequestBuilder, Error> {
        let token = self.ts.token().await?;
        Ok(builder
            .header("X-Goog-Api-Client", "rust")
            .header(reqwest::header::USER_AGENT, "google-cloud-storage")
            .header(reqwest::header::AUTHORIZATION, token.value()))
    }

    async fn send<T: for<'de> serde::Deserialize<'de>>(&self, builder: RequestBuilder) -> Result<T, Error> {
        let request = self.with_headers(builder).await?;
        let response = request.send().await?;
        if response.status().is_success() {
            let text = response.text().await?;
            tracing::trace!("response={}", text);
            Ok(serde_json::from_str(&text).unwrap())
        } else {
            Err(map_error(response).await)
        }
    }

    async fn send_get_empty(&self, builder: RequestBuilder) -> Result<(), Error> {
        let builder = self.with_headers(builder).await?;
        let response = builder.send().await?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(map_error(response).await)
        }
    }
}

async fn map_error(r: Response) -> Error {
    let status = r.status().as_u16();
    let text = match r.text().await {
        Ok(text) => text,
        Err(e) => format!("{}", e),
    };
    Error::Response(status, text)
}

async fn invoke<S>(
    cancel: Option<CancellationToken>,
    action: impl Future<Output = Result<S, Error>>,
) -> Result<S, Error> {
    match cancel {
        Some(cancel) => {
            tokio::select! {
                _ = cancel.cancelled() => Err(Error::Cancelled),
                v = action => v
            }
        }
        None => action.await,
    }
}

#[cfg(test)]
mod test {
    use crate::http::bucket_access_controls::delete::DeleteBucketAccessControlRequest;
    use crate::http::bucket_access_controls::get::GetBucketAccessControlRequest;
    use crate::http::bucket_access_controls::insert::{
        BucketAccessControlCreationConfig, InsertBucketAccessControlRequest,
    };
    use crate::http::bucket_access_controls::list::ListBucketAccessControlsRequest;
    use crate::http::bucket_access_controls::{BucketACLRole};
    use crate::http::buckets::delete::DeleteBucketRequest;
    use crate::http::buckets::get::GetBucketRequest;
    use crate::http::buckets::get_iam_policy::GetIamPolicyRequest;
    use crate::http::buckets::insert::{
        BucketCreationConfig, InsertBucketParam, InsertBucketRequest,
    };
    use crate::http::buckets::list::ListBucketsRequest;
    use crate::http::buckets::patch::{BucketPatchConfig, PatchBucketRequest};
    use crate::http::buckets::set_iam_policy::SetIamPolicyRequest;
    use crate::http::buckets::test_iam_permissions::TestIamPermissionsRequest;
    use crate::http::buckets::{Binding};
    use crate::http::default_object_access_controls::delete::DeleteDefaultObjectAccessControlRequest;
    use crate::http::default_object_access_controls::get::GetDefaultObjectAccessControlRequest;
    use crate::http::default_object_access_controls::insert::InsertDefaultObjectAccessControlRequest;
    use crate::http::default_object_access_controls::list::ListDefaultObjectAccessControlsRequest;
    use crate::http::hmac_keys::create::CreateHmacKeyRequest;
    use crate::http::hmac_keys::delete::DeleteHmacKeyRequest;
    use crate::http::hmac_keys::get::GetHmacKeyRequest;
    use crate::http::hmac_keys::list::ListHmacKeysRequest;
    use crate::http::hmac_keys::update::UpdateHmacKeyRequest;
    use crate::http::hmac_keys::HmacKeyMetadata;
    use crate::http::notifications::delete::DeleteNotificationRequest;
    use crate::http::notifications::get::GetNotificationRequest;
    use crate::http::notifications::insert::{InsertNotificationRequest, NotificationCreationConfig};
    use crate::http::notifications::list::ListNotificationsRequest;
    use crate::http::object_access_controls::delete::DeleteObjectAccessControlRequest;
    use crate::http::object_access_controls::get::GetObjectAccessControlRequest;
    use crate::http::object_access_controls::insert::{
        InsertObjectAccessControlRequest, ObjectAccessControlCreationConfig,
    };
    use crate::http::object_access_controls::list::ListObjectAccessControlsRequest;
    use crate::http::object_access_controls::{ObjectACLRole};
    use crate::http::objects::compose::{ComposeObjectRequest, ComposingTargets};
    use crate::http::objects::delete::DeleteObjectRequest;
    use crate::http::objects::get::GetObjectRequest;
    use crate::http::objects::list::ListObjectsRequest;
    use crate::http::objects::rewrite::RewriteObjectRequest;
    use crate::http::objects::upload::UploadObjectRequest;
    
    use crate::http::objects::SourceObjects;
    use crate::http::storage_client::{StorageClient, SCOPES};
    use bytes::Buf;
    use futures_util::StreamExt;
    use google_cloud_auth::{create_token_source, Config};
    use serde_json::de::Read;
    use serial_test::serial;
    
    use std::sync::Arc;

    const PROJECT: &str = "atl-dev1";

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt::try_init();
    }

    async fn client() -> StorageClient {
        let ts = create_token_source(Config {
            audience: None,
            scopes: Some(&SCOPES),
        })
        .await
        .unwrap();
        return StorageClient::new(Arc::from(ts));
    }

    #[tokio::test]
    #[serial]
    pub async fn list_buckets() {
        let client = client().await;
        let buckets = client
            .list_buckets(
                &ListBucketsRequest {
                    project: PROJECT.to_string(),
                    max_results: None,
                    page_token: None,
                    prefix: Some("rust-iam-test".to_string()),
                    projection: None,
                },
                None,
            )
            .await
            .unwrap();
        assert_eq!(1, buckets.items.len());
    }

    #[tokio::test]
    #[serial]
    pub async fn crud_bucket() {
        let client = client().await;
        let name = format!("rust-test-insert-{}", chrono::Utc::now().timestamp());
        let bucket = client
            .insert_bucket(
                &InsertBucketRequest {
                    name,
                    param: InsertBucketParam {
                        project: PROJECT.to_string(),
                        ..Default::default()
                    },
                    bucket: BucketCreationConfig {
                        location: "ASIA-NORTHEAST1".to_string(),
                        storage_class: Some("STANDARD".to_string()),
                        ..Default::default()
                    },
                },
                None,
            )
            .await
            .unwrap();

        let found = client
            .get_bucket(
                &GetBucketRequest {
                    bucket: bucket.name.to_string(),
                    ..Default::default()
                },
                None,
            )
            .await
            .unwrap();

        assert_eq!(found.location.as_str(), "ASIA-NORTHEAST1");

        let patched = client
            .patch_bucket(
                &PatchBucketRequest {
                    bucket: bucket.name.to_string(),
                    metadata: Some(BucketPatchConfig {
                        default_object_acl: Some(vec![ObjectAccessControlCreationConfig {
                            entity: "allAuthenticatedUsers".to_string(),
                            role: ObjectACLRole::READER,
                        }]),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                None,
            )
            .await
            .unwrap();

        let default_object_acl = patched.default_object_acl.unwrap();
        assert_eq!(default_object_acl.len(), 1);
        assert_eq!(default_object_acl[0].entity.as_str(), "allAuthenticatedUsers");
        assert_eq!(default_object_acl[0].role, ObjectACLRole::READER);
        assert_eq!(found.storage_class.as_str(), patched.storage_class.as_str());
        assert_eq!(found.location.as_str(), patched.location.as_str());

        client
            .delete_bucket(
                &DeleteBucketRequest {
                    bucket: bucket.name,
                    param: Default::default(),
                },
                None,
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn set_get_test_iam() {
        let bucket_name = "rust-iam-test";
        let client = client().await;
        let mut policy = client
            .get_iam_policy(
                &GetIamPolicyRequest {
                    resource: bucket_name.to_string(),
                    options_requested_policy_version: None,
                },
                None,
            )
            .await
            .unwrap();
        policy.bindings.push(Binding {
            role: "roles/storage.objectViewer".to_string(),
            members: vec!["allAuthenticatedUsers".to_string()],
            condition: None,
        });

        let mut result = client
            .set_iam_policy(
                &SetIamPolicyRequest {
                    resource: bucket_name.to_string(),
                    policy,
                },
                None,
            )
            .await
            .unwrap();
        assert_eq!(result.bindings.len(), 5);
        assert_eq!(result.bindings.pop().unwrap().role, "roles/storage.objectViewer");

        let permissions = client
            .test_iam_permissions(
                &TestIamPermissionsRequest {
                    resource: bucket_name.to_string(),
                    permissions: vec!["storage.buckets.get".to_string()],
                },
                None,
            )
            .await
            .unwrap();
        assert_eq!(permissions.permissions[0], "storage.buckets.get");
    }

    #[tokio::test]
    #[serial]
    pub async fn crud_default_object_controls() {
        let bucket_name = "rust-default-object-acl-test";
        let client = client().await;

        let _ = client
            .delete_default_object_access_control(
                &DeleteDefaultObjectAccessControlRequest {
                    bucket: bucket_name.to_string(),
                    entity: "allAuthenticatedUsers".to_string(),
                },
                None,
            )
            .await
            .unwrap();

        let _post = client
            .insert_default_object_access_control(
                &InsertDefaultObjectAccessControlRequest {
                    bucket: bucket_name.to_string(),
                    object_access_control: ObjectAccessControlCreationConfig {
                        entity: "allAuthenticatedUsers".to_string(),
                        role: ObjectACLRole::READER,
                    },
                },
                None,
            )
            .await
            .unwrap();

        let found = client
            .get_default_object_access_control(
                &GetDefaultObjectAccessControlRequest {
                    bucket: bucket_name.to_string(),
                    entity: "allAuthenticatedUsers".to_string(),
                },
                None,
            )
            .await
            .unwrap();
        assert_eq!(found.entity, "allAuthenticatedUsers");
        assert_eq!(found.role, ObjectACLRole::READER);

        let acls = client
            .list_default_object_access_controls(
                &ListDefaultObjectAccessControlsRequest {
                    bucket: bucket_name.to_string(),
                    ..Default::default()
                },
                None,
            )
            .await
            .unwrap();
        assert!(acls.items.is_some());
        assert_eq!(1, acls.items.unwrap().len());
    }

    #[tokio::test]
    #[serial]
    pub async fn crud_bucket_access_controls() {
        let bucket_name = "rust-bucket-acl-test";
        let client = client().await;

        let _post = client
            .insert_bucket_access_control(
                &InsertBucketAccessControlRequest {
                    bucket: bucket_name.to_string(),
                    acl: BucketAccessControlCreationConfig {
                        entity: "allAuthenticatedUsers".to_string(),
                        role: BucketACLRole::READER,
                    },
                },
                None,
            )
            .await
            .unwrap();

        let found = client
            .get_bucket_access_control(
                &GetBucketAccessControlRequest {
                    bucket: bucket_name.to_string(),
                    entity: "allAuthenticatedUsers".to_string(),
                },
                None,
            )
            .await
            .unwrap();
        assert_eq!(found.entity, "allAuthenticatedUsers");
        assert_eq!(found.role, BucketACLRole::READER);

        let acls = client
            .list_bucket_access_controls(
                &ListBucketAccessControlsRequest {
                    bucket: bucket_name.to_string(),
                },
                None,
            )
            .await
            .unwrap();
        assert_eq!(5, acls.items.len());

        let _ = client
            .delete_bucket_access_control(
                &DeleteBucketAccessControlRequest {
                    bucket: bucket_name.to_string(),
                    entity: "allAuthenticatedUsers".to_string(),
                },
                None,
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    #[serial]
    pub async fn crud_object_access_controls() {
        let bucket_name = "rust-default-object-acl-test";
        let object_name = "test.txt";
        let client = client().await;

        let _post = client
            .insert_object_access_control(
                &InsertObjectAccessControlRequest {
                    bucket: bucket_name.to_string(),
                    object: object_name.to_string(),
                    generation: None,
                    acl: ObjectAccessControlCreationConfig {
                        entity: "allAuthenticatedUsers".to_string(),
                        role: ObjectACLRole::READER,
                    },
                },
                None,
            )
            .await
            .unwrap();

        let found = client
            .get_object_access_control(
                &GetObjectAccessControlRequest {
                    bucket: bucket_name.to_string(),
                    entity: "allAuthenticatedUsers".to_string(),
                    object: object_name.to_string(),
                    generation: None,
                },
                None,
            )
            .await
            .unwrap();
        assert_eq!(found.entity, "allAuthenticatedUsers");
        assert_eq!(found.role, ObjectACLRole::READER);

        let acls = client
            .list_object_access_controls(
                &ListObjectAccessControlsRequest {
                    bucket: bucket_name.to_string(),
                    object: object_name.to_string(),
                    generation: None,
                },
                None,
            )
            .await
            .unwrap();
        assert_eq!(2, acls.items.len());

        let _ = client
            .delete_object_access_control(
                &DeleteObjectAccessControlRequest {
                    bucket: bucket_name.to_string(),
                    object: object_name.to_string(),
                    entity: "allAuthenticatedUsers".to_string(),
                    generation: None,
                },
                None,
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    #[serial]
    pub async fn crud_notification() {
        let bucket_name = "rust-bucket-test";
        let client = client().await;

        let notifications = client
            .list_notifications(
                &ListNotificationsRequest {
                    bucket: bucket_name.to_string(),
                },
                None,
            )
            .await
            .unwrap();

        for n in notifications.items.unwrap_or(vec![]) {
            let _ = client
                .delete_notification(
                    &DeleteNotificationRequest {
                        bucket: bucket_name.to_string(),
                        notification: n.id.to_string(),
                    },
                    None,
                )
                .await
                .unwrap();
        }

        let post = client
            .insert_notification(
                &InsertNotificationRequest {
                    bucket: bucket_name.to_string(),
                    notification: NotificationCreationConfig {
                        topic: format!("projects/{}/topics/{}", PROJECT, bucket_name.to_string()),
                        event_types: Some(vec!["OBJECT_METADATA_UPDATE".to_string(), "OBJECT_DELETE".to_string()]),
                        custom_attributes: Default::default(),
                        object_name_prefix: Some("notification-test".to_string()),
                        payload_format: "JSON_API_V1".to_string(),
                    },
                },
                None,
            )
            .await
            .unwrap();

        let found = client
            .get_notification(
                &GetNotificationRequest {
                    bucket: bucket_name.to_string(),
                    notification: post.id.to_string(),
                },
                None,
            )
            .await
            .unwrap();
        assert_eq!(found.id, post.id);
        assert_eq!(found.event_types.unwrap().len(), 2);
    }

    #[tokio::test]
    #[serial]
    pub async fn crud_hmac_key() {
        let _key_name = "rust-hmac-test";
        let client = client().await;

        let post = client
            .create_hmac_key(
                &CreateHmacKeyRequest {
                    project_id: PROJECT.to_string(),
                    service_account_email: format!("spanner@{}.iam.gserviceaccount.com", PROJECT),
                },
                None,
            )
            .await
            .unwrap();

        let found = client
            .get_hmac_key(
                &GetHmacKeyRequest {
                    access_id: post.metadata.access_id.to_string(),
                    project_id: PROJECT.to_string(),
                },
                None,
            )
            .await
            .unwrap();
        assert_eq!(found.id, post.metadata.id);
        assert_eq!(found.state, "ACTIVE");

        let keys = client
            .list_hmac_keys(
                &ListHmacKeysRequest {
                    project_id: PROJECT.to_string(),
                    ..Default::default()
                },
                None,
            )
            .await
            .unwrap();

        for n in keys.items.unwrap_or(vec![]) {
            let result = client
                .update_hmac_key(
                    &UpdateHmacKeyRequest {
                        access_id: n.access_id.to_string(),
                        project_id: n.project_id.to_string(),
                        metadata: HmacKeyMetadata {
                            state: "INACTIVE".to_string(),
                            ..n.clone()
                        },
                    },
                    None,
                )
                .await
                .unwrap();
            assert_eq!(result.state, "INACTIVE");

            let _ = client
                .delete_hmac_key(
                    &DeleteHmacKeyRequest {
                        access_id: n.access_id.to_string(),
                        project_id: n.project_id.to_string(),
                    },
                    None,
                )
                .await
                .unwrap();
        }
    }

    #[tokio::test]
    #[serial]
    pub async fn crud_object() {
        let bucket_name = "rust-object-test";
        let client = client().await;

        let objects = client
            .list_objects(
                &ListObjectsRequest {
                    bucket: bucket_name.to_string(),
                    ..Default::default()
                },
                None,
            )
            .await
            .unwrap()
            .items
            .unwrap_or(vec![]);
        for o in objects {
            client
                .delete_object(
                    &DeleteObjectRequest {
                        bucket: o.bucket.to_string(),
                        object: o.name.to_string(),
                        ..Default::default()
                    },
                    None,
                )
                .await
                .unwrap();
        }

        let uploaded = client
            .upload_object(
                &UploadObjectRequest {
                    bucket: bucket_name.to_string(),
                    name: "test1".to_string(),
                    ..Default::default()
                },
                vec![01],
                "text/plain",
                None,
            )
            .await
            .unwrap();

        let downloaded = client
            .download_object(
                &GetObjectRequest {
                    bucket: uploaded.bucket.to_string(),
                    object: uploaded.name.to_string(),
                    ..Default::default()
                },
                None,
            )
            .await
            .unwrap();
        assert_eq!(downloaded, vec![01]);

        let _rewrited = client
            .rewrite_object(
                &RewriteObjectRequest {
                    destination_bucket: bucket_name.to_string(),
                    destination_object: format!("{}_rewrite", uploaded.name),
                    source_bucket: bucket_name.to_string(),
                    source_object: uploaded.name.to_string(),
                    ..Default::default()
                },
                None,
            )
            .await
            .unwrap();

        let _composed = client
            .compose_object(
                &ComposeObjectRequest {
                    bucket: bucket_name.to_string(),
                    destination_object: format!("{}_composed", uploaded.name),
                    destination_predefined_acl: None,
                    composing_targets: ComposingTargets {
                        destination: None,
                        source_objects: vec![SourceObjects {
                            name: format!("{}_rewrite", uploaded.name),
                            ..Default::default()
                        }],
                    },
                    ..Default::default()
                },
                None,
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    #[serial]
    pub async fn streamed_object() {
        let bucket_name = "rust-object-test";
        let file_name = format!("stream_{}", chrono::Utc::now().timestamp());
        let client = client().await;

        let source = vec!["hello", " ", "world"];
        let size = source.iter().map(|x| x.len()).sum();
        let chunks: Vec<Result<_, ::std::io::Error>> = source.clone().into_iter().map(|x| Ok(x)).collect();
        let stream = futures_util::stream::iter(chunks);
        let uploaded = client
            .upload_streamed_object(
                &UploadObjectRequest {
                    bucket: bucket_name.to_string(),
                    name: file_name.to_string(),
                    predefined_acl: None,
                    ..Default::default()
                },
                stream,
                "application/octet-stream",
                size,
                None,
            )
            .await
            .unwrap();

        let mut downloaded = client
            .download_streamed_object(
                &GetObjectRequest {
                    bucket: uploaded.bucket.to_string(),
                    object: uploaded.name.to_string(),
                    ..Default::default()
                },
                None,
            )
            .await
            .unwrap();

        let mut data = Vec::with_capacity(size);
        while let Some(v) = downloaded.next().await {
            let d: bytes::Bytes = v.unwrap();
            data.extend_from_slice(d.chunk());
        }
        assert_eq!("hello world", String::from_utf8_lossy(data.as_slice()));
    }
}
