use std::sync::Arc;

use futures_util::{Stream, TryStream, TryStreamExt};
use reqwest::header::LOCATION;
use reqwest::{Body, Client, RequestBuilder};

use google_cloud_token::TokenSource;

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
use crate::http::objects::copy::CopyObjectRequest;
use crate::http::objects::delete::DeleteObjectRequest;
use crate::http::objects::download::Range;
use crate::http::objects::get::GetObjectRequest;
use crate::http::objects::list::{ListObjectsRequest, ListObjectsResponse};
use crate::http::objects::patch::PatchObjectRequest;
use crate::http::objects::rewrite::{RewriteObjectRequest, RewriteObjectResponse};
use crate::http::objects::upload::{UploadObjectRequest, UploadType};
use crate::http::objects::Object;
use crate::http::resumable_upload_client::ResumableUploadClient;
use crate::http::{
    bucket_access_controls, buckets, check_response_status, default_object_access_controls, hmac_keys, notifications,
    object_access_controls, objects, Error,
};

pub const SCOPES: [&str; 2] = [
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/devstorage.full_control",
];

#[derive(Clone)]
pub struct StorageClient {
    ts: Arc<dyn TokenSource>,
    v1_endpoint: String,
    v1_upload_endpoint: String,
    http: Client,
}

impl StorageClient {
    pub(crate) fn new(ts: Arc<dyn TokenSource>, endpoint: &str, http: Client) -> Self {
        Self {
            ts,
            v1_endpoint: format!("{endpoint}/storage/v1"),
            v1_upload_endpoint: format!("{endpoint}/upload/storage/v1"),
            http,
        }
    }

    /// Deletes the bucket.
    /// https://cloud.google.com/storage/docs/json_api/v1/buckets/delete
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::buckets::delete::DeleteBucketRequest;
    ///
    /// async fn run(client:Client) {
    ///     let result = client.delete_bucket(&DeleteBucketRequest {
    ///         bucket: "bucket".to_string(),
    ///         ..Default::default()
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn delete_bucket(&self, req: &DeleteBucketRequest) -> Result<(), Error> {
        let builder = buckets::delete::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send_get_empty(builder).await
    }

    /// Inserts the bucket.
    /// https://cloud.google.com/storage/docs/json_api/v1/buckets/insert
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::buckets::insert::{BucketCreationConfig, InsertBucketParam, InsertBucketRequest};
    ///
    /// async fn run(client:Client) {
    ///     let result = client.insert_bucket(&InsertBucketRequest {
    ///         name: "bucket".to_string(),
    ///         param: InsertBucketParam {
    ///             project: "project_id".to_string(),
    ///             ..Default::default()
    ///         },
    ///         ..Default::default()
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn insert_bucket(&self, req: &InsertBucketRequest) -> Result<Bucket, Error> {
        let builder = buckets::insert::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
    }

    /// Gets the bucket.
    /// https://cloud.google.com/storage/docs/json_api/v1/buckets/get
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::buckets::get::GetBucketRequest;
    ///
    /// async fn run(client:Client) {
    ///     let result = client.get_bucket(&GetBucketRequest {
    ///         bucket: "bucket".to_string(),
    ///         ..Default::default()
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_bucket(&self, req: &GetBucketRequest) -> Result<Bucket, Error> {
        let builder = buckets::get::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
    }

    /// Patches the bucket.
    /// https://cloud.google.com/storage/docs/json_api/v1/buckets/patch
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::client::ClientConfig;
    /// use google_cloud_storage::http::buckets::patch::{BucketPatchConfig, PatchBucketRequest};
    ///
    /// async fn run(config: ClientConfig) {
    ///     let mut client = Client::new(config);
    ///
    ///     let result = client.patch_bucket(&PatchBucketRequest {
    ///         bucket: "bucket".to_string(),
    ///         metadata: Some(BucketPatchConfig {
    ///             ..Default::default()
    ///         }),
    ///         ..Default::default()
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn patch_bucket(&self, req: &PatchBucketRequest) -> Result<Bucket, Error> {
        let builder = buckets::patch::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
    }

    /// Lists the bucket.
    /// https://cloud.google.com/storage/docs/json_api/v1/buckets/list
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::buckets::list::ListBucketsRequest;
    ///
    /// async fn run(client:Client) {
    ///     let result = client.list_buckets(&ListBucketsRequest{
    ///         project: "project_id".to_string(),
    ///         ..Default::default()
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn list_buckets(&self, req: &ListBucketsRequest) -> Result<ListBucketsResponse, Error> {
        let builder = buckets::list::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
    }

    /// Sets the iam policy.
    /// https://cloud.google.com/storage/docs/json_api/v1/buckets/setIamPolicy
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::buckets::{Binding, Policy};
    /// use google_cloud_storage::http::buckets::set_iam_policy::SetIamPolicyRequest;
    ///
    /// async fn run(client:Client) {
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
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn set_iam_policy(&self, req: &SetIamPolicyRequest) -> Result<Policy, Error> {
        let builder = buckets::set_iam_policy::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
    }

    /// Gets the iam policy.
    /// https://cloud.google.com/storage/docs/json_api/v1/buckets/getIamPolicy
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::buckets::get_iam_policy::GetIamPolicyRequest;
    /// use google_cloud_storage::http::buckets::list::ListBucketsRequest;
    ///
    /// async fn run(client:Client) {
    ///     let result = client.get_iam_policy(&GetIamPolicyRequest{
    ///         resource: "bucket".to_string(),
    ///         ..Default::default()
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_iam_policy(&self, req: &GetIamPolicyRequest) -> Result<Policy, Error> {
        let builder = buckets::get_iam_policy::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
    }

    /// Tests the iam permissions.
    /// https://cloud.google.com/storage/docs/json_api/v1/buckets/testIamPermissions
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::buckets::test_iam_permissions::TestIamPermissionsRequest;
    ///
    /// async fn run(client:Client) {
    ///     let result = client.test_iam_permissions(&TestIamPermissionsRequest{
    ///         resource: "bucket".to_string(),
    ///         permissions: vec!["storage.buckets.get".to_string()],
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn test_iam_permissions(
        &self,
        req: &TestIamPermissionsRequest,
    ) -> Result<TestIamPermissionsResponse, Error> {
        let builder = buckets::test_iam_permissions::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
    }

    /// Lists the default object ACL.
    /// https://cloud.google.com/storage/docs/json_api/v1/defaultObjectAccessControls/list
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::buckets::test_iam_permissions::TestIamPermissionsRequest;
    /// use google_cloud_storage::http::default_object_access_controls::list::ListDefaultObjectAccessControlsRequest;
    ///
    /// async fn run(client:Client) {
    ///     let result = client.list_default_object_access_controls(&ListDefaultObjectAccessControlsRequest{
    ///         bucket: "bucket".to_string(),
    ///         ..Default::default()
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn list_default_object_access_controls(
        &self,
        req: &ListDefaultObjectAccessControlsRequest,
    ) -> Result<ListDefaultObjectAccessControlsResponse, Error> {
        let builder = default_object_access_controls::list::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
    }

