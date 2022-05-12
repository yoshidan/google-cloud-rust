use std::fs::Permissions;
use crate::http::iam::{GetIamPolicyRequest, Policy, SetIamPolicyRequest, TestIamPermissionsRequest};
use crate::http::storage_client::{Error, StorageClient};
use tokio_util::sync::CancellationToken;

pub struct IAMHandle<'a> {
    name: &'a str,
    storage_client: &'a StorageClient,
}

impl<'a> IAMHandle<'a> {
    pub(crate) fn new(name: &'a str, storage_client: &'a StorageClient) -> Self {
        Self { name, storage_client }
    }

    pub async fn get(&self, cancel: Option<CancellationToken>) -> Result<Policy, Error> {
        let req = GetIamPolicyRequest {
            resource: self.name.to_string(),
            requested_policy_version: None,
        };
        self.storage_client.get_iam_policy(&req, cancel).await
    }

    pub async fn set(&self, policy: Policy, cancel: Option<CancellationToken>) -> Result<Policy, Error> {
        let req = SetIamPolicyRequest {
            resource: self.name.to_string(),
            policy
        };
        self.storage_client.set_iam_policy(&req, cancel).await
    }

    pub async fn test(&self, permissions: &[&str], cancel: Option<CancellationToken>) -> Result<Vec<String>, Error> {
        let req = TestIamPermissionsRequest {
            resource: self.name.to_string(),
            permissions: permissions.iter().map(|v| v.to_string()).collect()
        };
        let result = self.storage_client.test_iam_permission(&req, cancel).await?;
        return Ok(result.permissions);
    }
}

#[cfg(test)]
mod test {
    use crate::bucket::BucketHandle;
    use crate::client;
    use crate::http::entity::bucket::iam_configuration::{PublicAccessPrevention, UniformBucketLevelAccess};
    use crate::http::entity::bucket::lifecycle::rule::{Action, ActionType, Condition};
    use crate::http::entity::bucket::lifecycle::Rule;
    use crate::http::entity::bucket::{
        Billing, Cors, Encryption, IamConfiguration, Lifecycle, Logging, RetentionPolicy, Versioning, Website,
    };
    use crate::http::entity::common_enums::PredefinedBucketAcl;
    use crate::http::entity::{
        Bucket, BucketAccessControl, BucketCreationConfig, BucketPatchConfig, InsertBucketRequest, ObjectAccessControl,
        ObjectAccessControlsCreationConfig, PatchBucketRequest, RetentionPolicyCreationConfig,
    };
    use crate::http::CancellationToken;
    use chrono::{DateTime, Utc};
    use google_cloud_auth::credentials::CredentialsFile;
    use serde_json;
    use serial_test::serial;
    use std::collections::HashMap;
    use std::time;
    use std::time::Duration;
    use tokio::sync::OnceCell;
    use tracing::{info, Level};
    use crate::http::iam::Binding;

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt::try_init();
    }

    #[tokio::test]
    #[serial]
    async fn get() {
        let client = client::Client::new().await.unwrap();
        let bucket = client.bucket( "rust-iam-test");
        let iam = bucket.iam();
        let policy = iam.get(None).await.unwrap();
        info!("{:?}", serde_json::to_string(&policy));
        assert_eq!(policy.version, 1);
    }

    #[tokio::test]
    #[serial]
    async fn set() {
        let client = client::Client::new().await.unwrap();
        let bucket = client.bucket( "rust-iam-test");
        let iam = bucket.iam();
        let mut policy = iam.get(None).await.unwrap();
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
        let client = client::Client::new().await.unwrap();
        let bucket = client.bucket( "rust-iam-test");
        let iam = bucket.iam();
        let permissions = iam.test(&vec!["storage.buckets.get"], None).await.unwrap();
        info!("{:?}", permissions);
        assert!(!permissions.is_empty());
        assert_eq!(permissions[0], "storage.buckets.get");
    }
}
