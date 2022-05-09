use crate::apiv1::entity::common_enums::{PredefinedBucketAcl, PredefinedObjectAcl, Projection};
use crate::apiv1::entity::{DeleteBucketRequest, InsertBucketRequest};
use google_cloud_auth::token_source::TokenSource;
use google_cloud_gax::cancel::CancellationToken;
use reqwest::{RequestBuilder, Response};
use std::future::Future;
use std::mem;
use std::sync::Arc;
use tokio::select;

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
            let response = builder.send().await?;
            if response.status().is_success() {
                Ok(())
            } else {
                Err(map_error(response).await)
            }
        };
        invoke(cancel, action).await
    }

    pub async fn insert_bucket(
        &self,
        req: InsertBucketRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let action = async {
            let url = format!("{}/b?alt=json&prettyPrint=false", BASE_URL);
            let mut query_param: Vec<(&str, &str)> = vec![("project", req.project.as_str())];
            if let Some(predefined_acl) = to_predefined_bucket_acl_string(req.predefined_acl) {
                query_param.push(("predefinedAcl", predefined_acl))
            }
            if let Some(predefined_acl) = to_predefined_object_acl_string(req.predefined_default_object_acl) {
                query_param.push(("predefinedDefaultObjectAcl", predefined_acl))
            }
            if let Some(projection) = to_projection_string(req.projection) {
                query_param.push(("projection", projection))
            }
            let builder = self.with_headers(reqwest::Client::new().post(url)).await?;
            let response = builder.query(&query_param).json(&req.bucket).send().await?;
            if response.status().is_success() {
                Ok(())
            } else {
                Err(map_error(response).await)
            }
        };
        invoke(cancel, action).await
    }
}

fn to_projection_string(v: Projection) -> Option<&'static str> {
    ///see common_enums::Projection
    match v {
        Projection::NoAcl => Some("noAcl"),
        Projection::Full => Some("full"),
        _ => None,
    }
}

fn to_predefined_bucket_acl_string(v: PredefinedBucketAcl) -> Option<&'static str> {
    ///see common_enums::PredefinedBucketAcl
    match v {
        PredefinedBucketAcl::BucketAclAuthenticatedRead => Some("allAuthenticatedUsers"),
        PredefinedBucketAcl::BucketAclPrivate => Some("private"),
        PredefinedBucketAcl::BucketAclProjectPrivate => Some("projectPrivate"),
        PredefinedBucketAcl::BucketAclPublicRead => Some("publicRead"),
        PredefinedBucketAcl::BucketAclPublicReadWrite => Some("publicReadWrite"),
        _ => None,
    }
}

fn to_predefined_object_acl_string(v: PredefinedObjectAcl) -> Option<&'static str> {
    ///see common_enums::PredefinedObjectAcl
    match v {
        PredefinedObjectAcl::ObjectAclAuthenticatedRead => Some("allAuthenticatedUsers"),
        PredefinedObjectAcl::ObjectAclBucketOwnerFullControl => Some("bucketOwnerFullControl"),
        PredefinedObjectAcl::ObjectAclBucketOwnerRead => Some("bucketOwnerRead"),
        PredefinedObjectAcl::ObjectAclPrivate => Some("private"),
        PredefinedObjectAcl::ObjectAclProjectPrivate => Some("projectPrivate"),
        PredefinedObjectAcl::ObjectAclPublicRead => Some("publicRead"),
        _ => None,
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
            select! {
                _ = cancel.cancelled() => Err(Error::Cancelled),
                v = action => v
            }
        }
        None => action.await,
    }
}
