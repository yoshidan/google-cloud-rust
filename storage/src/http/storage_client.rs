use std::cmp::max;
use crate::http::CancellationToken;
use google_cloud_auth::token_source::TokenSource;
use google_cloud_metadata::project_id;
use reqwest::{RequestBuilder, Response};
use std::collections::HashMap;
use std::future::Future;
use std::mem;
use std::sync::Arc;
use tracing::info;
use crate::http::entity2::acl::{BucketAccessControl, BucketAccessControlsCreationConfig, DeleteBucketAccessControlsRequest, DeleteDefaultObjectAccessControlRequest, GetBucketAccessControlsRequest, GetDefaultObjectAccessControlRequest, GetObjectAccessControlRequest, InsertBucketAccessControlsRequest, InsertDefaultObjectAccessControlRequest, InsertObjectAccessControlRequest, ListBucketAccessControlsResponse, ListObjectAccessControlsRequest, ListObjectAccessControlsResponse, ObjectAccessControl, ObjectAccessControlsCreationConfig};
use crate::http::entity2::bucket::{Bucket, DeleteBucketRequest, GetBucketRequest, InsertBucketRequest, ListBucketsRequest, ListBucketsResponse, PatchBucketRequest};
use crate::http::entity2::channel::{Channel, ListChannelsResponse, StopChannelRequest, WatchableChannel};
use crate::http::entity2::common::Projection;
use crate::http::entity2::iam::{GetIamPolicyRequest, Policy, SetIamPolicyRequest, TestIamPermissionsRequest, TestIamPermissionsResponse};
use crate::http::entity2::notification::{DeleteNotificationRequest, GetNotificationRequest, InsertNotificationRequest, ListNotificationsResponse, Notification};
use crate::http::entity2::hmac_key::{CreateHmacKeyRequest, CreateHmacKeyResponse, DeleteHmacKeyRequest, GetHmacKeyRequest, HmacKeyMetadata, ListHmacKeysRequest};
use crate::http::entity2::object::{DeleteObjectRequest, InsertSimpleObjectRequest, ListObjectsRequest, ListObjectsResponse, Object, PatchObjectRequest, RewriteObjectRequest, RewriteObjectResponse};

