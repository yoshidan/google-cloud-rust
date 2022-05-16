use std::fs::Permissions;
use crate::http::iam::{GetIamPolicyRequest, Policy, SetIamPolicyRequest, TestIamPermissionsRequest};
use crate::http::storage_client::{Error, StorageClient};
use tokio_util::sync::CancellationToken;
use crate::http::buckets::get_iam_policy::GetIamPolicyRequest;
use crate::http::buckets::Policy;
use crate::http::buckets::set_iam_policy::SetIamPolicyRequest;
use crate::http::buckets::test_iam_permissions::TestIamPermissionsRequest;

pub struct IAMHandle<'a> {
    name: &'a str,
    storage_client: &'a StorageClient,
}

impl<'a> IAMHandle<'a> {
    pub(crate) fn new(name: &'a str, storage_client: &'a StorageClient) -> Self {
        Self { name, storage_client }
    }

    /// Gets the iam policy
    pub async fn get(&self, options_requested_policy_version: Option<i32>, cancel: Option<CancellationToken>) -> Result<Policy, Error> {
        let req = GetIamPolicyRequest {
            resource: self.name.to_string(),
            options_requested_policy_version
        };
        self.storage_client.get_iam_policy(&req, cancel).await
    }

    /// Sets the iam policy
    pub async fn set(&self, policy: impl Into<Policy>, cancel: Option<CancellationToken>) -> Result<Policy, Error> {
        let req = SetIamPolicyRequest {
            resource: self.name.to_string(),
            policy
        };
        self.storage_client.set_iam_policy(&req, cancel).await
    }

    /// Tests the iam policy
    pub async fn test(&self, permissions: impl Into<Vec<String>>, cancel: Option<CancellationToken>) -> Result<Vec<String>, Error> {
        let req = TestIamPermissionsRequest {
            resource: self.name.to_string(),
            permissions: permissions.into()
        };
        Ok(self.storage_client.test_iam_permissions(&req, cancel).await?.permissions)
    }
}

#[cfg(test)]
mod test {
    use crate::bucket::BucketHandle;
    use crate::client;
    use crate::http::old_entity::bucket::iam_configuration::{PublicAccessPrevention, UniformBucketLevelAccess};
    use crate::http::old_entity::bucket::lifecycle::rule::{Action, ActionType, Condition};
    use crate::http::old_entity::bucket::lifecycle::Rule;
    use crate::http::old_entity::bucket::{
        Billing, Cors, Encryption, IamConfiguration, Lifecycle, Logging, RetentionPolicy, Versioning, Website,
    };
    use crate::http::old_entity::common_enums::PredefinedBucketAcl;
    use crate::http::old_entity::{
        Bucket, BucketAccessControl, BucketCreationConfig, BucketPatchConfig, InsertBucketRequest, ObjectAccessControl,
        ObjectAccessControlsCreationConfig, PatchBucketRequest, RetentionPolicyCreationConfig,
    };
    use crate::http::CancellationToken;
    use chrono::{DateTime, Utc};
    use google_cloud_auth::credentials::CredentialsFile;
    use serde_json;
    use serial_test::serial;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time;
    use std::time::Duration;
    use tokio::sync::OnceCell;
    use tracing::{info, Level};
    use google_cloud_auth::{Config, create_token_source};
    use crate::http::buckets::Binding;
    use crate::http::iam::Binding;
    use crate::http::storage_client::{SCOPES, StorageClient};
    use crate::iam::IAMHandle;

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt::try_init();
    }

    async fn client() -> StorageClient{
        let ts = create_token_source(Config {
            audience: None,
            scopes: Some(&SCOPES),
        })
            .await
            .unwrap();
        StorageClient::new(Arc::from(ts))
    }

    #[tokio::test]
    #[serial]
    async fn get() {
        let iam = IAMHandle::new("rust-iam-test",  client().await?);
        let policy = iam.get(None, None).await.unwrap();
        info!("{:?}", serde_json::to_string(&policy));
        assert_eq!(policy.version, 1);
    }

    #[tokio::test]
    #[serial]
    async fn set() {
        let iam = IAMHandle::new("rust-iam-test",  client().await?);
        let mut policy = iam.get(None, None).await.unwrap();
        policy.bindings.push(Binding {
            role: "roles/storage.objectViewer".to_string(),
            members: vec!["allAuthenticatedUsers".to_string()],
            condition: None
        });
        let mut result = iam.set(policy, None).await.unwrap();
        info!("{:?}", serde_json::to_string(&result));
        assert_eq!(result.bindings.len(), 5);
        assert_eq!(result.bindings.pop().unwrap().role, "roles/storage.objectViewer");
    }

    #[tokio::test]
    #[serial]
    async fn test() {
        let iam = IAMHandle::new("rust-iam-test",  client().await?);
        let permissions = iam.test(&vec!["storage.buckets.get"], None).await.unwrap();
        info!("{:?}", permissions);
        assert!(!permissions.is_empty());
        assert_eq!(permissions[0], "storage.buckets.get");
    }
}
