use crate::http::entity::common_enums::{PredefinedBucketAcl, PredefinedObjectAcl, Projection};
use crate::http::entity::{
    Bucket, DeleteBucketRequest, GetBucketRequest, InsertBucketRequest, ListBucketsRequest, ListBucketsResponse,
    PatchBucketRequest, UpdateBucketRequest,
};
use crate::http::iam::{GetIamPolicyRequest, Policy, TestIamPermissionsRequest, TestIamPermissionsResponse};
use crate::http::CancellationToken;
use google_cloud_auth::token_source::TokenSource;
use google_cloud_metadata::project_id;
use reqwest::{RequestBuilder, Response};
use std::collections::HashMap;
use std::future::Future;
use std::mem;
use std::sync::Arc;

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

    async fn with_headers(&self, builder: RequestBuilder) -> Result<RequestBuilder, Error> {
        let token = self.ts.token().await?;
        Ok(builder
            .header("X-Goog-Api-Client", "rust")
            .header(reqwest::header::USER_AGENT, "google-cloud-storage")
            .header(reqwest::header::AUTHORIZATION, token.value()))
    }

    pub async fn delete_bucket(
        &self,
        req: DeleteBucketRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let action = async {
            let url = format!("{}/b/{}?alt=json&prettyPrint=false", BASE_URL, req.bucket);
            let builder = self.with_headers(reqwest::Client::new().delete(url)).await?;
            send(builder).await
        };
        invoke(cancel, action).await
    }

    pub async fn insert_bucket(
        &self,
        project: &str,
        req: &InsertBucketRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Bucket, Error> {
        let action = async {
            let url = format!("{}/b?alt=json&prettyPrint=false", BASE_URL);
            let mut query_param = vec![("project", project)];
            with_projection(&mut query_param, req.projection);
            with_acl(&mut query_param, req.predefined_acl, req.predefined_default_object_acl);
            let builder = self.with_headers(reqwest::Client::new().post(url)).await?;
            send(builder.query(&query_param).json(&req.bucket)).await
        };
        invoke(cancel, action).await
    }

    pub async fn get_bucket(&self, req: &GetBucketRequest, cancel: Option<CancellationToken>) -> Result<Bucket, Error> {
        let action = async {
            let url = format!("{}/b/{}?alt=json&prettyPrint=false", BASE_URL, req.bucket);
            let mut query_param = vec![];
            with_projection(&mut query_param, req.projection);
            let builder = self.with_headers(reqwest::Client::new().get(url)).await?;
            send(builder.query(&query_param)).await
        };
        invoke(cancel, action).await
    }

    pub async fn list_buckets(
        &self,
        project: &str,
        req: &ListBucketsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListBucketsResponse, Error> {
        let max_results = if let Some(max_results) = &req.max_results {
            max_results.to_string()
        } else {
            "".to_string()
        };
        let action = async {
            let url = format!("{}/b?alt=json&prettyPrint=false", BASE_URL);
            let mut query_param = vec![(("project", project))];
            with_projection(&mut query_param, req.projection);
            if let Some(page_token) = &req.page_token {
                query_param.push(("pageToken", page_token))
            }
            if let Some(prefix) = &req.prefix {
                query_param.push(("prefix", prefix))
            }
            if !max_results.is_empty() {
                query_param.push(("prefix", max_results.as_str()))
            }
            let builder = self.with_headers(reqwest::Client::new().get(url)).await?;
            send(builder.query(&query_param)).await
        };
        invoke(cancel, action).await
    }

    pub async fn patch_bucket(
        &self,
        bucket: &str,
        project: &str,
        req: &PatchBucketRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Bucket, Error> {
        let action = async {
            let url = format!("{}/b/{}?alt=json&prettyPrint=false", BASE_URL, bucket);
            let mut query_param = vec![("project", project)];
            with_projection(&mut query_param, req.projection);
            with_acl(&mut query_param, req.predefined_acl, req.predefined_default_object_acl);
            let builder = self.with_headers(reqwest::Client::new().patch(url)).await?;
            send(builder.query(&query_param).json(&req.metadata)).await
        };
        invoke(cancel, action).await
    }

    pub async fn get_iam_policy(
        &self,
        req: &GetIamPolicyRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Policy, Error> {
        let version = if let Some(version) = &req.requested_policy_version {
            version.to_string()
        } else {
            "".to_string()
        };
        let action = async {
            let url = format!("{}/b/{}/iam?alt=json&prettyPrint=false", BASE_URL, req.resource);
            let mut query_param = vec![];
            if !version.is_empty() {
                query_param.push(("optionsRequestedPolicyVersion", version.as_str()));
            }
            let builder = self.with_headers(reqwest::Client::new().get(url)).await?;
            send(builder.query(&query_param)).await
        };
        invoke(cancel, action).await
    }

    pub async fn test_iam_permission(
        &self,
        req: &TestIamPermissionsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<TestIamPermissionsResponse, Error> {
        let action = async {
            let url = format!("{}/b/{}/iam/testPermissions?alt=json&prettyPrint=false", BASE_URL, req.resource);
            let mut query_param = vec![];
            for permission in &req.permissions {
                query_param.push(("permissions", permission));
            }
            let builder = self.with_headers(reqwest::Client::new().get(url)).await?;
            send(builder.query(&query_param)).await
        };
        invoke(cancel, action).await
    }
}

async fn send<T: for<'de> serde::Deserialize<'de>>(builder: RequestBuilder) -> Result<T,Error> {
    let response = builder.send().await?;
    if response.status().is_success() {
        Ok(response.json().await?)
    } else {
        Err(map_error(response).await)
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
