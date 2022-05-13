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
use crate::http::entity2::acl::{BucketAccessControl, BucketAccessControlsCreationConfig, DeleteBucketAccessControlsRequest, GetBucketAccessControlsRequest, InsertBucketAccessControlsRequest, InsertObjectAccessControlRequest, ObjectAccessControl};
use crate::http::entity2::bucket::{Bucket, DeleteBucketRequest, GetBucketRequest, InsertBucketRequest, ListBucketsRequest, ListBucketsResponse, PatchBucketRequest};
use crate::http::entity2::channel::{Channel, ListChannelsResponse, StopChannelRequest, WatchableChannel};
use crate::http::entity2::iam::{GetIamPolicyRequest, Policy, SetIamPolicyRequest, TestIamPermissionsRequest, TestIamPermissionsResponse};
use crate::http::entity2::notification::{DeleteNotificationRequest, GetNotificationRequest, InsertNotificationRequest, ListNotificationsResponse, Notification};
use crate::http::entity::{InsertDefaultObjectAccessControlRequest, ListBucketAccessControlsResponse};

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
        bucket: &str,
        entity: &str,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        let action = async {
            let url = format!("{}/b/{}/defaultObjectAcl/{}?alt=json&prettyPrint=false", BASE_URL, bucket, entity);
            self.send(reqwest::Client::new().get(url)).await
        };
        invoke(cancel, action).await
    }

    pub async fn delete_default_object_acl(
        &self,
        bucket: &str,
        entity: &str,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let action = async {
            let url = format!("{}/b/{}/acl/{}?defaultObjectAcl=json&prettyPrint=false", BASE_URL, bucket, entity);
            self.send_get_empty(reqwest::Client::new().delete(url)).await
        };
        invoke(cancel, action).await
    }

    pub async fn insert_object_acl(
        &self,
        bucket: &str,
        object: &str,
        generation: Option<i64>,
        config: &ObjectAccessControlsCreationConfig,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        let action = async {
            let url = format!("{}/b/{}/o/{}/acl?alt=json&prettyPrint=false", BASE_URL, bucket, object);
            let mut query_param = vec![];
            if let Some(generation) = generation {
                query_param.push(("generation", generation));
            }
            self.send(reqwest::Client::new().post(url).query(&query_param).json(config)).await
        };
        invoke(cancel, action).await
    }

    pub async fn get_object_acl(
        &self,
        bucket: &str,
        object: &str,
        entity: &str,
        generation: Option<i64>,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        let action = async {
            let url = format!("{}/b/{}/o/{}/acl/{}?alt=json&prettyPrint=false", BASE_URL, bucket, object,entity);
            let mut query_param = vec![];
            if let Some(generation) = generation {
                query_param.push(("generation", generation));
            }
            self.send(reqwest::Client::new().get(url)).await
        };
        invoke(cancel, action).await
    }

    pub async fn list_object_acls(
        &self,
        bucket: &str,
        object: &str,
        generation: Option<i64>,
        cancel: Option<CancellationToken>,
    ) -> Result<Vec<ObjectAccessControl>, Error> {
        let action = async {
            let url = format!("{}/b/{}/o/{}/acl?alt=json&prettyPrint=false", BASE_URL, bucket, object);
            let mut query_param = vec![];
            if let Some(generation) = generation {
                query_param.push(("generation", generation));
            }
            self.send::<ListObjectAccessControlsResponse>(reqwest::Client::new().get(url)).await
        };
        invoke(cancel, action).await.map(|e| e.items )
    }

    pub async fn delete_object_acl(
        &self,
        bucket: &str,
        object: &str,
        generation: Option<i64>,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let action = async {
            let url = format!("{}/b/{}/o/{}/acl?alt=json&prettyPrint=false", BASE_URL, bucket, object);
            let mut query_param = vec![];
            if let Some(generation) = generation {
                query_param.push(("generation", generation));
            }
            self.send_get_empty(reqwest::Client::new().delete(url)).await
        };
        invoke(cancel, action).await
    }

    pub async fn create_hmac_keys(
        &self,
        project: &str,
        service_account_email: &str,
        cancel: Option<CancellationToken>,
    ) -> Result<HmacKeyMetadata, Error> {
        let action = async {
            let url = format!("{}/projects/{}/hmacKeys?alt=json&prettyPrint=false", BASE_URL, project);
            let query_param = vec![("generation", service_account_email)];
            self.send(reqwest::Client::new().post(url).query(&query_param)).await
        };
        invoke(cancel, action).await
    }

    pub async fn delete_hmac_keys(
        &self,
        project: &str,
        access_id: &str,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let action = async {
            let url = format!("{}/projects/{}/hmacKeys/{}?alt=json&prettyPrint=false", BASE_URL, project, access_id);
            self.send_get_empty(reqwest::Client::new().delete(url)).await
        };
        invoke(cancel, action).await
    }

    pub async fn get_hmac_keys(
        &self,
        project: &str,
        access_id: &str,
        cancel: Option<CancellationToken>,
    ) -> Result<HmacKeyMetadata, Error> {
        let action = async {
            let url = format!("{}/projects/{}/hmacKeys/{}?alt=json&prettyPrint=false", BASE_URL, project, access_id);
            self.send(reqwest::Client::new().get(url)).await
        };
        invoke(cancel, action).await
    }

    pub async fn list_hmac_keys(
        &self,
        project: &str,
        max_results: Option<&u32>,
        page_token: Option<&str>,
        service_account_email: Option<&str>,
        show_deleted_keys: bool,
        cancel: Option<CancellationToken>,
    ) -> Result<HmacKeyMetadata, Error> {
        let max_results = if let Some(max_results) = max_results {
            max_results.to_string()
        } else {
            "".to_string()
        };
        let show_deleted_keys = show_deleted_keys.to_string();
        let action = async {
            let url = format!("{}/projects/{}/mackKeys?alt=json&prettyPrint=false", BASE_URL, project);
            let mut query_param = vec![];
            if let Some(page_token) = page_token {
                query_param.push(("pageToken", page_token));
            }
            if let Some(service_account_email) = service_account_email {
                query_param.push(("serviceAccountEmail", service_account_email));
            }
            query_param.push(("showDeletedKeys", show_deleted_keys.as_str()));
            if !max_results.is_empty() {
                query_param.push(("maxResults", max_results.as_str()));
            }
            self.send(reqwest::Client::new().get(url).query(&query_param)).await
        };
        invoke(cancel, action).await
    }

    pub async fn insert_object_simple(
        &self,
        bucket: &str,
        object: &str,
        content_encoding: Option<&str>,
        kms_key_name: Option<&str>,
        predefined_acl: Option<PredefinedObjectAcl>,
        projection: Option<Projection>,
        body: &[u8],
        cancel: Option<CancellationToken>,
    ) -> Result<Object, Error> {
        let action = async {
            let url = format!("{}/b/{}/o?uploadType=media", BASE_URL, bucket);
            let mut query_param = vec![("name", object)];
            with_projection(&mut query_param, projection);
            with_acl(&mut query_param, None, predefined_acl);
            if let Some(content_encoding) = content_encoding {
                query_param.push(("contentEncoding", content_encoding));
            }
            if let Some(kms_key_name) = kms_key_name {
                query_param.push(("kmsKeyName", kms_key_name));
            }
            self.send(reqwest::Client::new().post(url).query(&query_param).body(body)).await
        };
        invoke(cancel, action).await
    }

    pub async fn delete_object(
        &self,
        bucket: &str,
        object: &str,
        generation: Option<i64>,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let action = async {
            let url = format!("{}/b/{}/o/{}", BASE_URL, bucket, object);
            let mut query_param = vec![];
            if let Some(generation) = generation {
                query_param.push(("generation", generation));
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
            let mut query_param = vec![];
            with_projection(&mut query_param, projection);
            let builder = reqwest::Client::new().get(url).query(&query_param);
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
            with_projection(&mut query_param, projection);
            self.send(reqwest::Client::new().get(url).query(&query_param)).await
        };
        invoke(cancel, action).await
    }

    pub async fn patch_object(
        &self,
        bucket: &str,
        object: &str,
        generation: Option<i64>,
        predefined_acl: Option<PredefinedObjectAcl>,
        projection: Option<Projection>,
        resource: &Object,
        cancel: Option<CancellationToken>,
    ) -> Result<Object, Error> {
        let generation = generation.map(|x| x.to_string());
        let action = async {
            let url = format!("{}/b/{}/o/{}", BASE_URL, bucket, object);
            let mut query_param = vec![];
            with_projection(&mut query_param, projection);
            with_acl(&mut query_param, None, predefined_acl);
            if let Some(generation) = generation {
                query_param.push(("generation", &generation));
            }
            self.send(reqwest::Client::new().patch(url).query(&query_param).json(&resource)).await
        };
        invoke(cancel, action).await
    }

    pub async fn list_objects(
        &self,
        bucket: &str,
        delimiter: Option<&str>,
        end_offset: Option<&str>,
        include_trailing_delimiter: Option<&bool>,
        max_results: Option<&i32>,
        page_token: Option<&str>,
        prefix: Option<&str>,
        projection: Option<Projection>,
        start_offset: Option<&str>,
        versions: Option<&bool>,
        cancel: Option<CancellationToken>,
    ) -> Result<ListObjectsResponse, Error> {
        let max_results = max_results.map(|x| x.to_string());
        let versions = versions.map(|x| x.to_string());
        let include_trailing_delimiter = include_trailing_delimiter.map(|x| x.to_string());
        let action = async {
            let url = format!("{}/b/{}/o/{}", BASE_URL, bucket, object);
            let mut query_param = vec![];
            with_projection(&mut query_param, projection);
            if let Some(delimiter) = delimiter {
                query_param.push(("delimiter", delimiter));
            }
            if let Some(v) = end_offset {
                query_param.push(("endOffset", v));
            }
            if let Some(v) = include_trailing_delimiter{
                query_param.push(("includeTrailingDelimiter", &v));
            }
            if let Some(v) = page_token{
                query_param.push(("pageToken", v));
            }
            if let Some(v) = prefix{
                query_param.push(("prefix", v));
            }
            if let Some(v) = start_offset {
                query_param.push(("startOffset", v));
            }
            if let Some(v) = versions {
                query_param.push(("versions", &v));
            }
            if let Some(v) = max_results {
                query_param.push(("maxResults", &v));
            }
            self.send(reqwest::Client::new().get(url).query(&query_param)).await
        };
        invoke(cancel, action).await
    }

    pub async fn rewrite_object(
        &self,
        destination_bucket: &str,
        destination_object: &str,
        source_bucket: &str,
        source_object: &str,
        destination_kms_key_name: Option<&str>,
        destination_predefined_object_acl: Option<PredefinedObjectAcl>,
        max_bytes_rewritten_per_call: Option<&i64>,
        projection: Option<Projection>,
        rewrite_token: Option<&str>,
        source_generation: Option<&i64>,
        cancel: Option<CancellationToken>,
    ) -> Result<RewriteResponse, Error> {
        let max_bytes_rewritten_per_call = max_bytes_rewritten_per_call.map(|x| x.to_string());
        let source_generation = source_generation.map(|x| x.to_string());
        let action = async {
            let url = format!("{}/b/{}/o/{}/rewriteTo/b/{}/o/{}", BASE_URL, source_bucket, source_object, destination_bucket, destination_object);
            let mut query_param = vec![];
            with_projection(&mut query_param, projection);
            if let Some(v) = destination_kms_key_name {
                query_param.push(("destinationKmsKeyName", v));
            }
            if let Some(v) = destination_predefined_object_acl {
                query_param.push(("destinationPredefinedAcl", v.into()));
            }
            if let Some(v) = max_bytes_rewritten_per_call {
                query_param.push(("maxBytesRewrittenPerCall", &v));
            }
            if let Some(v) = rewrite_token {
                query_param.push(("rewriteToken", &v));
            }
            if let Some(v) = source_generation {
                query_param.push(("sourceGeneration", &v));
            }
            self.send(reqwest::Client::new().post(url).query(&query_param)).await
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


fn with_projection(param: &mut Vec<(&str, &str)>, projection: Option<Projection>) {
    if let Some(projection) = projection {
        param.push(("projection", projection.into()));
    }
}

fn with_acl(
    param: &mut Vec<(&str, &str)>,
    bucket_acl: Option<PredefinedBucketAcl>,
    object_acl: Option<PredefinedObjectAcl>,
) {
    if let Some(bucket_acl) = bucket_acl {
        param.push(("predefinedAcl", bucket_acl.into()));
    }
    if let Some(object_acl) = object_acl {
        param.push(("predefinedDefaultObjectAcl", object_acl.into()));
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
