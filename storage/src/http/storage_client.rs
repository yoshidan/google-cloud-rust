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
    bucket_access_controls, buckets, default_object_access_controls, hmac_keys, notifications, object_access_controls,
    objects, CancellationToken, Error,
};
use futures_util::{Stream, TryStream};
use google_cloud_auth::token_source::TokenSource;

use reqwest::{Body, Client, RequestBuilder, Response};

use std::future::Future;

use crate::http::objects::download::Range;
use std::sync::Arc;

pub const SCOPES: [&str; 2] = [
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/devstorage.full_control",
];

#[derive(Clone)]
pub struct StorageClient {
    ts: Arc<dyn TokenSource>,
    v1_endpoint: String,
    v1_upload_endpoint: String,
}

impl StorageClient {
    pub(crate) fn new(ts: Arc<dyn TokenSource>, endpoint: &str) -> Self {
        Self {
            ts,
            v1_endpoint: format!("{}/storage/v1", endpoint),
            v1_upload_endpoint: format!("{}/upload/storage/v1", endpoint),
        }
    }

    /// Deletes the bucket.
    /// https://cloud.google.com/storage/docs/json_api/v1/buckets/delete
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::buckets::delete::DeleteBucketRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.delete_bucket(&DeleteBucketRequest {
    ///         bucket: "bucket".to_string(),
    ///         ..Default::default()
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn delete_bucket(
        &self,
        req: &DeleteBucketRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        self._delete_bucket(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn delete_bucket(
        &self,
        req: &DeleteBucketRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        self._delete_bucket(req, cancel).await
    }

    #[inline(always)]
    async fn _delete_bucket(&self, req: &DeleteBucketRequest, cancel: Option<CancellationToken>) -> Result<(), Error> {
        let action = async {
            let builder = buckets::delete::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send_get_empty(builder).await
        };
        invoke(cancel, action).await
    }

    /// Inserts the bucket.
    /// https://cloud.google.com/storage/docs/json_api/v1/buckets/insert
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::buckets::insert::{BucketCreationConfig, InsertBucketParam, InsertBucketRequest};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.insert_bucket(&InsertBucketRequest {
    ///         name: "bucket".to_string(),
    ///         param: InsertBucketParam {
    ///             project: client.project_id().to_string(),
    ///             ..Default::default()
    ///         },
    ///         ..Default::default()
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn insert_bucket(
        &self,
        req: &InsertBucketRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Bucket, Error> {
        self._insert_bucket(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn insert_bucket(
        &self,
        req: &InsertBucketRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Bucket, Error> {
        self._insert_bucket(req, cancel).await
    }

    #[inline(always)]
    async fn _insert_bucket(
        &self,
        req: &InsertBucketRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Bucket, Error> {
        let action = async {
            let builder = buckets::insert::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Gets the bucket.
    /// https://cloud.google.com/storage/docs/json_api/v1/buckets/get
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::buckets::get::GetBucketRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.get_bucket(&GetBucketRequest {
    ///         bucket: "bucket".to_string(),
    ///         ..Default::default()
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn get_bucket(&self, req: &GetBucketRequest, cancel: Option<CancellationToken>) -> Result<Bucket, Error> {
        self._get_bucket(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn get_bucket(&self, req: &GetBucketRequest, cancel: Option<CancellationToken>) -> Result<Bucket, Error> {
        self._get_bucket(req, cancel).await
    }

    #[inline(always)]
    async fn _get_bucket(&self, req: &GetBucketRequest, cancel: Option<CancellationToken>) -> Result<Bucket, Error> {
        let action = async {
            let builder = buckets::get::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Patches the bucket.
    /// https://cloud.google.com/storage/docs/json_api/v1/buckets/patch
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::buckets::patch::{BucketPatchConfig, PatchBucketRequest};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.patch_bucket(&PatchBucketRequest {
    ///         bucket: "bucket".to_string(),
    ///         metadata: Some(BucketPatchConfig {
    ///         ..Default::default()
    ///         }),
    ///         ..Default::default()
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn patch_bucket(
        &self,
        req: &PatchBucketRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Bucket, Error> {
        self._patch_bucket(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn patch_bucket(
        &self,
        req: &PatchBucketRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Bucket, Error> {
        self._patch_bucket(req, cancel).await
    }

    #[inline(always)]
    async fn _patch_bucket(
        &self,
        req: &PatchBucketRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Bucket, Error> {
        let action = async {
            let builder = buckets::patch::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Lists the bucket.
    /// https://cloud.google.com/storage/docs/json_api/v1/buckets/list
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::buckets::list::ListBucketsRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.list_buckets(&ListBucketsRequest{
    ///         project: client.project_id().to_string(),
    ///         ..Default::default()
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn list_buckets(
        &self,
        req: &ListBucketsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListBucketsResponse, Error> {
        self._list_buckets(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn list_buckets(
        &self,
        req: &ListBucketsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListBucketsResponse, Error> {
        self._list_buckets(req, cancel).await
    }

    #[inline(always)]
    async fn _list_buckets(
        &self,
        req: &ListBucketsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListBucketsResponse, Error> {
        let action = async {
            let builder = buckets::list::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Sets the iam policy.
    /// https://cloud.google.com/storage/docs/json_api/v1/buckets/setIamPolicy
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::buckets::{Binding, Policy};
    /// use google_cloud_storage::http::buckets::set_iam_policy::SetIamPolicyRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.set_iam_policy(&SetIamPolicyRequest{
    ///         resource: "bucket".to_string(),
    ///         policy: Policy {
    ///             bindings: vec![Binding {
    ///                 role: "roles/storage.objectViewer".to_string(),
    ///                 members: vec!["allAuthenticatedUsers".to_string()],
    ///                 condition: None,
    ///             }],
    ///             ..Default::default()
    ///         },
    ///         ..Default::default()
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn set_iam_policy(
        &self,
        req: &SetIamPolicyRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Policy, Error> {
        self._set_iam_policy(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn set_iam_policy(
        &self,
        req: &SetIamPolicyRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Policy, Error> {
        self._set_iam_policy(req, cancel).await
    }

    #[inline(always)]
    async fn _set_iam_policy(
        &self,
        req: &SetIamPolicyRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Policy, Error> {
        let action = async {
            let builder = buckets::set_iam_policy::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Gets the iam policy.
    /// https://cloud.google.com/storage/docs/json_api/v1/buckets/getIamPolicy
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::buckets::get_iam_policy::GetIamPolicyRequest;
    /// use google_cloud_storage::http::buckets::list::ListBucketsRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.get_iam_policy(&GetIamPolicyRequest{
    ///         resource: "bucket".to_string(),
    ///         ..Default::default()
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn get_iam_policy(
        &self,
        req: &GetIamPolicyRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Policy, Error> {
        self._get_iam_policy(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn get_iam_policy(
        &self,
        req: &GetIamPolicyRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Policy, Error> {
        self._get_iam_policy(req, cancel).await
    }

    #[inline(always)]
    async fn _get_iam_policy(
        &self,
        req: &GetIamPolicyRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Policy, Error> {
        let action = async {
            let builder = buckets::get_iam_policy::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Tests the iam permissions.
    /// https://cloud.google.com/storage/docs/json_api/v1/buckets/testIamPermissions
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::buckets::test_iam_permissions::TestIamPermissionsRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.test_iam_permissions(&TestIamPermissionsRequest{
    ///         resource: "bucket".to_string(),
    ///         permissions: vec!["storage.buckets.get".to_string()],
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn test_iam_permissions(
        &self,
        req: &TestIamPermissionsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<TestIamPermissionsResponse, Error> {
        self._test_iam_permissions(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn test_iam_permissions(
        &self,
        req: &TestIamPermissionsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<TestIamPermissionsResponse, Error> {
        self._test_iam_permissions(req, cancel).await
    }

    #[inline(always)]
    async fn _test_iam_permissions(
        &self,
        req: &TestIamPermissionsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<TestIamPermissionsResponse, Error> {
        let action = async {
            let builder = buckets::test_iam_permissions::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Lists the default object ACL.
    /// https://cloud.google.com/storage/docs/json_api/v1/defaultObjectAccessControls/list
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::buckets::test_iam_permissions::TestIamPermissionsRequest;
    /// use google_cloud_storage::http::default_object_access_controls::list::ListDefaultObjectAccessControlsRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.list_default_object_access_controls(&ListDefaultObjectAccessControlsRequest{
    ///         bucket: "bucket".to_string(),
    ///         ..Default::default()
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn list_default_object_access_controls(
        &self,
        req: &ListDefaultObjectAccessControlsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListDefaultObjectAccessControlsResponse, Error> {
        self._list_default_object_access_controls(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn list_default_object_access_controls(
        &self,
        req: &ListDefaultObjectAccessControlsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListDefaultObjectAccessControlsResponse, Error> {
        self._list_default_object_access_controls(req, cancel).await
    }

    #[inline(always)]
    async fn _list_default_object_access_controls(
        &self,
        req: &ListDefaultObjectAccessControlsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListDefaultObjectAccessControlsResponse, Error> {
        let action = async {
            let builder =
                default_object_access_controls::list::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Gets the default object ACL.
    /// https://cloud.google.com/storage/docs/json_api/v1/defaultObjectAccessControls/get
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::default_object_access_controls::get::GetDefaultObjectAccessControlRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.get_default_object_access_control(&GetDefaultObjectAccessControlRequest{
    ///         bucket: "bucket".to_string(),
    ///         entity: "allAuthenticatedUsers".to_string(),
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn get_default_object_access_control(
        &self,
        req: &GetDefaultObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        self._get_default_object_access_control(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn get_default_object_access_control(
        &self,
        req: &GetDefaultObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        self._get_default_object_access_control(req, cancel).await
    }

    #[inline(always)]
    async fn _get_default_object_access_control(
        &self,
        req: &GetDefaultObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        let action = async {
            let builder =
                default_object_access_controls::get::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Inserts the default object ACL.
    /// https://cloud.google.com/storage/docs/json_api/v1/defaultObjectAccessControls/insert
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::default_object_access_controls::insert::InsertDefaultObjectAccessControlRequest;
    /// use google_cloud_storage::http::object_access_controls::insert::ObjectAccessControlCreationConfig;
    /// use google_cloud_storage::http::object_access_controls::ObjectACLRole;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.insert_default_object_access_control(&InsertDefaultObjectAccessControlRequest{
    ///         bucket: "bucket".to_string(),
    ///         object_access_control: ObjectAccessControlCreationConfig {
    ///             entity: "allAuthenticatedUsers".to_string(),
    ///             role: ObjectACLRole::READER
    ///         } ,
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn insert_default_object_access_control(
        &self,
        req: &InsertDefaultObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        self._insert_default_object_access_control(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn insert_default_object_access_control(
        &self,
        req: &InsertDefaultObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        self._insert_default_object_access_control(req, cancel).await
    }

    #[inline(always)]
    async fn _insert_default_object_access_control(
        &self,
        req: &InsertDefaultObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        let action = async {
            let builder =
                default_object_access_controls::insert::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Patches the default object ACL.
    /// https://cloud.google.com/storage/docs/json_api/v1/defaultObjectAccessControls/patch
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::default_object_access_controls::patch::PatchDefaultObjectAccessControlRequest;
    /// use google_cloud_storage::http::object_access_controls::insert::ObjectAccessControlCreationConfig;
    /// use google_cloud_storage::http::object_access_controls::{ObjectAccessControl, ObjectACLRole};
    /// use google_cloud_storage::http::object_access_controls::patch::PatchObjectAccessControlRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.patch_default_object_access_control(&PatchDefaultObjectAccessControlRequest{
    ///         bucket: "bucket".to_string(),
    ///         entity: "allAuthenticatedUsers".to_string(),
    ///         object_access_control: ObjectAccessControl {
    ///             role: ObjectACLRole::READER,
    ///             ..Default::default()
    ///         },
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn patch_default_object_access_control(
        &self,
        req: &PatchDefaultObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        self._patch_default_object_access_control(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn patch_default_object_access_control(
        &self,
        req: &PatchDefaultObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        self._patch_default_object_access_control(req, cancel).await
    }

    #[inline(always)]
    async fn _patch_default_object_access_control(
        &self,
        req: &PatchDefaultObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        let action = async {
            let builder =
                default_object_access_controls::patch::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Deletes the default object ACL.
    /// https://cloud.google.com/storage/docs/json_api/v1/defaultObjectAccessControls/delete
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::default_object_access_controls::delete::DeleteDefaultObjectAccessControlRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.delete_default_object_access_control(&DeleteDefaultObjectAccessControlRequest{
    ///         bucket: "bucket".to_string(),
    ///         entity: "allAuthenticatedUsers".to_string(),
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn delete_default_object_access_control(
        &self,
        req: &DeleteDefaultObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        self._delete_default_object_access_control(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn delete_default_object_access_control(
        &self,
        req: &DeleteDefaultObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        self._delete_default_object_access_control(req, cancel).await
    }

    #[inline(always)]
    async fn _delete_default_object_access_control(
        &self,
        req: &DeleteDefaultObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let action = async {
            let builder =
                default_object_access_controls::delete::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send_get_empty(builder).await
        };
        invoke(cancel, action).await
    }

    /// Lists the bucket ACL.
    /// https://cloud.google.com/storage/docs/json_api/v1/bucketAccessControls/list
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::bucket_access_controls::list::ListBucketAccessControlsRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.list_bucket_access_controls(&ListBucketAccessControlsRequest{
    ///         bucket: "bucket".to_string(),
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn list_bucket_access_controls(
        &self,
        req: &ListBucketAccessControlsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListBucketAccessControlsResponse, Error> {
        self._list_bucket_access_controls(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn list_bucket_access_controls(
        &self,
        req: &ListBucketAccessControlsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListBucketAccessControlsResponse, Error> {
        self._list_bucket_access_controls(req, cancel).await
    }

    #[inline(always)]
    async fn _list_bucket_access_controls(
        &self,
        req: &ListBucketAccessControlsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListBucketAccessControlsResponse, Error> {
        let action = async {
            let builder = bucket_access_controls::list::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Gets the bucket ACL.
    /// https://cloud.google.com/storage/docs/json_api/v1/bucketAccessControls/get
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::bucket_access_controls::get::GetBucketAccessControlRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.get_bucket_access_control(&GetBucketAccessControlRequest{
    ///         bucket: "bucket".to_string(),
    ///         entity: "allAuthenticatedUsers".to_string(),
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn get_bucket_access_control(
        &self,
        req: &GetBucketAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<BucketAccessControl, Error> {
        self._get_bucket_access_control(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn get_bucket_access_control(
        &self,
        req: &GetBucketAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<BucketAccessControl, Error> {
        self._get_bucket_access_control(req, cancel).await
    }

    #[inline(always)]
    async fn _get_bucket_access_control(
        &self,
        req: &GetBucketAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<BucketAccessControl, Error> {
        let action = async {
            let builder = bucket_access_controls::get::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Inserts the bucket ACL.
    /// https://cloud.google.com/storage/docs/json_api/v1/bucketAccessControls/insert
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::bucket_access_controls::BucketACLRole;
    /// use google_cloud_storage::http::bucket_access_controls::insert::{BucketAccessControlCreationConfig, InsertBucketAccessControlRequest};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.insert_bucket_access_control(&InsertBucketAccessControlRequest{
    ///         bucket: "bucket".to_string(),
    ///         acl: BucketAccessControlCreationConfig {
    ///             entity: "allAuthenticatedUsers".to_string(),
    ///             role: BucketACLRole::READER
    ///         }
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn insert_bucket_access_control(
        &self,
        req: &InsertBucketAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<BucketAccessControl, Error> {
        self._insert_bucket_access_control(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn insert_bucket_access_control(
        &self,
        req: &InsertBucketAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<BucketAccessControl, Error> {
        self._insert_bucket_access_control(req, cancel).await
    }

    #[inline(always)]
    async fn _insert_bucket_access_control(
        &self,
        req: &InsertBucketAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<BucketAccessControl, Error> {
        let action = async {
            let builder = bucket_access_controls::insert::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Patches the bucket ACL.
    /// https://cloud.google.com/storage/docs/json_api/v1/bucketAccessControls/patch
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::bucket_access_controls::BucketAccessControl;
    /// use google_cloud_storage::http::bucket_access_controls::BucketACLRole;
    /// use google_cloud_storage::http::bucket_access_controls::patch::PatchBucketAccessControlRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.patch_bucket_access_control(&PatchBucketAccessControlRequest{
    ///         bucket: "bucket".to_string(),
    ///         entity: "allAuthenticatedUsers".to_string(),
    ///         acl: BucketAccessControl {
    ///             role: BucketACLRole::READER,
    ///             ..Default::default()
    ///         }
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn patch_bucket_access_control(
        &self,
        req: &PatchBucketAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<BucketAccessControl, Error> {
        self._patch_bucket_access_control(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn patch_bucket_access_control(
        &self,
        req: &PatchBucketAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<BucketAccessControl, Error> {
        self._patch_bucket_access_control(req, cancel).await
    }

    #[inline(always)]
    async fn _patch_bucket_access_control(
        &self,
        req: &PatchBucketAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<BucketAccessControl, Error> {
        let action = async {
            let builder = bucket_access_controls::patch::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Deletes the bucket ACL.
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::bucket_access_controls::BucketAccessControl;
    /// use google_cloud_storage::http::bucket_access_controls::delete::DeleteBucketAccessControlRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.delete_bucket_access_control(&DeleteBucketAccessControlRequest{
    ///         bucket: "bucket".to_string(),
    ///         entity: "allAuthenticatedUsers".to_string(),
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn delete_bucket_access_control(
        &self,
        req: &DeleteBucketAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        self._delete_bucket_access_control(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn delete_bucket_access_control(
        &self,
        req: &DeleteBucketAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        self._delete_bucket_access_control(req, cancel).await
    }

    #[inline(always)]
    pub async fn _delete_bucket_access_control(
        &self,
        req: &DeleteBucketAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let action = async {
            let builder = bucket_access_controls::delete::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send_get_empty(builder).await
        };
        invoke(cancel, action).await
    }

    /// Lists the object ACL.
    /// https://cloud.google.com/storage/docs/json_api/v1/objectAccessControls/list
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::object_access_controls::list::ListObjectAccessControlsRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.list_object_access_controls(&ListObjectAccessControlsRequest{
    ///         bucket: "bucket".to_string(),
    ///         object: "filename".to_string(),
    ///         ..Default::default()
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn list_object_access_controls(
        &self,
        req: &ListObjectAccessControlsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListBucketAccessControlsResponse, Error> {
        self._list_object_access_controls(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn list_object_access_controls(
        &self,
        req: &ListObjectAccessControlsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListBucketAccessControlsResponse, Error> {
        self._list_object_access_controls(req, cancel).await
    }

    #[inline(always)]
    async fn _list_object_access_controls(
        &self,
        req: &ListObjectAccessControlsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListBucketAccessControlsResponse, Error> {
        let action = async {
            let builder = object_access_controls::list::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Gets the object ACL.
    /// https://cloud.google.com/storage/docs/json_api/v1/objectAccessControls/get
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::object_access_controls::get::GetObjectAccessControlRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.get_object_access_control(&GetObjectAccessControlRequest{
    ///         bucket: "bucket".to_string(),
    ///         object: "filename".to_string(),
    ///         entity: "allAuthenticatedUsers".to_string(),
    ///         ..Default::default()
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn get_object_access_control(
        &self,
        req: &GetObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        self._get_object_access_control(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn get_object_access_control(
        &self,
        req: &GetObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        self._get_object_access_control(req, cancel).await
    }

    #[inline(always)]
    async fn _get_object_access_control(
        &self,
        req: &GetObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        let action = async {
            let builder = object_access_controls::get::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Inserts the object ACL.
    /// https://cloud.google.com/storage/docs/json_api/v1/objectAccessControls/insert
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::object_access_controls::insert::{InsertObjectAccessControlRequest, ObjectAccessControlCreationConfig};
    /// use google_cloud_storage::http::object_access_controls::ObjectACLRole;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.insert_object_access_control(&InsertObjectAccessControlRequest{
    ///         bucket: "bucket".to_string(),
    ///         object: "filename".to_string(),
    ///         acl: ObjectAccessControlCreationConfig {
    ///             entity: "allAuthenticatedUsers".to_string(),
    ///             role: ObjectACLRole::READER
    ///         },
    ///         ..Default::default()
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn insert_object_access_control(
        &self,
        req: &InsertObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        self._insert_object_access_control(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn insert_object_access_control(
        &self,
        req: &InsertObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        self._insert_object_access_control(req, cancel).await
    }

    #[inline(always)]
    async fn _insert_object_access_control(
        &self,
        req: &InsertObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        let action = async {
            let builder = object_access_controls::insert::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Patches the bucket ACL.
    /// https://cloud.google.com/storage/docs/json_api/v1/objectAccessControls/patch
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::object_access_controls::{ObjectAccessControl, ObjectACLRole};
    /// use google_cloud_storage::http::object_access_controls::patch::PatchObjectAccessControlRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.patch_object_access_control(&PatchObjectAccessControlRequest{
    ///         bucket: "bucket".to_string(),
    ///         object: "filename".to_string(),
    ///         entity: "allAuthenticatedUsers".to_string(),
    ///         acl: ObjectAccessControl {
    ///             role: ObjectACLRole::READER,
    ///             ..Default::default()
    ///         },
    ///         ..Default::default()
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn patch_object_access_control(
        &self,
        req: &PatchObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        self._patch_object_access_control(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn patch_object_access_control(
        &self,
        req: &PatchObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        self._patch_object_access_control(req, cancel).await
    }

    #[inline(always)]
    pub async fn _patch_object_access_control(
        &self,
        req: &PatchObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        let action = async {
            let builder = object_access_controls::patch::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Deletes the bucket ACL.
    /// https://cloud.google.com/storage/docs/json_api/v1/objectAccessControls/delete
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::object_access_controls::{ObjectAccessControl, ObjectACLRole};
    /// use google_cloud_storage::http::object_access_controls::delete::DeleteObjectAccessControlRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.delete_object_access_control(&DeleteObjectAccessControlRequest{
    ///         bucket: "bucket".to_string(),
    ///         object: "filename".to_string(),
    ///         entity: "allAuthenticatedUsers".to_string(),
    ///         ..Default::default()
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn delete_object_access_control(
        &self,
        req: &DeleteObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        self._delete_object_access_control(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn delete_object_access_control(
        &self,
        req: &DeleteObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        self._delete_object_access_control(req, cancel).await
    }

    #[inline(always)]
    async fn _delete_object_access_control(
        &self,
        req: &DeleteObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let action = async {
            let builder = object_access_controls::delete::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send_get_empty(builder).await
        };
        invoke(cancel, action).await
    }

    /// Lists the notification.
    /// https://cloud.google.com/storage/docs/json_api/v1/notifications/list
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::notifications::list::ListNotificationsRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.list_notifications(&ListNotificationsRequest{
    ///         bucket: "bucket".to_string(),
    ///         ..Default::default()
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn list_notifications(
        &self,
        req: &ListNotificationsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListNotificationsResponse, Error> {
        self._list_notifications(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn list_notifications(
        &self,
        req: &ListNotificationsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListNotificationsResponse, Error> {
        self._list_notifications(req, cancel).await
    }

    #[inline(always)]
    async fn _list_notifications(
        &self,
        req: &ListNotificationsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListNotificationsResponse, Error> {
        let action = async {
            let builder = notifications::list::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Gets the notification.
    /// https://cloud.google.com/storage/docs/json_api/v1/notifications/get
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::notifications::get::GetNotificationRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.get_notification(&GetNotificationRequest{
    ///         bucket: "bucket".to_string(),
    ///         notification: "notification".to_string()
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn get_notification(
        &self,
        req: &GetNotificationRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Notification, Error> {
        self._get_notification(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn get_notification(
        &self,
        req: &GetNotificationRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Notification, Error> {
        self._get_notification(req, cancel).await
    }

    #[inline(always)]
    async fn _get_notification(
        &self,
        req: &GetNotificationRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Notification, Error> {
        let action = async {
            let builder = notifications::get::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Inserts the notification.
    /// https://cloud.google.com/storage/docs/json_api/v1/notifications/insert
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::notifications::EventType;
    /// use google_cloud_storage::http::notifications::insert::{InsertNotificationRequest, NotificationCreationConfig};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.insert_notification(&InsertNotificationRequest {
    ///         bucket: "bucket".to_string(),
    ///         notification: NotificationCreationConfig {
    ///             topic: format!("projects/{}/topics/{}", "project","bucket"),
    ///             event_types: Some(vec![EventType::ObjectMetadataUpdate, EventType::ObjectDelete]),
    ///             ..Default::default()
    ///         }
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn insert_notification(
        &self,
        req: &InsertNotificationRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Notification, Error> {
        self._insert_notification(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn insert_notification(
        &self,
        req: &InsertNotificationRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Notification, Error> {
        self._insert_notification(req, cancel).await
    }

    #[inline(always)]
    async fn _insert_notification(
        &self,
        req: &InsertNotificationRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Notification, Error> {
        let action = async {
            let builder = notifications::insert::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Deletes the notification.
    /// https://cloud.google.com/storage/docs/json_api/v1/notifications/delete
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::notifications::delete::DeleteNotificationRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.delete_notification(&DeleteNotificationRequest {
    ///         bucket: "bucket".to_string(),
    ///         notification: "notification".to_string()
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn delete_notification(
        &self,
        req: &DeleteNotificationRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        self._delete_notification(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn delete_notification(
        &self,
        req: &DeleteNotificationRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        self._delete_notification(req, cancel).await
    }

    #[inline(always)]
    async fn _delete_notification(
        &self,
        req: &DeleteNotificationRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let action = async {
            let builder = notifications::delete::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send_get_empty(builder).await
        };
        invoke(cancel, action).await
    }

    /// Lists the hmac keys.
    /// https://cloud.google.com/storage/docs/json_api/v1/projects/hmacKeys/list
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::hmac_keys::list::ListHmacKeysRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.list_hmac_keys(&ListHmacKeysRequest {
    ///         project_id: client.project_id().to_string(),
    ///         ..Default::default()
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn list_hmac_keys(
        &self,
        req: &ListHmacKeysRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListHmacKeysResponse, Error> {
        self._list_hmac_keys(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn list_hmac_keys(
        &self,
        req: &ListHmacKeysRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListHmacKeysResponse, Error> {
        self._list_hmac_keys(req, cancel).await
    }

    #[inline(always)]
    async fn _list_hmac_keys(
        &self,
        req: &ListHmacKeysRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListHmacKeysResponse, Error> {
        let action = async {
            let builder = hmac_keys::list::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Gets the hmac keys.
    /// https://cloud.google.com/storage/docs/json_api/v1/projects/hmacKeys/get
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::hmac_keys::get::GetHmacKeyRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.get_hmac_key(&GetHmacKeyRequest {
    ///         access_id: "access_id".to_string(),
    ///         project_id: client.project_id().to_string(),
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn get_hmac_key(
        &self,
        req: &GetHmacKeyRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<HmacKeyMetadata, Error> {
        self._get_hmac_key(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn get_hmac_key(
        &self,
        req: &GetHmacKeyRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<HmacKeyMetadata, Error> {
        self._get_hmac_key(req, cancel).await
    }

    #[inline(always)]
    async fn _get_hmac_key(
        &self,
        req: &GetHmacKeyRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<HmacKeyMetadata, Error> {
        let action = async {
            let builder = hmac_keys::get::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Creates the hmac key.
    /// https://cloud.google.com/storage/docs/json_api/v1/projects/hmacKeys/create
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::hmac_keys::create::CreateHmacKeyRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.create_hmac_key(&CreateHmacKeyRequest {
    ///         service_account_email: "service_account_email".to_string(),
    ///         project_id: client.project_id().to_string(),
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn create_hmac_key(
        &self,
        req: &CreateHmacKeyRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<CreateHmacKeyResponse, Error> {
        self._create_hmac_key(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn create_hmac_key(
        &self,
        req: &CreateHmacKeyRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<CreateHmacKeyResponse, Error> {
        self._create_hmac_key(req, cancel).await
    }

    #[inline(always)]
    async fn _create_hmac_key(
        &self,
        req: &CreateHmacKeyRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<CreateHmacKeyResponse, Error> {
        let action = async {
            let builder = hmac_keys::create::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Updates the hmac key.
    /// https://cloud.google.com/storage/docs/json_api/v1/projects/hmacKeys/update
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::hmac_keys::HmacKeyMetadata;
    /// use google_cloud_storage::http::hmac_keys::update::UpdateHmacKeyRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.update_hmac_key(&UpdateHmacKeyRequest{
    ///         access_id: "access_id".to_string(),
    ///         project_id: client.project_id().to_string(),
    ///         metadata: HmacKeyMetadata {
    ///             state: "INACTIVE".to_string(),
    ///             ..Default::default()
    ///         },
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn update_hmac_key(
        &self,
        req: &UpdateHmacKeyRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<HmacKeyMetadata, Error> {
        self._update_hmac_key(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn update_hmac_key(
        &self,
        req: &UpdateHmacKeyRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<HmacKeyMetadata, Error> {
        self._update_hmac_key(req, cancel).await
    }

    #[inline(always)]
    async fn _update_hmac_key(
        &self,
        req: &UpdateHmacKeyRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<HmacKeyMetadata, Error> {
        let action = async {
            let builder = hmac_keys::update::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Deletes the hmac key.
    /// https://cloud.google.com/storage/docs/json_api/v1/projects/hmacKeys/delete
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::hmac_keys::delete::DeleteHmacKeyRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.delete_hmac_key(&DeleteHmacKeyRequest{
    ///         access_id: "access_id".to_string(),
    ///         project_id: client.project_id().to_string(),
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn delete_hmac_key(
        &self,
        req: &DeleteHmacKeyRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        self._delete_hmac_key(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn delete_hmac_key(
        &self,
        req: &DeleteHmacKeyRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        self._delete_hmac_key(req, cancel).await
    }

    #[inline(always)]
    async fn _delete_hmac_key(
        &self,
        req: &DeleteHmacKeyRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let action = async {
            let builder = hmac_keys::delete::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send_get_empty(builder).await
        };
        invoke(cancel, action).await
    }

    /// Lists the objects.
    /// https://cloud.google.com/storage/docs/json_api/v1/objects/list
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::objects::list::ListObjectsRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.list_objects(&ListObjectsRequest{
    ///         bucket: "bucket".to_string(),
    ///         ..Default::default()
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn list_objects(
        &self,
        req: &ListObjectsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListObjectsResponse, Error> {
        self._list_objects(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn list_objects(
        &self,
        req: &ListObjectsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListObjectsResponse, Error> {
        self._list_objects(req, cancel).await
    }

    #[inline(always)]
    async fn _list_objects(
        &self,
        req: &ListObjectsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListObjectsResponse, Error> {
        let action = async {
            let builder = objects::list::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Gets the object.
    /// https://cloud.google.com/storage/docs/json_api/v1/objects/get
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::objects::get::GetObjectRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.get_object(&GetObjectRequest{
    ///         bucket: "bucket".to_string(),
    ///         object: "object".to_string(),
    ///         ..Default::default()
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn get_object(&self, req: &GetObjectRequest, cancel: Option<CancellationToken>) -> Result<Object, Error> {
        self._get_object(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn get_object(&self, req: &GetObjectRequest, cancel: Option<CancellationToken>) -> Result<Object, Error> {
        self._get_object(req, cancel).await
    }

    #[inline(always)]
    async fn _get_object(&self, req: &GetObjectRequest, cancel: Option<CancellationToken>) -> Result<Object, Error> {
        let action = async {
            let builder = objects::get::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Download the object.
    /// https://cloud.google.com/storage/docs/json_api/v1/objects/get
    /// alt is always media
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::objects::get::GetObjectRequest;
    /// use google_cloud_storage::http::objects::download::Range;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.download_object(&GetObjectRequest{
    ///         bucket: "bucket".to_string(),
    ///         object: "object".to_string(),
    ///         ..Default::default()
    ///     }, &Range::default(), None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn download_object(
        &self,
        req: &GetObjectRequest,
        range: &Range,
        cancel: Option<CancellationToken>,
    ) -> Result<Vec<u8>, Error> {
        self._download_object(req, range, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn download_object(
        &self,
        req: &GetObjectRequest,
        range: &Range,
        cancel: Option<CancellationToken>,
    ) -> Result<Vec<u8>, Error> {
        self._download_object(req, range, cancel).await
    }

    #[inline(always)]
    async fn _download_object(
        &self,
        req: &GetObjectRequest,
        range: &Range,
        cancel: Option<CancellationToken>,
    ) -> Result<Vec<u8>, Error> {
        let action = async {
            let builder = objects::download::build(self.v1_endpoint.as_str(), &Client::default(), req, range);
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
    /// https://cloud.google.com/storage/docs/json_api/v1/objects/get
    /// alt is always media
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::objects::get::GetObjectRequest;
    /// use google_cloud_storage::http::objects::download::Range;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.download_streamed_object(&GetObjectRequest{
    ///         bucket: "bucket".to_string(),
    ///         object: "object".to_string(),
    ///         ..Default::default()
    ///     }, &Range::default(), None).await;
    ///
    ///     //  while let Some(v) = downloaded.next().await? {
    ///     //      let d: bytes::Bytes = v.unwrap();
    ///     //  }
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn download_streamed_object(
        &self,
        req: &GetObjectRequest,
        range: &Range,
        cancel: Option<CancellationToken>,
    ) -> Result<impl Stream<Item = reqwest::Result<bytes::Bytes>>, Error> {
        self._download_streamed_object(req, range, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn download_streamed_object(
        &self,
        req: &GetObjectRequest,
        range: &Range,
        cancel: Option<CancellationToken>,
    ) -> Result<impl Stream<Item = reqwest::Result<bytes::Bytes>>, Error> {
        self._download_streamed_object(req, range, cancel).await
    }

    #[inline(always)]
    async fn _download_streamed_object(
        &self,
        req: &GetObjectRequest,
        range: &Range,
        cancel: Option<CancellationToken>,
    ) -> Result<impl Stream<Item = reqwest::Result<bytes::Bytes>>, Error> {
        let action = async {
            let builder = objects::download::build(self.v1_endpoint.as_str(), &Client::default(), req, range);
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
    /// https://cloud.google.com/storage/docs/json_api/v1/objects/insert
    /// 'uploadType' is always media - Data-only upload. Upload the object data only, without any metadata.
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::objects::upload::UploadObjectRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.upload_object(&UploadObjectRequest{
    ///         bucket: "bucket".to_string(),
    ///         name: "filename".to_string(),
    ///         ..Default::default()
    ///     }, "hello world".as_bytes(), "application/octet-stream", None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn upload_object(
        &self,
        req: &UploadObjectRequest,
        data: &[u8],
        content_type: &str,
        cancel: Option<CancellationToken>,
    ) -> Result<Object, Error> {
        self._upload_object(req, data, content_type, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn upload_object(
        &self,
        req: &UploadObjectRequest,
        data: &[u8],
        content_type: &str,
        cancel: Option<CancellationToken>,
    ) -> Result<Object, Error> {
        self._upload_object(req, data, content_type, cancel).await
    }

    #[inline(always)]
    async fn _upload_object(
        &self,
        req: &UploadObjectRequest,
        data: &[u8],
        content_type: &str,
        cancel: Option<CancellationToken>,
    ) -> Result<Object, Error> {
        let action = async {
            let builder = objects::upload::build(
                self.v1_upload_endpoint.as_str(),
                &Client::default(),
                req,
                Some(data.len()),
                content_type,
                Vec::from(data),
            );
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Uploads the streamed object.
    /// https://cloud.google.com/storage/docs/json_api/v1/objects/insert
    /// 'uploadType' is always media - Data-only upload. Upload the object data only, without any metadata.
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::objects::upload::UploadObjectRequest;
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let source = vec!["hello", " ", "world"];
    ///     let size = source.iter().map(|x| x.len()).sum();
    ///     let chunks: Vec<Result<_, ::std::io::Error>> = source.clone().into_iter().map(|x| Ok(x)).collect();
    ///     let stream = futures_util::stream::iter(chunks);
    ///     let result = client.upload_streamed_object(&UploadObjectRequest{
    ///         bucket: "bucket".to_string(),
    ///         name: "filename".to_string(),
    ///         ..Default::default()
    ///     }, stream, "application/octet-stream", Some(size), None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn upload_streamed_object<S>(
        &self,
        req: &UploadObjectRequest,
        data: S,
        content_type: &str,
        content_length: Option<usize>,
        cancel: Option<CancellationToken>,
    ) -> Result<Object, Error>
    where
        S: TryStream + Send + Sync + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        bytes::Bytes: From<S::Ok>,
    {
        self._upload_streamed_object(req, data, content_type, content_length, cancel)
            .await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn upload_streamed_object<S>(
        &self,
        req: &UploadObjectRequest,
        data: S,
        content_type: &str,
        content_length: Option<usize>,
        cancel: Option<CancellationToken>,
    ) -> Result<Object, Error>
    where
        S: TryStream + Send + Sync + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        bytes::Bytes: From<S::Ok>,
    {
        self._upload_streamed_object(req, data, content_type, content_length, cancel)
            .await
    }

    #[inline(always)]
    async fn _upload_streamed_object<S>(
        &self,
        req: &UploadObjectRequest,
        data: S,
        content_type: &str,
        content_length: Option<usize>,
        cancel: Option<CancellationToken>,
    ) -> Result<Object, Error>
    where
        S: TryStream + Send + Sync + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        bytes::Bytes: From<S::Ok>,
    {
        let action = async {
            let builder = objects::upload::build(
                self.v1_upload_endpoint.as_str(),
                &Client::default(),
                req,
                content_length,
                content_type,
                Body::wrap_stream(data),
            );
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Patches the object.
    /// https://cloud.google.com/storage/docs/json_api/v1/objects/patch
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::objects::patch::PatchObjectRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.patch_object(&PatchObjectRequest{
    ///         bucket: "bucket".to_string(),
    ///         object: "object".to_string(),
    ///         ..Default::default()
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn patch_object(
        &self,
        req: &PatchObjectRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Object, Error> {
        self._patch_object(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn patch_object(
        &self,
        req: &PatchObjectRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Object, Error> {
        self._patch_object(req, cancel).await
    }

    #[inline(always)]
    async fn _patch_object(
        &self,
        req: &PatchObjectRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Object, Error> {
        let action = async {
            let builder = objects::patch::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Deletes the object.
    /// https://cloud.google.com/storage/docs/json_api/v1/objects/delete
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::objects::delete::DeleteObjectRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.delete_object(&DeleteObjectRequest{
    ///         bucket: "bucket".to_string(),
    ///         object: "object".to_string(),
    ///         ..Default::default()
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn delete_object(
        &self,
        req: &DeleteObjectRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        self._delete_object(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn delete_object(
        &self,
        req: &DeleteObjectRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        self._delete_object(req, cancel).await
    }

    #[inline(always)]
    async fn _delete_object(&self, req: &DeleteObjectRequest, cancel: Option<CancellationToken>) -> Result<(), Error> {
        let action = async {
            let builder = objects::delete::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send_get_empty(builder).await
        };
        invoke(cancel, action).await
    }

    /// Rewrites the object.
    /// https://cloud.google.com/storage/docs/json_api/v1/objects/rewrite
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::objects::rewrite::RewriteObjectRequest;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.rewrite_object(&RewriteObjectRequest{
    ///         source_bucket: "bucket1".to_string(),
    ///         source_object: "object".to_string(),
    ///         destination_bucket: "bucket2".to_string(),
    ///         destination_object: "object1".to_string(),
    ///         ..Default::default()
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn rewrite_object(
        &self,
        req: &RewriteObjectRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<RewriteObjectResponse, Error> {
        self._rewrite_object(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn rewrite_object(
        &self,
        req: &RewriteObjectRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<RewriteObjectResponse, Error> {
        self._rewrite_object(req, cancel).await
    }

    #[inline(always)]
    async fn _rewrite_object(
        &self,
        req: &RewriteObjectRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<RewriteObjectResponse, Error> {
        let action = async {
            let builder = objects::rewrite::build(self.v1_endpoint.as_str(), &Client::default(), req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Composes the object.
    /// https://cloud.google.com/storage/docs/json_api/v1/objects/compose
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::objects::compose::{ComposeObjectRequest, ComposingTargets};
    /// use google_cloud_storage::http::objects::rewrite::RewriteObjectRequest;
    /// use google_cloud_storage::http::objects::SourceObjects;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let result = client.compose_object(&ComposeObjectRequest{
    ///         bucket: "bucket1".to_string(),
    ///         destination_object: "object1".to_string(),
    ///         composing_targets: ComposingTargets {
    ///             source_objects: vec![SourceObjects {
    ///                 name: "src".to_string(),
    ///                 ..Default::default()
    ///             }],
    ///             ..Default::default()
    ///         },
    ///         ..Default::default()
    ///     }, None).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn compose_object(
        &self,
        req: &ComposeObjectRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Object, Error> {
        self._compose_object(req, cancel).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn compose_object(
        &self,
        req: &ComposeObjectRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Object, Error> {
        self._compose_object(req, cancel).await
    }

    #[inline(always)]
    async fn _compose_object(
        &self,
        req: &ComposeObjectRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Object, Error> {
        let action = async {
            let builder = objects::compose::build(self.v1_endpoint.as_str(), &Client::default(), req);
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
            let full = response.bytes().await?;
            tracing::trace!("response={:?}", &full);
            Ok(serde_json::from_slice(&full)?)
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
    use crate::http::bucket_access_controls::BucketACLRole;
    use crate::http::buckets::delete::DeleteBucketRequest;
    use crate::http::buckets::get::GetBucketRequest;
    use crate::http::buckets::get_iam_policy::GetIamPolicyRequest;
    use crate::http::buckets::insert::{BucketCreationConfig, InsertBucketParam, InsertBucketRequest};
    use crate::http::buckets::list::ListBucketsRequest;
    use crate::http::buckets::patch::{BucketPatchConfig, PatchBucketRequest};
    use crate::http::buckets::set_iam_policy::SetIamPolicyRequest;
    use crate::http::buckets::test_iam_permissions::TestIamPermissionsRequest;
    use crate::http::buckets::Binding;
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
    use crate::http::object_access_controls::ObjectACLRole;
    use crate::http::objects::compose::{ComposeObjectRequest, ComposingTargets};
    use crate::http::objects::delete::DeleteObjectRequest;
    use crate::http::objects::get::GetObjectRequest;
    use crate::http::objects::list::ListObjectsRequest;
    use crate::http::objects::rewrite::RewriteObjectRequest;
    use crate::http::objects::upload::UploadObjectRequest;

    use crate::http::notifications::EventType;
    use crate::http::objects::download::Range;
    use crate::http::objects::SourceObjects;
    use crate::http::storage_client::{StorageClient, SCOPES};
    use bytes::Buf;
    use futures_util::StreamExt;
    use google_cloud_auth::{create_token_source, Config};
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
        StorageClient::new(Arc::from(ts), "https://storage.googleapis.com")
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
        let name = format!("rust-test-insert-{}", time::OffsetDateTime::now_utc().unix_timestamp());
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

        client
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

        client
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

        client
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

        for n in notifications.items.unwrap_or_default() {
            client
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
                        topic: format!("projects/{}/topics/{}", PROJECT, bucket_name),
                        event_types: Some(vec![EventType::ObjectMetadataUpdate, EventType::ObjectDelete]),
                        object_name_prefix: Some("notification-test".to_string()),
                        ..Default::default()
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

        for n in keys.items.unwrap_or_default() {
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

            client
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
            .unwrap_or_default();
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
                &[1, 2, 3, 4, 5, 6],
                "text/plain",
                None,
            )
            .await
            .unwrap();

        let download = |range: Range| {
            let client = client.clone();
            let bucket_name = uploaded.bucket.clone();
            let object_name = uploaded.name.clone();
            async move {
                client
                    .download_object(
                        &GetObjectRequest {
                            bucket: bucket_name,
                            object: object_name,
                            ..Default::default()
                        },
                        &range,
                        None,
                    )
                    .await
                    .unwrap()
            }
        };

        let downloaded = download(Range::default()).await;
        assert_eq!(downloaded, vec![1, 2, 3, 4, 5, 6]);
        let downloaded = download(Range(Some(1), None)).await;
        assert_eq!(downloaded, vec![2, 3, 4, 5, 6]);
        let downloaded = download(Range(Some(1), Some(2))).await;
        assert_eq!(downloaded, vec![2, 3]);
        let downloaded = download(Range(None, Some(2))).await;
        assert_eq!(downloaded, vec![5, 6]);

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
        let file_name = format!("stream_{}", time::OffsetDateTime::now_utc().unix_timestamp());
        let client = client().await;

        // let stream= reqwest::Client::default().get("https://avatars.githubusercontent.com/u/958174?s=96&v=4").send().await.unwrap().bytes_stream();
        let source = vec!["hello", " ", "world"];
        let size = source.iter().map(|x| x.len()).sum();
        let chunks: Vec<Result<_, ::std::io::Error>> = source.clone().into_iter().map(Ok).collect();
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
                Some(size),
                None,
            )
            .await
            .unwrap();

        let download = |range: Range| {
            let client = client.clone();
            let bucket_name = uploaded.bucket.clone();
            let object_name = uploaded.name.clone();
            async move {
                let mut downloaded = client
                    .download_streamed_object(
                        &GetObjectRequest {
                            bucket: bucket_name,
                            object: object_name,
                            ..Default::default()
                        },
                        &range,
                        None,
                    )
                    .await
                    .unwrap();
                let mut data = Vec::with_capacity(10);
                while let Some(v) = downloaded.next().await {
                    let d: bytes::Bytes = v.unwrap();
                    data.extend_from_slice(d.chunk());
                }
                data
            }
        };
        let downloaded = download(Range::default()).await;
        assert_eq!("hello world", String::from_utf8_lossy(downloaded.as_slice()));
        let downloaded = download(Range(Some(1), None)).await;
        assert_eq!("ello world", String::from_utf8_lossy(downloaded.as_slice()));
        let downloaded = download(Range(Some(1), Some(2))).await;
        assert_eq!("el", String::from_utf8_lossy(downloaded.as_slice()));
        let downloaded = download(Range(None, Some(2))).await;
        assert_eq!("ld", String::from_utf8_lossy(downloaded.as_slice()));
    }
}