const BASE_URL: &str = "https://storage.googleapis.com/storage/v1";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("http error status={0} message={1}")]
    Response(u16, String),
    #[error(transparent)]
    HttpClient(#[from] reqwest::Error),
    #[error(transparent)]
    AuthError(#[from] google_cloud_auth::error::Error),
    #[error("operation cancelled")]
    Cancelled,
}

#[derive(Clone)]
pub(crate) struct StorageClient {
    ts: Arc<dyn TokenSource>,
}

impl StorageClient {
    pub(crate) fn new(ts: Arc<dyn TokenSource>) -> Self {
        Self { ts }
    }

    pub async fn delete_bucket(
        &self,
        req: DeleteBucketRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let action = async {
            let url = format!("{}/b/{}", BASE_URL, req.bucket);
            let param = req.metageneration.to_param();
            self.send_get_empty(reqwest::Client::new().delete(url).query(&param)).await
        };
        invoke(cancel, action).await
    }

    pub async fn insert_bucket(
        &self,
        req: &InsertBucketRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Bucket, Error> {
        let action = async {
            let url = format!("{}/b", BASE_URL);
            let mut p = vec![req.project.as_param()];
            if let Some(v) = req.projection {
                p.push(v.as_param());
            }
            if let Some(v) = req.predefined_acl {
                p.push(v.as_param());
            }
            if let Some(v) = req.predefined_default_object_acl{
                p.push(v.as_default_object_acl());
            }
            self.send(reqwest::Client::new().post(url).query(&p).json(&req.bucket)).await
        };
        invoke(cancel, action).await
    }

    pub async fn get_bucket(
        &self,
        req: &GetBucketRequest,
        cancel: Option<CancellationToken>
    ) -> Result<Bucket, Error> {
        let metageneration =  req.metageneration.to_param();
        let action = async {
            let url = format!("{}/b/{}", BASE_URL, req.bucket);
            let mut p = vec![];
            if let Some(v) = req.projection {
                p.push(v.as_param());
            }
            for v in metageneration {
                p.push(v.as_param());
            }
            self.send(reqwest::Client::new().get(url).query(&query_param)).await
        };
        invoke(cancel, action).await
    }

    pub async fn list_buckets(
        &self,
        req: &ListBucketsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListBucketsResponse, Error> {
        let max_results =  req.max_results.map(|x| x.to_param());
        let action = async {
            let url = format!("{}/b?alt=json&prettyPrint=false", BASE_URL);
            let mut p = vec![req.project.as_param()];
            if let Some(v) = req.projection {
                p.push(v.as_param());
            }
            if let Some(v) = &req.page_token {
                p.push(v.as_param());
            }
            if let Some(v) = &req.prefix {
                p.push(v.as_param());
            }
            if let Some(v) = &max_results {
                p.push(v.as_param());
            }
            self.send(reqwest::Client::new().get(url).query(&p)).await
        };
        invoke(cancel, action).await
    }

    pub async fn patch_bucket(
        &self,
        req: &PatchBucketRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Bucket, Error> {
        let metageneration =  req.metageneration.to_param();
        let action = async {
            let url = format!("{}/b/{}", BASE_URL, req.bucket);
            if let Some(v) = req.projection {
                p.push(v.as_param());
            }
            if let Some(v) = req.predefined_acl {
                p.push(v.as_param());
            }
            if let Some(v) = req.predefined_default_object_acl{
                p.push(v.as_default_object_acl());
            }
            for v in metageneration {
                p.push(v.as_param());
            }
            self.send(reqwest::Client::new().patch(url).query(&p).json(&req.metadata)).await
        };
        invoke(cancel, action).await
    }

    pub async fn get_iam_policy(
        &self,
        req: &GetIamPolicyRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Policy, Error> {
        let requested_policy_version = req.requested_policy_version.map(|x| x.to_string());
        let action = async {
            let url = format!("{}/b/{}/iam?alt=json&prettyPrint=false", BASE_URL, req.resource);
            let mut p= vec![];
            if let Some(v) = requested_policy_version {
                p.push(("optionsRequestedPolicyVersion", v.as_str()));
            }
            self.send(reqwest::Client::new().get(url).query(&p)).await
        };
        invoke(cancel, action).await
    }

    pub async fn test_iam_permission(
        &self,
        req: &TestIamPermissionsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<TestIamPermissionsResponse, Error> {
        let action = async {
            let url = format!("{}/b/{}/iam/testPermissions", BASE_URL, req.resource);
            let mut p = vec![];
            for permission in &req.permissions {
                p.push(("permissions", permission));
            }
            self.send(reqwest::Client::new().get(url).query(&p)).await
        };
        invoke(cancel, action).await
    }

    pub async fn set_iam_policy(
        &self,
        req: &SetIamPolicyRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Policy, Error> {
        let action = async {
            let url = format!("{}/b/{}/iam", BASE_URL, req.resource);
            self.send(reqwest::Client::new().put(url).json(&req.policy)).await
        };
        invoke(cancel, action).await
    }

    pub async fn insert_bucket_acl(
        &self,
        req: &InsertBucketAccessControlsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<BucketAccessControl, Error> {
        let action = async {
            let url = format!("{}/b/{}/acl", BASE_URL, req.bucket);
            self.send(reqwest::Client::new().post(url).json(&req.acl)).await
        };
        invoke(cancel, action).await
    }

    pub async fn get_bucket_acl(
        &self,
        req: GetBucketAccessControlsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<BucketAccessControl, Error> {
        let action = async {
            let url = format!("{}/b/{}/acl/{}", BASE_URL, req.bucket, req.entity);
            self.send(reqwest::Client::new().get(url)).await
        };
        invoke(cancel, action).await
    }

    pub async fn delete_bucket_acl(
        &self,
        req: DeleteBucketAccessControlsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let action = async {
            let url = format!("{}/b/{}/acl/{}", BASE_URL, req.bucket, req.entity);
            self.send_get_empty(reqwest::Client::new().delete(url)).await
        };
        invoke(cancel, action).await
    }

    pub async fn list_bucket_acls(
        &self,
        bucket: &str,
        cancel: Option<CancellationToken>,
    ) -> Result<Vec<BucketAccessControl>, Error> {
        let action = async {
            let url = format!("{}/b/{}/acl", BASE_URL, bucket);
            self.send::<ListBucketAccessControlsResponse>(reqwest::Client::new().get(url)).await
        };
        invoke(cancel, action).await.map(|e| e.items )
    }

    pub async fn insert_notification(
        &self,
        req: &InsertNotificationRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Notification, Error> {
        let action = async {
            let url = format!("{}/b/{}/notificationConfigs", BASE_URL, req.bucket);
            self.send(reqwest::Client::new().post(url).json(&req.notification)).await
        };
        invoke(cancel, action).await
    }

    pub async fn list_notifications(
        &self,
        bucket: &str,
        cancel: Option<CancellationToken>,
    ) -> Result<Vec<Notification>, Error> {
        let action = async {
            let url = format!("{}/b/{}/notificationConfigs", BASE_URL, bucket);
            self.send::<ListNotificationsResponse>(reqwest::Client::new().get(url)).await
        };
        invoke(cancel, action).await.map(|e| e.items )
    }

    pub async fn get_notification(
        &self,
        req: GetNotificationRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Notification, Error> {
        let action = async {
            let url = format!("{}/b/{}/notificationConfigs/{}", BASE_URL, req.bucket, req.notification);
            self.send(reqwest::Client::new().get(url)).await
        };
        invoke(cancel, action).await
    }

    pub async fn delete_notification(
        &self,
        req: DeleteNotificationRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let action = async {
            let url = format!("{}/b/{}/notificationConfigs/{}", BASE_URL, req.bucket, req.notification);
            self.send_get_empty(reqwest::Client::new().delete(url)).await
        };
        invoke(cancel, action).await
    }

    pub async fn list_channels(
        &self,
        bucket: &str,
        cancel: Option<CancellationToken>,
    ) -> Result<Vec<Channel>, Error> {
        let action = async {
            let url = format!("{}/b/{}/channels", BASE_URL, bucket);
            self.send::<ListChannelsResponse>(reqwest::Client::new().get(url)).await
        };
        invoke(cancel, action).await.map(|e| e.items )
    }

    pub async fn stop_channel(
        &self,
        req: &StopChannelRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let action = async {
            let url = format!("{}/channels/stop", BASE_URL);
            self.send_get_empty(reqwest::Client::new().post(url).json(&req.channel)).await
        };
        invoke(cancel, action).await
    }

    pub async fn insert_default_object_acl(
        &self,
        req: &InsertDefaultObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        let action = async {
            let url = format!("{}/b/{}/defaultObjectAcl", BASE_URL, req.bucket);
            self.send(reqwest::Client::new().post(url).json(&req.object_access_control)).await
        };
        invoke(cancel, action).await
    }

    pub async fn get_default_object_acl(
        &self,
        req: GetDefaultObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        let action = async {
            let url = format!("{}/b/{}/defaultObjectAcl/{}", BASE_URL, req.bucket, req.entity);
            self.send(reqwest::Client::new().get(url)).await
        };
        invoke(cancel, action).await
    }

    pub async fn delete_default_object_acl(
        &self,
        req: DeleteDefaultObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let action = async {
            let url = format!("{}/b/{}/defaultObjectAcl/{}", BASE_URL, req.bucket, req.entity);
            self.send_get_empty(reqwest::Client::new().delete(url)).await
        };
        invoke(cancel, action).await
    }

    pub async fn insert_object_acl(
        &self,
        req: InsertObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        let generation = req.generation.map(|x| x.to_param());
        let action = async {
            let url = format!("{}/b/{}/o/{}/acl", BASE_URL, req.bucket, req.object);
            let mut p= vec![];
            if let Some(generation) = generation {
                p.push(generation.as_param());
            }
            self.send(reqwest::Client::new().post(url).query(&p).json(config)).await
        };
        invoke(cancel, action).await
    }

    pub async fn get_object_acl(
        &self,
        req: GetObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        let generation = req.generation.map(|x| x.to_param());
        let action = async {
            let url = format!("{}/b/{}/o/{}/acl/{}", BASE_URL, req.bucket, req.object,req.entity);
            let mut p = vec![];
            if let Some(generation) = generation {
                p.push(generation.as_param());
            }
            self.send(reqwest::Client::new().get(url).query(&p)).await
        };
        invoke(cancel, action).await
    }

    pub async fn list_object_acls(
        &self,
        req: ListObjectAccessControlsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Vec<ObjectAccessControl>, Error> {
        let generation = req.generation.map(|x| x.to_param());
        let action = async {
            let url = format!("{}/b/{}/o/{}/acl", BASE_URL, req.bucket, req.object);
            let mut p = vec![];
            if let Some(generation) = generation {
                p.push(generation.as_param());
            }
            self.send::<ListObjectAccessControlsResponse>(reqwest::Client::new().get(url).query(&p)).await
        };
        invoke(cancel, action).await.map(|e| e.items )
    }

    pub async fn delete_object_acl(
        &self,
        req: DeleteBucketRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let generation = req.generation.map(|x| x.to_param());
        let action = async {
            let url = format!("{}/b/{}/o/{}/acl", BASE_URL, req.bucket, req.object);
            let mut p = vec![];
            if let Some(generation) = generation {
                p.push(generation.as_param());
            }
            self.send_get_empty(reqwest::Client::new().delete(url)).await
        };
        invoke(cancel, action).await
    }

    pub async fn create_hmac_keys(
        &self,
        req: CreateHmacKeyRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<CreateHmacKeyResponse, Error> {
        let action = async {
            let url = format!("{}/projects/{}/hmacKeys", BASE_URL, req.project_id);
            let p= vec![("service_account_email", req.service_account_email.as_str())];
            self.send(reqwest::Client::new().post(url).query(&p)).await
        };
        invoke(cancel, action).await
    }

    pub async fn delete_hmac_keys(
        &self,
        req: DeleteHmacKeyRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let action = async {
            let url = format!("{}/projects/{}/hmacKeys/{}", BASE_URL, req.project_id, req.access_id);
            self.send_get_empty(reqwest::Client::new().delete(url)).await
        };
        invoke(cancel, action).await
    }

    pub async fn get_hmac_keys(
        &self,
        req: GetHmacKeyRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<HmacKeyMetadata, Error> {
        let action = async {
            let url = format!("{}/projects/{}/hmacKeys/{}", BASE_URL, req.project_id, req.access_id);
            self.send(reqwest::Client::new().get(url)).await
        };
        invoke(cancel, action).await
    }

    pub async fn list_hmac_keys(
        &self,
        req: ListHmacKeysRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<HmacKeyMetadata, Error> {
        let show_deleted_keys = req.show_deleted_keys.to_string();
        let max_results = req.max_results.map(|x| x.to_param());
        let action = async {
            let url = format!("{}/projects/{}/mackKeys", BASE_URL, req.project_id);
            let mut p= vec![("showDeletedKeys", show_deleted_keys.as_str())];
            if let Some(v) = req.page_token {
                p.push(v.as_param());
            }
            if let Some(v) = req.service_account_email {
                p.push(("serviceAccountEmail", v.as_str()));
            }
            if let Some(v) = max_results {
                p.push(v.as_param());
            }
            self.send(reqwest::Client::new().get(url).query(&query_param)).await
        };
        invoke(cancel, action).await
    }

    pub async fn insert_object_simple(
        &self,
        req: InsertSimpleObjectRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Object, Error> {
        let metageneration_match  = req.metageneration_match.map(|x| x.to_param());
        let generation_match = req.generation_match.map(|x| x.to_param());
        let action = async {
            let url = format!("{}/b/{}/o?uploadType=media", BASE_URL, bucket);
            let mut p= vec![("name", object)];
            if let Some(v) = req.content_encoding {
                p.push(("contentEncoding", v.as_str()));
            }
            if let Some(v) = generation_match {
                for v in v {
                    p.push(v.as_param());
                }
            }
            if let Some(v) = metageneration_match {
                for v in v {
                    p.push(v.as_param());
                }
            }
            if let Some(v) = req.kms_key_name {
                p.push(("kmsKeyName", v.as_str()));
            }
            if let Some(v) = req.predefined_acl{
                p.push(v.as_param());
            }
            if let Some(v) = req.projection {
                p.push(v.as_param());
            }
            self.send(reqwest::Client::new().post(url).query(&query_param).body(body)).await
        };
        invoke(cancel, action).await
    }

    pub async fn delete_object(
        &self,
        req: DeleteObjectRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let metageneration_match = req.metageneration_match.map(|x| x.to_param());
        let generation_match = req.generation_match.map(|x| x.to_param());
        let generation = req.generation.map(|x| x.to_param());
        let action = async {
            let url = format!("{}/b/{}/o/{}", BASE_URL, bucket, object);
            let mut p = vec![];
            if let Some(v) = generation {
                p.push(v.as_param());
            }
            if let Some(v) = generation_match {
                for v in v {
                    p.push(v.as_param());
                }
            }
            if let Some(v) = metageneration_match {
                for v in v {
                    p.push(v.as_param());
                }
            }
            self.send_get_empty(reqwest::Client::new().delete(url).query(&query_param)).await
        };
        invoke(cancel, action).await
    }

    pub async fn download_object(
        &self,
        bucket: &str,
        object: &str,
        projection: Option<Projection>,
        cancel: Option<CancellationToken>,
    ) -> Result<Vec<u8>, Error> {
        let action = async {
            let url = format!("{}/b/{}/o/{}?alt=media", BASE_URL, bucket, object);
            let mut p = vec![];
            if let Some(v) = projection {
                p.push(v.as_param());
            }
            let builder = reqwest::Client::new().get(url).query(&p);
            let builder = self.with_headers(builder).await?;
            let response = builder.send().await?;
            if response.status().is_success() {
                Ok(response.bytes().await?.to_vec())
            } else {
                Err(map_error(response).await)
            }
        };
        invoke(cancel, action).await
    }

    pub async fn get_object(
        &self,
        bucket: &str,
        object: &str,
        projection: Option<Projection>,
        cancel: Option<CancellationToken>,
    ) -> Result<Object, Error> {
        let action = async {
            let url = format!("{}/b/{}/o/{}", BASE_URL, bucket, object);
            let mut query_param = vec![];
            if let Some(v) = projection{
                p.push(v.as_param());
            };
            self.send(reqwest::Client::new().get(url).query(&query_param)).await
        };
        invoke(cancel, action).await
    }

    pub async fn patch_object(
        &self,
        req: &PatchObjectRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Object, Error> {
        let metageneration_match = req.metageneration_match.map(|x| x.to_param());
        let generation_match = req.generation_match.map(|x| x.to_param());
        let generation = req.generation.map(|x| x.to_param());
        let action = async {
            let url = format!("{}/b/{}/o/{}", BASE_URL, bucket, object);
            let mut query_param = vec![];
            if let Some(v) = generation {
                p.push(v.as_param());
            }
            if let Some(v) = generation_match {
                for v in v {
                    p.push(v.as_param());
                }
            }
            if let Some(v) = metageneration_match {
                for v in v {
                    p.push(v.as_param());
                }
            }
            if let Some(v) = req.projection {
                p.push(v.as_param());
            }
            if let Some(v) = req.predefined_acl {
                p.push(v.as_param());
            }
            self.send(reqwest::Client::new().patch(url).query(&query_param).json(&resource)).await
        };
        invoke(cancel, action).await
    }

    pub async fn list_objects(
        &self,
        req: ListObjectsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListObjectsResponse, Error> {
        let max_results = req.max_results.map(|x| x.to_param());
        let versions = req.versions.map(|x| x.to_string());
        let include_trailing_delimiter = req.include_trailing_delimiter.map(|x| x.to_string());
        let action = async {
            let url = format!("{}/b/{}/o/{}", BASE_URL, bucket, object);
            let mut p = vec![];
            if let Some(v) = req.projection {
                p.push(v.as_param());
            }
            if let Some(v) = req.delimiter {
                p.push(("delimiter", &v));
            }
            if let Some(v) = req.end_offset {
                p.push(("endOffset", &v));
            }
            if let Some(v) = include_trailing_delimiter{
                p.push(("includeTrailingDelimiter", &v));
            }
            if let Some(v) = req.page_token{
                p.push(v.as_param());
            }
            if let Some(v) = req.prefix{
                p.push(v.as_param());
            }
            if let Some(v) = req.start_offset {
                p.push(("startOffset", &v));
            }
            if let Some(v) = versions {
                p.push(("versions", &v));
            }
            if let Some(v) = max_results {
                p.push(v.as_param());
            }
            self.send(reqwest::Client::new().get(url).query(&p)).await
        };
        invoke(cancel, action).await
    }

    pub async fn rewrite_object(
        &self,
        req: RewriteObjectRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<RewriteObjectResponse, Error> {
        let max_bytes_rewritten_per_call = req.max_bytes_rewritten_per_call.map(|x| x.to_string());
        let metageneration_match = req.metageneration_match.map(|x| x.to_param());
        let generation_match = req.generation_match.map(|x| x.to_param());
        let src_metageneration_match = req.source_metageneration_match.map(|x| x.to_source_param());
        let src_generation_match = req.source_generation_match.map(|x| x.to_source_param());
        let source_generation = req.source_generation.map(|x| x.to_string());
        let action = async {
            let url = format!("{}/b/{}/o/{}/rewriteTo/b/{}/o/{}", BASE_URL, source_bucket, source_object, destination_bucket, destination_object);
            let mut p = vec![];
            if let Some(v)  = req.projection {
                p.push(v.as_param());
            }
            if let Some(v) = req.destination_kms_key_name {
                p.push(("destinationKmsKeyName", &v));
            }
            if let Some(v) = req.destination_predefined_object_acl {
                p.push(("destinationPredefinedAcl", v.as_str()));
            }
            if let Some(v) = metageneration_match {
                for v in v {
                    p.push(v.as_param());
                }
            }
            if let Some(v) = generation_match {
                for v in v {
                    p.push(v.as_param());
                }
            }
            if let Some(v) = src_metageneration_match {
                for v in v {
                    p.push(v.as_param());
                }
            }
            if let Some(v) = src_generation_match {
                for v in v {
                    p.push(v.as_param());
                }
            }
            if let Some(v) = max_bytes_rewritten_per_call {
                p.push(("maxBytesRewrittenPerCall", &v));
            }
            if let Some(v) = req.rewrite_token {
                p.push(("rewriteToken", &v));
            }
            if let Some(v) = source_generation {
                p.push(("sourceGeneration", &v));
            }
            self.send(reqwest::Client::new().post(url).query(&p)).await
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


    async fn send<T: for<'de> serde::Deserialize<'de>>(&self, builder: RequestBuilder) -> Result<T,Error> {
        let builder = self.with_headers(builder).await?;
        let response = builder.send().await?;
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(map_error(response).await)
        }
    }

    async fn send_get_empty(&self, builder: RequestBuilder) -> Result<(),Error> {
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
