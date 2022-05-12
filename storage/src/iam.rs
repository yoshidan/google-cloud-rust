use crate::http::iam::{GetIamPolicyRequest, Policy};
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
    use tracing::{info, Level};

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt::try_init();
    }

    #[tokio::test]
    #[serial]
    async fn get() {
        let client = client::Client::new().await.unwrap();
        let bucket = client.bucket("atl-dev1-test").await;
        let iam = bucket.iam();
        let policy = iam.get(None).await.unwrap();
        assert_eq!(policy.version, 1);
        info!("{:?}", serde_json::to_string(&policy));
    }
}
