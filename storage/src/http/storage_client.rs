use std::cmp::max;
use crate::http::{buckets, CancellationToken, channels, default_object_access_controls, Error};
use google_cloud_auth::token_source::TokenSource;
use google_cloud_metadata::project_id;
use reqwest::{Client, RequestBuilder, Response};
use std::collections::HashMap;
use std::future::Future;
use std::iter::Cycle;
use std::mem;
use std::sync::Arc;
use tracing::info;
use crate::http::bucket_access_controls::insert::InsertBucketAccessControlsRequest;
use crate::http::buckets::{Bucket, Policy};
use crate::http::buckets::delete::DeleteBucketRequest;
use crate::http::buckets::get::GetBucketRequest;
use crate::http::buckets::get_iam_policy::GetIamPolicyRequest;
use crate::http::buckets::insert::InsertBucketRequest;
use crate::http::buckets::list::{ListBucketsRequest, ListBucketsResponse};
use crate::http::buckets::list_channels::{ListChannelsRequest, ListChannelsResponse};
use crate::http::buckets::patch::PatchBucketRequest;
use crate::http::buckets::set_iam_policy::SetIamPolicyRequest;
use crate::http::buckets::test_iam_permissions::{TestIamPermissionsRequest, TestIamPermissionsResponse};
use crate::http::channels::stop::StopChannelRequest;
use crate::http::channels::WatchableChannel;
use crate::http::default_object_access_controls::delete::DeleteDefaultObjectAccessControlRequest;
use crate::http::default_object_access_controls::get::GetDefaultObjectAccessControlRequest;
use crate::http::default_object_access_controls::insert::InsertDefaultObjectAccessControlRequest;
use crate::http::default_object_access_controls::list::{ListDefaultObjectAccessControlsRequest, ListDefaultObjectAccessControlsResponse};
use crate::http::object_access_controls::ObjectAccessControl;

pub const SCOPES: [&str; 2] = [
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/devstorage.full_control",
];

