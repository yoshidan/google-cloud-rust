use crate::http::entity::common_enums::{PredefinedBucketAcl, PredefinedObjectAcl, Projection};
use crate::http::entity::{Bucket, BucketAccessControl, BucketAccessControlsCreationConfig, Channel, DeleteBucketRequest, GetBucketRequest, HmacKeyMetadata, InsertBucketRequest, ListBucketAccessControlsResponse, ListBucketsRequest, ListBucketsResponse, ListChannelsResponse, ListNotificationsResponse, ListObjectAccessControlsResponse, LockRetentionPolicyRequest, Notification, NotificationCreationConfig, ObjectAccessControl, ObjectAccessControlsCreationConfig, PatchBucketRequest, UpdateBucketRequest};
use crate::http::iam::{GetIamPolicyRequest, Policy, SetIamPolicyRequest, TestIamPermissionsRequest, TestIamPermissionsResponse};
use crate::http::CancellationToken;
use google_cloud_auth::token_source::TokenSource;
use google_cloud_metadata::project_id;
use reqwest::{RequestBuilder, Response};
use std::collections::HashMap;
use std::future::Future;
use std::mem;
use std::sync::Arc;
use tracing::info;
use crate::http::entity::list_channels_response::Items;

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
            let url = format!("{}/b/{}?alt=json&prettyPrint=false", BASE_URL, req.bucket);
            self.send_get_empty(reqwest::Client::new().delete(url)).await
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
            self.send(reqwest::Client::new().post(url).query(&query_param).json(&req.bucket)).await
        };
        invoke(cancel, action).await
    }

    pub async fn get_bucket(&self, req: &GetBucketRequest, cancel: Option<CancellationToken>) -> Result<Bucket, Error> {
        let action = async {
            let url = format!("{}/b/{}?alt=json&prettyPrint=false", BASE_URL, req.bucket);
            let mut query_param = vec![];
            with_projection(&mut query_param, req.projection);
            self.send(reqwest::Client::new().get(url).query(&query_param)).await
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
                query_param.push(("maxResults", max_results.as_str()))
            }
            self.send(reqwest::Client::new().get(url).query(&query_param)).await
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
            self.send(reqwest::Client::new().patch(url).query(&query_param).json(&req.metadata)).await
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
            self.send(reqwest::Client::new().get(url).query(&query_param)).await
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
            self.send(reqwest::Client::new().get(url).query(&query_param)).await
        };
        invoke(cancel, action).await
    }

    pub async fn set_iam_policy(
        &self,
        req: &SetIamPolicyRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Policy, Error> {
        let action = async {
            let url = format!("{}/b/{}/iam?alt=json&prettyPrint=false", BASE_URL, req.resource);
            self.send(reqwest::Client::new().put(url).json(&req.policy)).await
        };
        invoke(cancel, action).await
    }

    pub async fn insert_bucket_acl(
        &self,
        bucket: &str,
        config: &BucketAccessControlsCreationConfig,
        cancel: Option<CancellationToken>,
    ) -> Result<BucketAccessControl, Error> {
        let action = async {
            let url = format!("{}/b/{}/acl?alt=json&prettyPrint=false", BASE_URL, bucket);
            self.send(reqwest::Client::new().post(url).json(config)).await
        };
        invoke(cancel, action).await
    }

    pub async fn get_bucket_acl(
        &self,
        bucket: &str,
        entity: &str,
        cancel: Option<CancellationToken>,
    ) -> Result<BucketAccessControl, Error> {
        let action = async {
            let url = format!("{}/b/{}/acl/{}?alt=json&prettyPrint=false", BASE_URL, bucket, entity);
            self.send(reqwest::Client::new().get(url)).await
        };
        invoke(cancel, action).await
    }

    pub async fn delete_bucket_acl(
        &self,
        bucket: &str,
        entity: &str,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let action = async {
            let url = format!("{}/b/{}/acl/{}?alt=json&prettyPrint=false", BASE_URL, bucket, entity);
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
            let url = format!("{}/b/{}/acl?alt=json&prettyPrint=false", BASE_URL, bucket);
            self.send::<ListBucketAccessControlsResponse>(reqwest::Client::new().get(url)).await
        };
        invoke(cancel, action).await.map(|e| e.items )
    }


    pub async fn insert_notification(
        &self,
        bucket: &str,
        config: &NotificationCreationConfig,
        cancel: Option<CancellationToken>,
    ) -> Result<Notification, Error> {
        let action = async {
            let url = format!("{}/b/{}/notificationConfigs?alt=json&prettyPrint=false", BASE_URL, bucket);
            self.send(reqwest::Client::new().post(url).json(config)).await
        };
        invoke(cancel, action).await
    }

    pub async fn list_notifications(
        &self,
        bucket: &str,
        cancel: Option<CancellationToken>,
    ) -> Result<Vec<Notification>, Error> {
        let action = async {
            let url = format!("{}/b/{}/notificationConfigs?alt=json&prettyPrint=false", BASE_URL, bucket);
            self.send::<ListNotificationsResponse>(reqwest::Client::new().get(url)).await
        };
        invoke(cancel, action).await.map(|e| e.items )
    }

    pub async fn get_notification(
        &self,
        bucket: &str,
        notification: &str,
        cancel: Option<CancellationToken>,
    ) -> Result<Notification, Error> {
        let action = async {
            let url = format!("{}/b/{}/notificationConfigs/{}?alt=json&prettyPrint=false", BASE_URL, bucket, notification);
            self.send(reqwest::Client::new().get(url)).await
        };
        invoke(cancel, action).await
    }

    pub async fn delete_notification(
        &self,
        bucket: &str,
        notification: &str,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let action = async {
            let url = format!("{}/b/{}/notificationConfigs/{}?alt=json&prettyPrint=false", BASE_URL, bucket, notification);
            self.send_get_empty(reqwest::Client::new().delete(url)).await
        };
        invoke(cancel, action).await
    }

    pub async fn list_channels(
        &self,
        bucket: &str,
        cancel: Option<CancellationToken>,
    ) -> Result<Vec<Items>, Error> {
        let action = async {
            let url = format!("{}/b/{}/channels?alt=json&prettyPrint=false", BASE_URL, bucket);
            self.send::<ListChannelsResponse>(reqwest::Client::new().get(url)).await
        };
        invoke(cancel, action).await.map(|e| e.items )
    }

    pub async fn stop_channel(
        &self,
        channel: &Items,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let action = async {
            let url = format!("{}/channels/stop?alt=json&prettyPrint=false", BASE_URL);
            self.send_get_empty(reqwest::Client::new().post(url).json(channel)).await
        };
        invoke(cancel, action).await
    }

    pub async fn insert_default_object_acl(
        &self,
        bucket: &str,
        config: &ObjectAccessControlsCreationConfig,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        let action = async {
            let url = format!("{}/b/{}/defaultObjectAcl?alt=json&prettyPrint=false", BASE_URL, bucket);
            self.send(reqwest::Client::new().post(url).json(config)).await
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