    /// Gets the default object ACL.
    /// https://cloud.google.com/storage/docs/json_api/v1/defaultObjectAccessControls/get
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::default_object_access_controls::get::GetDefaultObjectAccessControlRequest;
    ///
    /// async fn run(client:Client) {
    ///     let result = client.get_default_object_access_control(&GetDefaultObjectAccessControlRequest{
    ///         bucket: "bucket".to_string(),
    ///         entity: "allAuthenticatedUsers".to_string(),
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_default_object_access_control(
        &self,
        req: &GetDefaultObjectAccessControlRequest,
    ) -> Result<ObjectAccessControl, Error> {
        let builder = default_object_access_controls::get::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
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
    /// async fn run(client:Client) {
    ///     let result = client.insert_default_object_access_control(&InsertDefaultObjectAccessControlRequest{
    ///         bucket: "bucket".to_string(),
    ///         object_access_control: ObjectAccessControlCreationConfig {
    ///             entity: "allAuthenticatedUsers".to_string(),
    ///             role: ObjectACLRole::READER
    ///         } ,
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn insert_default_object_access_control(
        &self,
        req: &InsertDefaultObjectAccessControlRequest,
    ) -> Result<ObjectAccessControl, Error> {
        let builder = default_object_access_controls::insert::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
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
    /// async fn run(client:Client) {
    ///     let result = client.patch_default_object_access_control(&PatchDefaultObjectAccessControlRequest{
    ///         bucket: "bucket".to_string(),
    ///         entity: "allAuthenticatedUsers".to_string(),
    ///         object_access_control: ObjectAccessControl {
    ///             role: ObjectACLRole::READER,
    ///             ..Default::default()
    ///         },
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn patch_default_object_access_control(
        &self,
        req: &PatchDefaultObjectAccessControlRequest,
    ) -> Result<ObjectAccessControl, Error> {
        let builder = default_object_access_controls::patch::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
    }

    /// Deletes the default object ACL.
    /// https://cloud.google.com/storage/docs/json_api/v1/defaultObjectAccessControls/delete
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::default_object_access_controls::delete::DeleteDefaultObjectAccessControlRequest;
    ///
    /// async fn run(client:Client) {
    ///     let result = client.delete_default_object_access_control(&DeleteDefaultObjectAccessControlRequest{
    ///         bucket: "bucket".to_string(),
    ///         entity: "allAuthenticatedUsers".to_string(),
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn delete_default_object_access_control(
        &self,
        req: &DeleteDefaultObjectAccessControlRequest,
    ) -> Result<(), Error> {
        let builder = default_object_access_controls::delete::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send_get_empty(builder).await
    }

    /// Lists the bucket ACL.
    /// https://cloud.google.com/storage/docs/json_api/v1/bucketAccessControls/list
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::bucket_access_controls::list::ListBucketAccessControlsRequest;
    ///
    /// async fn run(client:Client) {
    ///     let result = client.list_bucket_access_controls(&ListBucketAccessControlsRequest{
    ///         bucket: "bucket".to_string(),
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn list_bucket_access_controls(
        &self,
        req: &ListBucketAccessControlsRequest,
    ) -> Result<ListBucketAccessControlsResponse, Error> {
        let builder = bucket_access_controls::list::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
    }

    /// Gets the bucket ACL.
    /// https://cloud.google.com/storage/docs/json_api/v1/bucketAccessControls/get
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::bucket_access_controls::get::GetBucketAccessControlRequest;
    ///
    /// async fn run(client:Client) {
    ///     let result = client.get_bucket_access_control(&GetBucketAccessControlRequest{
    ///         bucket: "bucket".to_string(),
    ///         entity: "allAuthenticatedUsers".to_string(),
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_bucket_access_control(
        &self,
        req: &GetBucketAccessControlRequest,
    ) -> Result<BucketAccessControl, Error> {
        let builder = bucket_access_controls::get::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
    }

    /// Inserts the bucket ACL.
    /// https://cloud.google.com/storage/docs/json_api/v1/bucketAccessControls/insert
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::bucket_access_controls::BucketACLRole;
    /// use google_cloud_storage::http::bucket_access_controls::insert::{BucketAccessControlCreationConfig, InsertBucketAccessControlRequest};
    ///
    /// async fn run(client:Client) {
    ///     let result = client.insert_bucket_access_control(&InsertBucketAccessControlRequest{
    ///         bucket: "bucket".to_string(),
    ///         acl: BucketAccessControlCreationConfig {
    ///             entity: "allAuthenticatedUsers".to_string(),
    ///             role: BucketACLRole::READER
    ///         }
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn insert_bucket_access_control(
        &self,
        req: &InsertBucketAccessControlRequest,
    ) -> Result<BucketAccessControl, Error> {
        let builder = bucket_access_controls::insert::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
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
    /// async fn run(client:Client) {
    ///     let result = client.patch_bucket_access_control(&PatchBucketAccessControlRequest{
    ///         bucket: "bucket".to_string(),
    ///         entity: "allAuthenticatedUsers".to_string(),
    ///         acl: BucketAccessControl {
    ///             role: BucketACLRole::READER,
    ///             ..Default::default()
    ///         }
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn patch_bucket_access_control(
        &self,
        req: &PatchBucketAccessControlRequest,
    ) -> Result<BucketAccessControl, Error> {
        let builder = bucket_access_controls::patch::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
    }

    /// Deletes the bucket ACL.
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::bucket_access_controls::BucketAccessControl;
    /// use google_cloud_storage::http::bucket_access_controls::delete::DeleteBucketAccessControlRequest;
    ///
    /// async fn run(client:Client) {
    ///     let result = client.delete_bucket_access_control(&DeleteBucketAccessControlRequest{
    ///         bucket: "bucket".to_string(),
    ///         entity: "allAuthenticatedUsers".to_string(),
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn delete_bucket_access_control(&self, req: &DeleteBucketAccessControlRequest) -> Result<(), Error> {
        let builder = bucket_access_controls::delete::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send_get_empty(builder).await
    }

    /// Lists the object ACL.
    /// https://cloud.google.com/storage/docs/json_api/v1/objectAccessControls/list
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::object_access_controls::list::ListObjectAccessControlsRequest;
    ///
    /// async fn run(client:Client) {
    ///     let result = client.list_object_access_controls(&ListObjectAccessControlsRequest{
    ///         bucket: "bucket".to_string(),
    ///         object: "filename".to_string(),
    ///         ..Default::default()
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn list_object_access_controls(
        &self,
        req: &ListObjectAccessControlsRequest,
    ) -> Result<ListBucketAccessControlsResponse, Error> {
        let builder = object_access_controls::list::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
    }

    /// Gets the object ACL.
    /// https://cloud.google.com/storage/docs/json_api/v1/objectAccessControls/get
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::object_access_controls::get::GetObjectAccessControlRequest;
    ///
    ///
    /// async fn run(client:Client) {
    ///
    ///     let result = client.get_object_access_control(&GetObjectAccessControlRequest{
    ///         bucket: "bucket".to_string(),
    ///         object: "filename".to_string(),
    ///         entity: "allAuthenticatedUsers".to_string(),
    ///         ..Default::default()
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_object_access_control(
        &self,
        req: &GetObjectAccessControlRequest,
    ) -> Result<ObjectAccessControl, Error> {
        let builder = object_access_controls::get::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
    }

    /// Inserts the object ACL.
    /// https://cloud.google.com/storage/docs/json_api/v1/objectAccessControls/insert
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::object_access_controls::insert::{InsertObjectAccessControlRequest, ObjectAccessControlCreationConfig};
    /// use google_cloud_storage::http::object_access_controls::ObjectACLRole;
    ///
    ///
    /// async fn run(client:Client) {
    ///
    ///     let result = client.insert_object_access_control(&InsertObjectAccessControlRequest{
    ///         bucket: "bucket".to_string(),
    ///         object: "filename".to_string(),
    ///         acl: ObjectAccessControlCreationConfig {
    ///             entity: "allAuthenticatedUsers".to_string(),
    ///             role: ObjectACLRole::READER
    ///         },
    ///         ..Default::default()
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn insert_object_access_control(
        &self,
        req: &InsertObjectAccessControlRequest,
    ) -> Result<ObjectAccessControl, Error> {
        let builder = object_access_controls::insert::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
    }

    /// Patches the bucket ACL.
    /// https://cloud.google.com/storage/docs/json_api/v1/objectAccessControls/patch
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::object_access_controls::{ObjectAccessControl, ObjectACLRole};
    /// use google_cloud_storage::http::object_access_controls::patch::PatchObjectAccessControlRequest;
    ///
    ///
    /// async fn run(client:Client) {
    ///
    ///     let result = client.patch_object_access_control(&PatchObjectAccessControlRequest{
    ///         bucket: "bucket".to_string(),
    ///         object: "filename".to_string(),
    ///         entity: "allAuthenticatedUsers".to_string(),
    ///         acl: ObjectAccessControl {
    ///             role: ObjectACLRole::READER,
    ///             ..Default::default()
    ///         },
    ///         ..Default::default()
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn patch_object_access_control(
        &self,
        req: &PatchObjectAccessControlRequest,
    ) -> Result<ObjectAccessControl, Error> {
        let builder = object_access_controls::patch::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
    }

    /// Deletes the bucket ACL.
    /// https://cloud.google.com/storage/docs/json_api/v1/objectAccessControls/delete
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::object_access_controls::{ObjectAccessControl, ObjectACLRole};
    /// use google_cloud_storage::http::object_access_controls::delete::DeleteObjectAccessControlRequest;
    ///
    /// async fn run(client:Client) {
    ///     let result = client.delete_object_access_control(&DeleteObjectAccessControlRequest{
    ///         bucket: "bucket".to_string(),
    ///         object: "filename".to_string(),
    ///         entity: "allAuthenticatedUsers".to_string(),
    ///         ..Default::default()
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn delete_object_access_control(&self, req: &DeleteObjectAccessControlRequest) -> Result<(), Error> {
        let builder = object_access_controls::delete::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send_get_empty(builder).await
    }

    /// Lists the notification.
    /// https://cloud.google.com/storage/docs/json_api/v1/notifications/list
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::notifications::list::ListNotificationsRequest;
    ///
    /// async fn run(client:Client) {
    ///     let result = client.list_notifications(&ListNotificationsRequest{
    ///         bucket: "bucket".to_string(),
    ///         ..Default::default()
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn list_notifications(&self, req: &ListNotificationsRequest) -> Result<ListNotificationsResponse, Error> {
        let builder = notifications::list::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
    }

    /// Gets the notification.
    /// https://cloud.google.com/storage/docs/json_api/v1/notifications/get
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::notifications::get::GetNotificationRequest;
    ///
    ///
    /// async fn run(client:Client) {
    ///
    ///     let result = client.get_notification(&GetNotificationRequest{
    ///         bucket: "bucket".to_string(),
    ///         notification: "notification".to_string()
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_notification(&self, req: &GetNotificationRequest) -> Result<Notification, Error> {
        let builder = notifications::get::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
    }

    /// Inserts the notification.
    /// https://cloud.google.com/storage/docs/json_api/v1/notifications/insert
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::notifications::EventType;
    /// use google_cloud_storage::http::notifications::insert::{InsertNotificationRequest, NotificationCreationConfig};
    ///
    ///
    /// async fn run(client:Client) {
    ///
    ///     let result = client.insert_notification(&InsertNotificationRequest {
    ///         bucket: "bucket".to_string(),
    ///         notification: NotificationCreationConfig {
    ///             topic: format!("projects/{}/topics/{}", "project","bucket"),
    ///             event_types: Some(vec![EventType::ObjectMetadataUpdate, EventType::ObjectDelete]),
    ///             ..Default::default()
    ///         }
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn insert_notification(&self, req: &InsertNotificationRequest) -> Result<Notification, Error> {
        let builder = notifications::insert::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
    }

    /// Deletes the notification.
    /// https://cloud.google.com/storage/docs/json_api/v1/notifications/delete
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::notifications::delete::DeleteNotificationRequest;
    ///
    ///
    /// async fn run(client:Client) {
    ///
    ///     let result = client.delete_notification(&DeleteNotificationRequest {
    ///         bucket: "bucket".to_string(),
    ///         notification: "notification".to_string()
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn delete_notification(&self, req: &DeleteNotificationRequest) -> Result<(), Error> {
        let builder = notifications::delete::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send_get_empty(builder).await
    }

    /// Lists the hmac keys.
    /// https://cloud.google.com/storage/docs/json_api/v1/projects/hmacKeys/list
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::hmac_keys::list::ListHmacKeysRequest;
    ///
    ///
    /// async fn run(client:Client) {
    ///
    ///     let result = client.list_hmac_keys(&ListHmacKeysRequest {
    ///         project_id: "project_id".to_string(),
    ///         ..Default::default()
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn list_hmac_keys(&self, req: &ListHmacKeysRequest) -> Result<ListHmacKeysResponse, Error> {
        let builder = hmac_keys::list::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
    }

    /// Gets the hmac keys.
    /// https://cloud.google.com/storage/docs/json_api/v1/projects/hmacKeys/get
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::hmac_keys::get::GetHmacKeyRequest;
    ///
    ///
    /// async fn run(client:Client) {
    ///
    ///     let result = client.get_hmac_key(&GetHmacKeyRequest {
    ///         access_id: "access_id".to_string(),
    ///         project_id: "project_id".to_string(),
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_hmac_key(&self, req: &GetHmacKeyRequest) -> Result<HmacKeyMetadata, Error> {
        let builder = hmac_keys::get::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
    }

    /// Creates the hmac key.
    /// https://cloud.google.com/storage/docs/json_api/v1/projects/hmacKeys/create
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::hmac_keys::create::CreateHmacKeyRequest;
    ///
    ///
    /// async fn run(client:Client) {
    ///
    ///     let result = client.create_hmac_key(&CreateHmacKeyRequest {
    ///         service_account_email: "service_account_email".to_string(),
    ///         project_id: "project".to_string(),
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn create_hmac_key(&self, req: &CreateHmacKeyRequest) -> Result<CreateHmacKeyResponse, Error> {
        let builder = hmac_keys::create::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
    }

    /// Updates the hmac key.
    /// https://cloud.google.com/storage/docs/json_api/v1/projects/hmacKeys/update
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::hmac_keys::HmacKeyMetadata;
    /// use google_cloud_storage::http::hmac_keys::update::UpdateHmacKeyRequest;
    ///
    ///
    /// async fn run(client:Client) {
    ///
    ///     let result = client.update_hmac_key(&UpdateHmacKeyRequest{
    ///         access_id: "access_id".to_string(),
    ///         project_id: "project_id".to_string(),
    ///         metadata: HmacKeyMetadata {
    ///             state: "INACTIVE".to_string(),
    ///             ..Default::default()
    ///         },
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn update_hmac_key(&self, req: &UpdateHmacKeyRequest) -> Result<HmacKeyMetadata, Error> {
        let builder = hmac_keys::update::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
    }

    /// Deletes the hmac key.
    /// https://cloud.google.com/storage/docs/json_api/v1/projects/hmacKeys/delete
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::hmac_keys::delete::DeleteHmacKeyRequest;
    ///
    ///
    /// async fn run(client:Client) {
    ///
    ///     let result = client.delete_hmac_key(&DeleteHmacKeyRequest{
    ///         access_id: "access_id".to_string(),
    ///         project_id:"project_id".to_string(),
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn delete_hmac_key(&self, req: &DeleteHmacKeyRequest) -> Result<(), Error> {
        let builder = hmac_keys::delete::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send_get_empty(builder).await
    }

    /// Lists the objects.
    /// https://cloud.google.com/storage/docs/json_api/v1/objects/list
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::objects::list::ListObjectsRequest;
    ///
    ///
    /// async fn run(client:Client) {
    ///
    ///     let result = client.list_objects(&ListObjectsRequest{
    ///         bucket: "bucket".to_string(),
    ///         ..Default::default()
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn list_objects(&self, req: &ListObjectsRequest) -> Result<ListObjectsResponse, Error> {
        let builder = objects::list::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
    }

    /// Gets the object.
    /// https://cloud.google.com/storage/docs/json_api/v1/objects/get
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::objects::get::GetObjectRequest;
    ///
    /// async fn run(client:Client) {
    ///     let result = client.get_object(&GetObjectRequest{
    ///         bucket: "bucket".to_string(),
    ///         object: "object".to_string(),
    ///         ..Default::default()
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_object(&self, req: &GetObjectRequest) -> Result<Object, Error> {
        let builder = objects::get::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
    }

    /// Copy the object.
    /// https://cloud.google.com/storage/docs/json_api/v1/objects/copy
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::objects::copy::CopyObjectRequest;
    ///
    /// async fn run(client:Client) {
    ///     let result = client.copy_object(&CopyObjectRequest{
    ///         source_bucket: "bucket".to_string(),
    ///         destination_bucket: "bucket".to_string(),
    ///         destination_object: "object".to_string(),
    ///         source_object: "object".to_string(),
    ///         ..Default::default()
    ///     }).await;
    /// }
    /// ```
    ///
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn copy_object(&self, req: &CopyObjectRequest) -> Result<Object, Error> {
        let builder = objects::copy::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
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
    ///
    /// async fn run(client:Client) {
    ///
    ///     let result = client.download_object(&GetObjectRequest{
    ///         bucket: "bucket".to_string(),
    ///         object: "object".to_string(),
    ///         ..Default::default()
    ///     }, &Range::default()).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn download_object(&self, req: &GetObjectRequest, range: &Range) -> Result<Vec<u8>, Error> {
        let builder = objects::download::build(self.v1_endpoint.as_str(), &self.http, req, range);
        let request = self.with_headers(builder).await?;
        let response = request.send().await?;
        let response = check_response_status(response).await?;
        Ok(response.bytes().await?.to_vec())
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
    /// async fn run(client:Client) {
    ///     let result = client.download_streamed_object(&GetObjectRequest{
    ///         bucket: "bucket".to_string(),
    ///         object: "object".to_string(),
    ///         ..Default::default()
    ///     }, &Range::default()).await;
    ///
    ///     //  while let Some(v) = downloaded.next().await? {
    ///     //      let d: bytes::Bytes = v.unwrap();
    ///     //  }
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn download_streamed_object(
        &self,
        req: &GetObjectRequest,
        range: &Range,
    ) -> Result<impl Stream<Item = Result<bytes::Bytes, Error>>, Error> {
        let builder = objects::download::build(self.v1_endpoint.as_str(), &self.http, req, range);
        let request = self.with_headers(builder).await?;
        let response = request.send().await?;
        let response = check_response_status(response).await?;
        Ok(response.bytes_stream().map_err(Error::from))
    }

    /// Uploads the object.
    /// https://cloud.google.com/storage/docs/json_api/v1/objects/insert
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::objects::Object;
    /// use google_cloud_storage::http::objects::upload::{Media, UploadObjectRequest, UploadType};
    ///
    /// async fn run_simple(client:Client) {
    ///     let upload_type = UploadType::Simple(Media::new("filename"));
    ///     let result = client.upload_object(&UploadObjectRequest{
    ///         bucket: "bucket".to_string(),
    ///         ..Default::default()
    ///     }, "hello world".as_bytes(), &upload_type).await;
    /// }
    ///
    /// async fn run_multipart(client:Client) {
    ///     let mut metadata = HashMap::<String, String>::new();
    ///     metadata.insert("key1".to_string(), "value1".to_string());
    ///     let upload_type = UploadType::Multipart(Box::new(Object {
    ///         name: "test1_meta".to_string(),
    ///         content_type: Some("text/plain".to_string()),
    ///         metadata: Some(metadata),
    ///         ..Default::default()
    ///     }));
    ///     let result = client.upload_object(&UploadObjectRequest{
    ///         bucket: "bucket".to_string(),
    ///         ..Default::default()
    ///     }, "hello world".as_bytes(), &upload_type).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn upload_object<T: Into<Body>>(
        &self,
        req: &UploadObjectRequest,
        data: T,
        upload_type: &UploadType,
    ) -> Result<Object, Error> {
        match upload_type {
            UploadType::Multipart(meta) => {
                let builder =
                    objects::upload::build_multipart(self.v1_upload_endpoint.as_str(), &self.http, req, meta, data)?;
                self.send(builder).await
            }
            UploadType::Simple(media) => {
                let builder = objects::upload::build(self.v1_upload_endpoint.as_str(), &self.http, req, media, data);
                self.send(builder).await
            }
        }
    }

    /// Creates resumable upload from known URL.
    ///
    /// Assumes URL is correct, if not, `ResumableUploadClient` is not guaranteed to perform correctly.
    pub fn get_resumable_upload(&self, url: String) -> ResumableUploadClient {
        ResumableUploadClient::new(url, self.http.clone())
    }

    /// Perform resumable uploads
    /// https://cloud.google.com/storage/docs/performing-resumable-uploads
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::objects::Object;
    /// use google_cloud_storage::http::objects::upload::{Media, UploadObjectRequest, UploadType};
    /// use google_cloud_storage::http::resumable_upload_client::{ChunkSize, UploadStatus};
    ///
    /// async fn run_simple(client:Client) {
    ///     let upload_type = UploadType::Simple(Media::new("filename"));
    ///     let uploader = client.prepare_resumable_upload(&UploadObjectRequest{
    ///         bucket: "bucket".to_string(),
    ///         ..Default::default()
    ///     }, &upload_type).await.unwrap();
    ///
    ///     // We can also use upload_multiple_chunk.
    ///     let data = [1,2,3,4,5];
    ///     let result = uploader.upload_single_chunk(Vec::from(data), data.len()).await;
    /// }
    ///
    /// async fn run_with_metadata(client:Client) {
    ///     let mut metadata = HashMap::<String, String>::new();
    ///     metadata.insert("key1".to_string(), "value1".to_string());
    ///     let upload_type = UploadType::Multipart(Box::new(Object {
    ///         name: "test1_meta".to_string(),
    ///         content_type: Some("text/plain".to_string()),
    ///         metadata: Some(metadata),
    ///         ..Default::default()
    ///     }));
    ///     let uploader = client.prepare_resumable_upload(&UploadObjectRequest{
    ///         bucket: "bucket".to_string(),
    ///         ..Default::default()
    ///     }, &upload_type).await.unwrap();
    ///
    ///     let chunk1_data : Vec<u8>= (0..256 * 1024).map(|i| (i % 256) as u8).collect();
    ///     let chunk2_data : Vec<u8>= (1..256 * 1024 + 50).map(|i| (i % 256) as u8).collect();
    ///     let chunk1_size = chunk1_data.len() as u64;
    ///     let chunk2_size = chunk2_data.len() as u64;
    ///     let total_size = Some(chunk1_size + chunk2_size);
    ///
    ///     // The chunk size should be multiple of 256KiB, unless it's the last chunk that completes the upload.
    ///     let chunk1 = ChunkSize::new(0, chunk1_size - 1, total_size.clone());
    ///     let status1 = uploader.upload_multiple_chunk(chunk1_data.clone(), &chunk1).await.unwrap();
    ///     assert_eq!(status1, UploadStatus::ResumeIncomplete);
    ///
    ///     let chunk2 = ChunkSize::new(chunk1_size, chunk1_size + chunk2_size - 1, total_size.clone());
    ///     let status2 = uploader.upload_multiple_chunk(chunk2_data.clone(), &chunk2).await.unwrap();
    ///     assert!(matches!(status2, UploadStatus::Ok(_)));
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn prepare_resumable_upload(
        &self,
        req: &UploadObjectRequest,
        upload_type: &UploadType,
    ) -> Result<ResumableUploadClient, Error> {
        let request = match upload_type {
            UploadType::Multipart(meta) => objects::upload::build_resumable_session_metadata(
                self.v1_upload_endpoint.as_str(),
                &self.http,
                req,
                meta,
            ),
            UploadType::Simple(media) => objects::upload::build_resumable_session_simple(
                self.v1_upload_endpoint.as_str(),
                &self.http,
                req,
                media,
            ),
        };
        self.send_get_url(request)
            .await
            .map(|url| ResumableUploadClient::new(url, self.http.clone()))
    }

    /// Uploads the streamed object.
    /// https://cloud.google.com/storage/docs/json_api/v1/objects/insert
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::objects::upload::{Media, UploadObjectRequest, UploadType};
    ///
    /// async fn run(client:Client) {
    ///     let source = vec!["hello", " ", "world"];
    ///     let size = source.iter().map(|x| x.len() as u64).sum();
    ///     let chunks: Vec<Result<_, ::std::io::Error>> = source.clone().into_iter().map(|x| Ok(x)).collect();
    ///     let stream = futures_util::stream::iter(chunks);
    ///     let mut media = Media::new("filename");
    ///     media.content_length = Some(size);
    ///     let mut upload_type = UploadType::Simple(media);
    ///     let result = client.upload_streamed_object(&UploadObjectRequest{
    ///         bucket: "bucket".to_string(),
    ///         ..Default::default()
    ///     }, stream, &upload_type).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn upload_streamed_object<S>(
        &self,
        req: &UploadObjectRequest,
        data: S,
        upload_type: &UploadType,
    ) -> Result<Object, Error>
    where
        S: TryStream + Send + Sync + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        bytes::Bytes: From<S::Ok>,
    {
        //TODO resumable upload
        self.upload_object(req, Body::wrap_stream(data), upload_type).await
    }

    /// Patches the object.
    /// https://cloud.google.com/storage/docs/json_api/v1/objects/patch
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::objects::patch::PatchObjectRequest;
    ///
    ///
    /// async fn run(client:Client) {
    ///
    ///     let result = client.patch_object(&PatchObjectRequest{
    ///         bucket: "bucket".to_string(),
    ///         object: "object".to_string(),
    ///         ..Default::default()
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn patch_object(&self, req: &PatchObjectRequest) -> Result<Object, Error> {
        let builder = objects::patch::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
    }

    /// Deletes the object.
    /// https://cloud.google.com/storage/docs/json_api/v1/objects/delete
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::objects::delete::DeleteObjectRequest;
    ///
    ///
    /// async fn run(client:Client) {
    ///
    ///     let result = client.delete_object(&DeleteObjectRequest{
    ///         bucket: "bucket".to_string(),
    ///         object: "object".to_string(),
    ///         ..Default::default()
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn delete_object(&self, req: &DeleteObjectRequest) -> Result<(), Error> {
        let builder = objects::delete::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send_get_empty(builder).await
    }

    /// Rewrites the object.
    /// https://cloud.google.com/storage/docs/json_api/v1/objects/rewrite
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::http::objects::rewrite::RewriteObjectRequest;
    ///
    ///
    /// async fn run(client:Client) {
    ///     let mut done = false;
    ///     let mut rewrite_token = None;
    ///
    ///     while !done {
    ///         let result = client.rewrite_object(&RewriteObjectRequest{
    ///             source_bucket: "bucket1".to_string(),
    ///             source_object: "object".to_string(),
    ///             destination_bucket: "bucket2".to_string(),
    ///             destination_object: "object1".to_string(),
    ///             rewrite_token: rewrite_token.clone(),
    ///             ..Default::default()
    ///         }).await.unwrap();
    ///
    ///         done = result.done;
    ///         rewrite_token = result.rewrite_token;
    ///     }
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn rewrite_object(&self, req: &RewriteObjectRequest) -> Result<RewriteObjectResponse, Error> {
        let builder = objects::rewrite::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
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
    /// async fn run(client:Client) {
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
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn compose_object(&self, req: &ComposeObjectRequest) -> Result<Object, Error> {
        let builder = objects::compose::build(self.v1_endpoint.as_str(), &self.http, req);
        self.send(builder).await
    }

    async fn with_headers(&self, builder: RequestBuilder) -> Result<RequestBuilder, Error> {
        let token = self.ts.token().await.map_err(Error::TokenSource)?;
        Ok(builder
            .header("X-Goog-Api-Client", "rust")
            .header(reqwest::header::USER_AGENT, "google-cloud-storage")
            .header(reqwest::header::AUTHORIZATION, token))
    }

    async fn send<T>(&self, builder: RequestBuilder) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let request = self.with_headers(builder).await?;
        let response = request.send().await?;
        let response = check_response_status(response).await?;
        Ok(response.json().await?)
    }

    async fn send_get_empty(&self, builder: RequestBuilder) -> Result<(), Error> {
        let builder = self.with_headers(builder).await?;
        let response = builder.send().await?;
        check_response_status(response).await?;
        Ok(())
    }

    async fn send_get_url(&self, builder: RequestBuilder) -> Result<String, Error> {
        let builder = self.with_headers(builder).await?;
        let response = builder.send().await?;
        let response = check_response_status(response).await?;
        Ok(String::from_utf8_lossy(response.headers()[LOCATION].as_bytes()).into_owned())
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use bytes::Buf;
    use futures_util::StreamExt;
    use serial_test::serial;

    use google_cloud_auth::project::Config;
    use google_cloud_auth::token::DefaultTokenSourceProvider;
    use google_cloud_token::TokenSourceProvider;

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
    use crate::http::notifications::EventType;
    use crate::http::object_access_controls::delete::DeleteObjectAccessControlRequest;
    use crate::http::object_access_controls::get::GetObjectAccessControlRequest;
    use crate::http::object_access_controls::insert::{
        InsertObjectAccessControlRequest, ObjectAccessControlCreationConfig,
    };
    use crate::http::object_access_controls::list::ListObjectAccessControlsRequest;
    use crate::http::object_access_controls::ObjectACLRole;
    use crate::http::objects::compose::{ComposeObjectRequest, ComposingTargets};
    use crate::http::objects::copy::CopyObjectRequest;
    use crate::http::objects::delete::DeleteObjectRequest;
    use crate::http::objects::download::Range;
    use crate::http::objects::get::GetObjectRequest;
    use crate::http::objects::list::ListObjectsRequest;
    use crate::http::objects::rewrite::RewriteObjectRequest;
    use crate::http::objects::upload::{Media, UploadObjectRequest, UploadType};
    use crate::http::objects::{Object, SourceObjects};
    use crate::http::resumable_upload_client::{ChunkSize, UploadStatus};
    use crate::http::storage_client::{StorageClient, SCOPES};

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt::try_init();
    }

    async fn client() -> (StorageClient, String) {
        let tsp = DefaultTokenSourceProvider::new(Config {
            audience: None,
            scopes: Some(&SCOPES),
        })
        .await
        .unwrap();
        let cred = tsp.source_credentials.clone();
        let ts = tsp.token_source();
        let client = StorageClient::new(ts, "https://storage.googleapis.com", reqwest::Client::new());
        (client, cred.unwrap().project_id.unwrap())
    }

    #[tokio::test]
    #[serial]
    pub async fn list_buckets() {
        let (client, project) = client().await;
        let buckets = client
            .list_buckets(&ListBucketsRequest {
                project,
                max_results: None,
                page_token: None,
                prefix: Some("rust-iam-test".to_string()),
                projection: None,
            })
            .await
            .unwrap();
        assert_eq!(1, buckets.items.len());
    }

    #[tokio::test]
    #[serial]
    pub async fn crud_bucket() {
        let (client, project) = client().await;
        let name = format!("rust-test-insert-{}", time::OffsetDateTime::now_utc().unix_timestamp());
        let bucket = client
            .insert_bucket(&InsertBucketRequest {
                name,
                param: InsertBucketParam {
                    project,
                    ..Default::default()
                },
                bucket: BucketCreationConfig {
                    location: "ASIA-NORTHEAST1".to_string(),
                    storage_class: Some("STANDARD".to_string()),
                    ..Default::default()
                },
            })
            .await
            .unwrap();

        let found = client
            .get_bucket(&GetBucketRequest {
                bucket: bucket.name.to_string(),
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(found.location.as_str(), "ASIA-NORTHEAST1");

        let patched = client
            .patch_bucket(&PatchBucketRequest {
                bucket: bucket.name.to_string(),
                metadata: Some(BucketPatchConfig {
                    default_object_acl: Some(vec![ObjectAccessControlCreationConfig {
                        entity: "allAuthenticatedUsers".to_string(),
                        role: ObjectACLRole::READER,
                    }]),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .await
            .unwrap();

        let default_object_acl = patched.default_object_acl.unwrap();
        assert_eq!(default_object_acl.len(), 1);
        assert_eq!(default_object_acl[0].entity.as_str(), "allAuthenticatedUsers");
        assert_eq!(default_object_acl[0].role, ObjectACLRole::READER);
        assert_eq!(found.storage_class.as_str(), patched.storage_class.as_str());
        assert_eq!(found.location.as_str(), patched.location.as_str());

        client
            .delete_bucket(&DeleteBucketRequest {
                bucket: bucket.name,
                param: Default::default(),
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn set_get_test_iam() {
        let bucket_name = "rust-iam-test";
        let (client, _project) = client().await;
        let mut policy = client
            .get_iam_policy(&GetIamPolicyRequest {
                resource: bucket_name.to_string(),
                options_requested_policy_version: None,
            })
            .await
            .unwrap();
        policy.bindings.push(Binding {
            role: "roles/storage.objectViewer".to_string(),
            members: vec!["allAuthenticatedUsers".to_string()],
            condition: None,
        });

        let mut result = client
            .set_iam_policy(&SetIamPolicyRequest {
                resource: bucket_name.to_string(),
                policy,
            })
            .await
            .unwrap();
        assert_eq!(result.bindings.len(), 5);
        assert_eq!(result.bindings.pop().unwrap().role, "roles/storage.objectViewer");

        let permissions = client
            .test_iam_permissions(&TestIamPermissionsRequest {
                resource: bucket_name.to_string(),
                permissions: vec!["storage.buckets.get".to_string()],
            })
            .await
            .unwrap();
        assert_eq!(permissions.permissions[0], "storage.buckets.get");
    }

    #[tokio::test]
    #[serial]
    pub async fn crud_default_object_controls() {
        let bucket_name = "rust-default-object-acl-test";
        let (client, _project) = client().await;

        client
            .delete_default_object_access_control(&DeleteDefaultObjectAccessControlRequest {
                bucket: bucket_name.to_string(),
                entity: "allAuthenticatedUsers".to_string(),
            })
            .await
            .unwrap();

        let _post = client
            .insert_default_object_access_control(&InsertDefaultObjectAccessControlRequest {
                bucket: bucket_name.to_string(),
                object_access_control: ObjectAccessControlCreationConfig {
                    entity: "allAuthenticatedUsers".to_string(),
                    role: ObjectACLRole::READER,
                },
            })
            .await
            .unwrap();

        let found = client
            .get_default_object_access_control(&GetDefaultObjectAccessControlRequest {
                bucket: bucket_name.to_string(),
                entity: "allAuthenticatedUsers".to_string(),
            })
            .await
            .unwrap();
        assert_eq!(found.entity, "allAuthenticatedUsers");
        assert_eq!(found.role, ObjectACLRole::READER);

        let acls = client
            .list_default_object_access_controls(&ListDefaultObjectAccessControlsRequest {
                bucket: bucket_name.to_string(),
                ..Default::default()
            })
            .await
            .unwrap();
        assert!(acls.items.is_some());
        assert_eq!(1, acls.items.unwrap().len());
    }

    #[tokio::test]
    #[serial]
    pub async fn crud_bucket_access_controls() {
        let bucket_name = "rust-bucket-acl-test";
        let (client, _project) = client().await;

        let _post = client
            .insert_bucket_access_control(&InsertBucketAccessControlRequest {
                bucket: bucket_name.to_string(),
                acl: BucketAccessControlCreationConfig {
                    entity: "allAuthenticatedUsers".to_string(),
                    role: BucketACLRole::READER,
                },
            })
            .await
            .unwrap();

        let found = client
            .get_bucket_access_control(&GetBucketAccessControlRequest {
                bucket: bucket_name.to_string(),
                entity: "allAuthenticatedUsers".to_string(),
            })
            .await
            .unwrap();
        assert_eq!(found.entity, "allAuthenticatedUsers");
        assert_eq!(found.role, BucketACLRole::READER);

        let acls = client
            .list_bucket_access_controls(&ListBucketAccessControlsRequest {
                bucket: bucket_name.to_string(),
            })
            .await
            .unwrap();
        assert_eq!(5, acls.items.len());

        client
            .delete_bucket_access_control(&DeleteBucketAccessControlRequest {
                bucket: bucket_name.to_string(),
                entity: "allAuthenticatedUsers".to_string(),
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    #[serial]
    pub async fn crud_object_access_controls() {
        let bucket_name = "rust-default-object-acl-test";
        let object_name = "test.txt";
        let (client, _project) = client().await;

        let _post = client
            .insert_object_access_control(&InsertObjectAccessControlRequest {
                bucket: bucket_name.to_string(),
                object: object_name.to_string(),
                generation: None,
                acl: ObjectAccessControlCreationConfig {
                    entity: "allAuthenticatedUsers".to_string(),
                    role: ObjectACLRole::READER,
                },
            })
            .await
            .unwrap();

        let found = client
            .get_object_access_control(&GetObjectAccessControlRequest {
                bucket: bucket_name.to_string(),
                entity: "allAuthenticatedUsers".to_string(),
                object: object_name.to_string(),
                generation: None,
            })
            .await
            .unwrap();
        assert_eq!(found.entity, "allAuthenticatedUsers");
        assert_eq!(found.role, ObjectACLRole::READER);

        let acls = client
            .list_object_access_controls(&ListObjectAccessControlsRequest {
                bucket: bucket_name.to_string(),
                object: object_name.to_string(),
                generation: None,
            })
            .await
            .unwrap();
        assert_eq!(2, acls.items.len());

        client
            .delete_object_access_control(&DeleteObjectAccessControlRequest {
                bucket: bucket_name.to_string(),
                object: object_name.to_string(),
                entity: "allAuthenticatedUsers".to_string(),
                generation: None,
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    #[serial]
    pub async fn crud_notification() {
        let bucket_name = "rust-bucket-test";
        let (client, project) = client().await;

        let notifications = client
            .list_notifications(&ListNotificationsRequest {
                bucket: bucket_name.to_string(),
            })
            .await
            .unwrap();

        for n in notifications.items.unwrap_or_default() {
            client
                .delete_notification(&DeleteNotificationRequest {
                    bucket: bucket_name.to_string(),
                    notification: n.id.to_string(),
                })
                .await
                .unwrap();
        }

        let post = client
            .insert_notification(&InsertNotificationRequest {
                bucket: bucket_name.to_string(),
                notification: NotificationCreationConfig {
                    topic: format!("projects/{project}/topics/{bucket_name}"),
                    event_types: Some(vec![EventType::ObjectMetadataUpdate, EventType::ObjectDelete]),
                    object_name_prefix: Some("notification-test".to_string()),
                    ..Default::default()
                },
            })
            .await
            .unwrap();

        let found = client
            .get_notification(&GetNotificationRequest {
                bucket: bucket_name.to_string(),
                notification: post.id.to_string(),
            })
            .await
            .unwrap();
        assert_eq!(found.id, post.id);
        assert_eq!(found.event_types.unwrap().len(), 2);
    }

    #[tokio::test]
    #[serial]
    pub async fn crud_hmac_key() {
        let _key_name = "rust-hmac-test";
        let (client, project_id) = client().await;

        let post = client
            .create_hmac_key(&CreateHmacKeyRequest {
                project_id: project_id.clone(),
                service_account_email: format!("spanner@{project_id}.iam.gserviceaccount.com"),
            })
            .await
            .unwrap();

        let found = client
            .get_hmac_key(&GetHmacKeyRequest {
                access_id: post.metadata.access_id.to_string(),
                project_id: project_id.clone(),
            })
            .await
            .unwrap();
        assert_eq!(found.id, post.metadata.id);
        assert_eq!(found.state, "ACTIVE");

        let keys = client
            .list_hmac_keys(&ListHmacKeysRequest {
                project_id: project_id.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        for n in keys.items.unwrap_or_default() {
            let result = client
                .update_hmac_key(&UpdateHmacKeyRequest {
                    access_id: n.access_id.to_string(),
                    project_id: n.project_id.to_string(),
                    metadata: HmacKeyMetadata {
                        state: "INACTIVE".to_string(),
                        ..n.clone()
                    },
                })
                .await
                .unwrap();
            assert_eq!(result.state, "INACTIVE");

            client
                .delete_hmac_key(&DeleteHmacKeyRequest {
                    access_id: n.access_id.to_string(),
                    project_id: n.project_id.to_string(),
                })
                .await
                .unwrap();
        }
    }

    #[tokio::test]
    #[serial]
    pub async fn metadata() {
        let bucket_name = "rust-object-test";
        let (client, _project) = client().await;
        let mut metadata = HashMap::<String, String>::new();
        metadata.insert("key1".to_string(), "value1".to_string());
        let uploaded = client
            .upload_object(
                &UploadObjectRequest {
                    bucket: bucket_name.to_string(),
                    ..Default::default()
                },
                vec![1, 2, 3, 4, 5, 6, 7],
                &UploadType::Multipart(Box::new(Object {
                    name: "test1_meta".to_string(),
                    content_type: Some("text/plain".to_string()),
                    content_language: Some("ja".to_string()),
                    metadata: Some(metadata),
                    ..Default::default()
                })),
            )
            .await
            .unwrap();
        assert_eq!(uploaded.content_type.unwrap(), "text/plain".to_string());
        assert_eq!(uploaded.content_language.unwrap(), "ja".to_string());
        assert_eq!(uploaded.metadata.unwrap().get("key1").unwrap().clone(), "value1".to_string());

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
                    )
                    .await
                    .unwrap()
            }
        };

        let object = client
            .get_object(&GetObjectRequest {
                bucket: uploaded.bucket.clone(),
                object: uploaded.name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(object.content_type.unwrap(), "text/plain".to_string());
        assert_eq!(object.content_language.unwrap(), "ja".to_string());
        assert_eq!(object.metadata.unwrap().get("key1").unwrap().clone(), "value1".to_string());

        let downloaded = download(Range::default()).await;
        assert_eq!(downloaded, vec![1, 2, 3, 4, 5, 6, 7]);
    }

    #[tokio::test]
    #[serial]
    pub async fn crud_object() {
        let bucket_name = "rust-object-test";
        let (client, _project) = client().await;

        let objects = client
            .list_objects(&ListObjectsRequest {
                bucket: bucket_name.to_string(),
                ..Default::default()
            })
            .await
            .unwrap()
            .items
            .unwrap_or_default();
        for o in objects {
            client
                .delete_object(&DeleteObjectRequest {
                    bucket: o.bucket.to_string(),
                    object: o.name.to_string(),
                    ..Default::default()
                })
                .await
                .unwrap();
        }

        let mut media = Media::new("test1");
        media.content_type = "text/plain".into();
        let uploaded = client
            .upload_object(
                &UploadObjectRequest {
                    bucket: bucket_name.to_string(),
                    ..Default::default()
                },
                vec![1, 2, 3, 4, 5, 6],
                &UploadType::Simple(media),
            )
            .await
            .unwrap();

        assert_eq!(uploaded.content_type.unwrap(), "text/plain".to_string());

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

        let _copied = client
            .copy_object(&CopyObjectRequest {
                destination_bucket: bucket_name.to_string(),
                destination_object: format!("{}_copy", uploaded.name),
                source_bucket: bucket_name.to_string(),
                source_object: uploaded.name.to_string(),
                ..Default::default()
            })
            .await
            .unwrap();

        let _rewrited = client
            .rewrite_object(&RewriteObjectRequest {
                destination_bucket: bucket_name.to_string(),
                destination_object: format!("{}_rewrite", uploaded.name),
                source_bucket: bucket_name.to_string(),
                source_object: uploaded.name.to_string(),
                ..Default::default()
            })
            .await
            .unwrap();

        let _composed = client
            .compose_object(&ComposeObjectRequest {
                bucket: bucket_name.to_string(),
                destination_object: format!("{}_composed", uploaded.name),
                destination_predefined_acl: None,
                composing_targets: ComposingTargets {
                    destination: Some(Object {
                        content_type: Some("image/jpeg".to_string()),
                        ..Default::default()
                    }),
                    source_objects: vec![SourceObjects {
                        name: format!("{}_rewrite", uploaded.name),
                        ..Default::default()
                    }],
                },
                ..Default::default()
            })
            .await
            .unwrap();
    }

    #[tokio::test]
    #[serial]
    pub async fn streamed_object() {
        let bucket_name = "rust-object-test";
        let file_name = format!("stream_{}", time::OffsetDateTime::now_utc().unix_timestamp());
        let (client, _project) = client().await;

        // let stream= reqwest::Client::default().get("https://avatars.githubusercontent.com/u/958174?s=96&v=4").send().await.unwrap().bytes_stream();
        let source = vec!["hello", " ", "world"];
        let size = source.iter().map(|x| x.len() as u64).sum();
        let chunks: Vec<Result<_, ::std::io::Error>> = source.clone().into_iter().map(Ok).collect();
        let stream = futures_util::stream::iter(chunks);
        let mut media = Media::new(file_name);
        media.content_length = Some(size);
        let upload_type = UploadType::Simple(media);
        let uploaded = client
            .upload_streamed_object(
                &UploadObjectRequest {
                    bucket: bucket_name.to_string(),
                    predefined_acl: None,
                    ..Default::default()
                },
                stream,
                &upload_type,
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

    #[tokio::test]
    #[serial]
    pub async fn resumable_simple_upload() {
        let bucket_name = "rust-object-test";
        let file_name = format!("resumable_{}", time::OffsetDateTime::now_utc().unix_timestamp());
        let (client, _project) = client().await;

        let mut media = Media::new(file_name.clone());
        media.content_type = "text/plain".into();
        let upload_type = UploadType::Simple(media);
        let uploader = client
            .prepare_resumable_upload(
                &UploadObjectRequest {
                    bucket: bucket_name.to_string(),
                    ..Default::default()
                },
                &upload_type,
            )
            .await
            .unwrap();
        let data = vec![1, 2, 3, 4, 5];
        uploader.upload_single_chunk(data.clone(), 5).await.unwrap();

        let get_request = &GetObjectRequest {
            bucket: bucket_name.to_string(),
            object: file_name.to_string(),
            ..Default::default()
        };
        let download = client.download_object(get_request, &Range::default()).await.unwrap();
        assert_eq!(data, download);

        let object = client.get_object(get_request).await.unwrap();
        assert_eq!(object.content_type.unwrap(), "text/plain");
    }

    #[tokio::test]
    #[serial]
    pub async fn resumable_multiple_chunk_upload() {
        let bucket_name = "rust-object-test";
        let file_name = format!("resumable_multiple_chunk{}", time::OffsetDateTime::now_utc().unix_timestamp());
        let (client, _project) = client().await;

        let metadata = Object {
            name: file_name.to_string(),
            content_type: Some("video/mp4".to_string()),
            ..Default::default()
        };
        let upload_type = UploadType::Multipart(Box::new(metadata));
        let uploader = client
            .prepare_resumable_upload(
                &UploadObjectRequest {
                    bucket: bucket_name.to_string(),
                    ..Default::default()
                },
                &upload_type,
            )
            .await
            .unwrap();
        let mut chunk1_data: Vec<u8> = (0..256 * 1024).map(|i| (i % 256) as u8).collect();
        let chunk2_data: Vec<u8> = (1..256 * 1024 + 50).map(|i| (i % 256) as u8).collect();
        let total_size = Some(chunk1_data.len() as u64 + chunk2_data.len() as u64);

        tracing::info!("start upload chunk {}", uploader.url());
        let chunk1 = ChunkSize::new(0, chunk1_data.len() as u64 - 1, total_size);
        tracing::info!("upload chunk1 {:?}", chunk1);
        let status1 = uploader
            .upload_multiple_chunk(chunk1_data.clone(), &chunk1)
            .await
            .unwrap();
        assert_eq!(status1, UploadStatus::ResumeIncomplete);

        tracing::info!("check status chunk1");
        let status_check = uploader.status(total_size).await.unwrap();
        assert_eq!(status_check, UploadStatus::ResumeIncomplete);

        let chunk2 = ChunkSize::new(
            chunk1_data.len() as u64,
            chunk1_data.len() as u64 + chunk2_data.len() as u64 - 1,
            total_size,
        );
        tracing::info!("upload chunk2 {:?}", chunk2);
        let status2 = uploader
            .upload_multiple_chunk(chunk2_data.clone(), &chunk2)
            .await
            .unwrap();
        assert!(matches!(status2, UploadStatus::Ok(_)));

        tracing::info!("check status chunk2");
        let status_check2 = uploader.status(total_size).await.unwrap();
        assert!(matches!(status_check2, UploadStatus::Ok(_)));

        let get_request = &GetObjectRequest {
            bucket: bucket_name.to_string(),
            object: file_name.to_string(),
            ..Default::default()
        };

        let object = client.get_object(get_request).await.unwrap();
        assert_eq!(object.content_type.unwrap(), "video/mp4");

        let download = client.download_object(get_request, &Range::default()).await.unwrap();
        chunk1_data.extend(chunk2_data);
        assert_eq!(chunk1_data, download);
    }

    #[tokio::test]
    #[serial]
    pub async fn resumable_upload_cancel() {
        let bucket_name = "rust-object-test";
        let file_name = format!("resumable_cancel{}", time::OffsetDateTime::now_utc().unix_timestamp());
        let (client, _project) = client().await;

        let metadata = Object {
            name: file_name.to_string(),
            content_type: Some("video/mp4".to_string()),
            ..Default::default()
        };
        let upload_type = UploadType::Multipart(Box::new(metadata));
        let uploader = client
            .prepare_resumable_upload(
                &UploadObjectRequest {
                    bucket: bucket_name.to_string(),
                    ..Default::default()
                },
                &upload_type,
            )
            .await
            .unwrap();
        let cloned = uploader.clone();
        uploader.cancel().await.unwrap();

        let result = cloned.upload_single_chunk(vec![1], 1).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[serial]
    pub async fn resumable_multiple_chunk_upload_unknown() {
        let bucket_name = "rust-object-test";
        let file_name = format!(
            "resumable_multiple_chunk_unknown{}",
            time::OffsetDateTime::now_utc().unix_timestamp()
        );
        let (client, _project) = client().await;

        let metadata = Object {
            name: file_name.to_string(),
            content_type: Some("video/mp4".to_string()),
            ..Default::default()
        };
        let upload_type = UploadType::Multipart(Box::new(metadata));
        let uploader = client
            .prepare_resumable_upload(
                &UploadObjectRequest {
                    bucket: bucket_name.to_string(),
                    ..Default::default()
                },
                &upload_type,
            )
            .await
            .unwrap();
        let mut chunk1_data: Vec<u8> = (0..256 * 1024).map(|i| (i % 256) as u8).collect();
        let chunk2_data: Vec<u8> = vec![10, 20, 30];
        let total_size = None;

        tracing::info!("start upload chunk {}", uploader.url());
        let chunk1 = ChunkSize::new(0, chunk1_data.len() as u64 - 1, total_size);
        tracing::info!("upload chunk1 {:?}", chunk1);
        let status1 = uploader
            .upload_multiple_chunk(chunk1_data.clone(), &chunk1)
            .await
            .unwrap();
        assert_eq!(status1, UploadStatus::ResumeIncomplete);

        tracing::info!("upload chunk1 resume {:?}", chunk1);
        let status1 = uploader
            .upload_multiple_chunk(chunk1_data.clone(), &chunk1)
            .await
            .unwrap();
        assert_eq!(status1, UploadStatus::ResumeIncomplete);

        // total size is required for final chunk.
        let remaining = chunk1_data.len() as u64 + chunk2_data.len() as u64;
        let chunk2 = ChunkSize::new(chunk1_data.len() as u64, remaining - 1, Some(remaining));
        tracing::info!("upload chunk2 {:?}", chunk2);
        let status2 = uploader
            .upload_multiple_chunk(chunk2_data.clone(), &chunk2)
            .await
            .unwrap();
        assert!(matches!(status2, UploadStatus::Ok(_)));

        let get_request = &GetObjectRequest {
            bucket: bucket_name.to_string(),
            object: file_name.to_string(),
            ..Default::default()
        };

        let object = client.get_object(get_request).await.unwrap();
        assert_eq!(object.content_type.unwrap(), "video/mp4");

        let download = client.download_object(get_request, &Range::default()).await.unwrap();
        chunk1_data.extend(chunk2_data);
        assert_eq!(chunk1_data, download);
    }
}