#[derive(Clone)]
pub(crate) struct StorageClient {
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
            let builder = buckets::delete::build(&Client::new(), &req);
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
            let builder = buckets::insert::build(&Client::new(), &req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Gets the bucket.
    pub async fn get_bucket(
        &self,
        req: &GetBucketRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Bucket, Error> {
        let action = async {
            let builder = buckets::get::build(&Client::new(), &req);
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
            let builder = buckets::patch::build(&Client::new(), &req);
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
            let builder = buckets::list::build(&Client::new(), &req);
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
            let builder = buckets::set_iam_policy::build(&Client::new(), &req);
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
            let builder = buckets::get_iam_policy::build(&Client::new(), &req);
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
            let builder = buckets::test_iam_permissions::build(&Client::new(), &req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Lists the channels.
    pub async fn list_channels(
        &self,
        req: &ListChannelsRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ListChannelsResponse, Error> {
        let action = async {
            let builder = buckets::list_channels::build(&Client::new(), &req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Stops the channel.
    pub async fn stop_channel(
        &self,
        req: &StopChannelRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let action = async {
            let builder = channels::stop::build(&Client::new(), &req);
            self.send_get_empty(builder).await
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
            let builder = default_object_access_controls::list::build(&Client::new(), &req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Gets the default object ACL.
    pub async fn get_default_object_access_controls(
        &self,
        req: &GetDefaultObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        let action = async {
            let builder = default_object_access_controls::get::build(&Client::new(), &req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Inserts the default object ACL.
    pub async fn insert_default_object_access_controls(
        &self,
        req: &InsertDefaultObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        let action = async {
            let builder = default_object_access_controls::insert::build(&Client::new(), &req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Patchs the default object ACL.
    pub async fn patch_default_object_access_controls(
        &self,
        req: &PatchBucketRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<ObjectAccessControl, Error> {
        let action = async {
            let builder = default_object_access_controls::patch::build(&Client::new(), &req);
            self.send(builder).await
        };
        invoke(cancel, action).await
    }

    /// Deletes the default object ACL.
    pub async fn delete_default_object_access_controls(
        &self,
        req: &DeleteDefaultObjectAccessControlRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<(), Error> {
        let action = async {
            let builder = default_object_access_controls::delete::build(&Client::new(), &req);
            self.send_get_empty(builder).await
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
        let request = self.with_headers(builder).await?;
        let response = request.send().await?;
        if response.status().is_success() {
            let text = response.text().await?;
            tracing::trace!("{}", text);
            Ok(serde_json::from_str(&text).unwrap())
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

#[cfg(test)]
mod test {
    use std::sync::Arc;
    use google_cloud_auth::{Config, create_token_source};
    use crate::http::buckets::delete::DeleteBucketRequest;
    use crate::http::storage_client::{SCOPES, StorageClient};
    use serial_test::serial;
    use crate::http::bucket_access_controls::PredefinedBucketAcl;
    use crate::http::buckets::{Binding, Bucket, Policy};
    use crate::http::buckets::get::GetBucketRequest;
    use crate::http::buckets::get_iam_policy::GetIamPolicyRequest;
    use crate::http::buckets::insert::{BucketCreationConfig, InsertBucketParam, InsertBucketRequest, RetentionPolicyCreationConfig};
    use crate::http::buckets::list::ListBucketsRequest;
    use crate::http::buckets::patch::{BucketPatchConfig, PatchBucketRequest};
    use crate::http::buckets::set_iam_policy::SetIamPolicyRequest;
    use crate::http::buckets::test_iam_permissions::TestIamPermissionsRequest;
    use crate::http::object_access_controls::insert::ObjectAccessControlsCreationConfig;
    use crate::http::object_access_controls::{ObjectACLRole, PredefinedObjectAcl};

    const PROJECT : &str = "atl-dev1";

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt::try_init();
    }

    async fn client() -> StorageClient {
        let ts  = create_token_source(Config {
            audience: None,
            scopes: Some(&SCOPES)
        }).await.unwrap();
        return StorageClient::new(Arc::from(ts));
    }

    #[tokio::test]
    #[serial]
    pub async fn list_buckets() {
        let client = client().await;
        let buckets = client.list_buckets(&ListBucketsRequest {
            project: PROJECT.to_string(),
            max_results: None,
            page_token: None,
            prefix: Some("rust-iam-test".to_string()),
            projection: None
        }, None).await.unwrap();
        assert_eq!(1, buckets.items.len());
    }

    #[tokio::test]
    #[serial]
    pub async fn crud_bucket() {
        let client = client().await;
        let name = format!("rust-test-insert-{}", chrono::Utc::now().timestamp()) ;
        let bucket = client.insert_bucket(&InsertBucketRequest {
            name,
            param: InsertBucketParam {
                project: PROJECT.to_string(),
                ..Default::default()
            },
            bucket: BucketCreationConfig {
                location: "ASIA-NORTHEAST1".to_string(),
                storage_class: Some("STANDARD".to_string()),
                ..Default::default()
            }
        }, None).await.unwrap();

        let found = client.get_bucket(&GetBucketRequest {
            bucket: bucket.name.to_string(),
           ..Default::default()
        }, None).await.unwrap();

        assert_eq!(found.location.as_str(), "ASIA-NORTHEAST1");

        let patched = client.patch_bucket(&PatchBucketRequest {
            bucket: bucket.name.to_string(),
            metadata: Some(BucketPatchConfig {
                default_object_acl: Some(vec![ObjectAccessControlsCreationConfig {
                    entity: "allAuthenticatedUsers".to_string(),
                    role: ObjectACLRole::READER,
                }]),
                ..Default::default()
            }),
            ..Default::default()
        }, None).await.unwrap();

        let default_object_acl = patched.default_object_acl.unwrap();
        assert_eq!(default_object_acl.len(), 1);
        assert_eq!(default_object_acl[0].entity.as_str(), "allAuthenticatedUsers");
        assert_eq!(default_object_acl[0].role, ObjectACLRole::READER);
        assert_eq!(found.storage_class.as_str(), patched.storage_class.as_str());
        assert_eq!(found.location.as_str(), patched.location.as_str());

        client.delete_bucket(&DeleteBucketRequest {
            bucket: bucket.name,
            param: Default::default()
        }, None).await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn set_get_test_iam() {
        let bucket_name = "rust-iam-test";
        let client = client().await;
        let mut policy = client.get_iam_policy(&GetIamPolicyRequest {
            resource: bucket_name.to_string(),
            options_requested_policy_version: None
        }, None).await.unwrap();
        policy.bindings.push(Binding {
            role: "roles/storage.objectViewer".to_string(),
            members: vec!["allAuthenticatedUsers".to_string()],
            condition: None
        });

        let mut result = client.set_iam_policy(&SetIamPolicyRequest {
            resource: bucket_name.to_string(),
            policy,
        }, None).await.unwrap();
        assert_eq!(result.bindings.len(), 5);
        assert_eq!(result.bindings.pop().unwrap().role, "roles/storage.objectViewer");

        let permissions = client.test_iam_permissions(&TestIamPermissionsRequest {
            resource: bucket_name.to_string(),
            permissions: vec!["storage.buckets.get".to_string()],
        }, None).await.unwrap();
        assert_eq!(permissions.permissions[0], "storage.buckets.get");
    }
}