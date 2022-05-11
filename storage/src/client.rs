use crate::http::storage_client::StorageClient;
use crate::bucket::BucketHandle;
use google_cloud_auth::credentials::CredentialsFile;
use google_cloud_auth::{create_token_source_from_credentials, Config};
use std::sync::Arc;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    AuthError(#[from] google_cloud_auth::error::Error),
    #[error(transparent)]
    MetadataError(#[from] google_cloud_metadata::Error),
}

pub struct Client {
    private_key: Option<String>,
    service_account_email: String,
    project_id: String,
    storage_client: StorageClient,
}

impl Client {
    pub async fn new() -> Result<Self, Error> {
        const SCOPES: [&str; 2] = [
            "https://www.googleapis.com/auth/cloud-platform",
            "https://www.googleapis.com/auth/devstorage.full_control",
        ];
        let cred = CredentialsFile::new().await?;
        let ts = create_token_source_from_credentials(
            &cred,
            Config {
                audience: None,
                scopes: Some(&SCOPES),
            },
        )
        .await?;
        Ok(Client {
            private_key: cred.private_key,
            service_account_email: match cred.client_email {
                Some(email) => email,
                None => {
                    if google_cloud_metadata::on_gce().await {
                        google_cloud_metadata::email("default").await?
                    } else {
                        "".to_string()
                    }
                }
            },
            project_id: match cred.project_id {
                Some(project_id) => project_id.to_string(),
                None => {
                    if google_cloud_metadata::on_gce().await {
                        google_cloud_metadata::project_id().await.to_string()
                    } else {
                        "".to_string()
                    }
                }
            },
            storage_client: StorageClient::new(Arc::from(ts)),
        })
    }

    pub async fn bucket(&self, name: &str) -> BucketHandle<'_> {
        BucketHandle::new(
            //format!("projects/{}/buckets/{}", self.project_id, name), <- for v2 gRPC API
            name.to_string(),
            match &self.private_key {
                Some(v) => v,
                None => "",
            },
            &self.service_account_email,
            &self.project_id,
            self.storage_client.clone(),
        )
    }
}

#[cfg(test)]
mod test {
    use crate::http::partial::BucketCreationConfig;
    use crate::client;
    use chrono::{DateTime, Utc};
    use google_cloud_auth::credentials::CredentialsFile;
    use google_cloud_gax::cancel::CancellationToken;
    use google_cloud_gax::retry::RetrySetting;
    use serial_test::serial;
    use std::collections::HashMap;
    use std::time;
    use std::time::Duration;
    use tracing::Level;
    use serde_json;
    use crate::bucket::BucketHandle;
    use crate::http::entity::bucket::iam_configuration::{PublicAccessPrevention, UniformBucketLevelAccess};
    use crate::http::entity::bucket::{IamConfiguration, Versioning};
    use crate::http::entity::{Bucket, BucketAccessControl};
    use crate::http::entity::common_enums::PredefinedBucketAcl;

    #[ctor::ctor]
    fn init() {
        tracing_subscriber::fmt::try_init();
    }

    #[tokio::test]
    #[serial]
    async fn new() {
        let client = client::Client::new().await.unwrap();
        assert!(!client.service_account_email.is_empty());
        assert!(client.private_key.is_some());
    }

    #[tokio::test]
    #[serial]
    async fn delete() {
        let client = client::Client::new().await.unwrap();
        let bucket = client.bucket("atl-dev1-test").await;
        let result = bucket.delete(Some(CancellationToken::default())).await;
        assert!(result.is_ok(), "{}", result.unwrap_err())
    }

    #[tokio::test]
    #[serial]
    async fn create_authenticated() {
        let config = BucketCreationConfig {
            //認証ユーザのみ、きめ細かい管理
            predefined_acl: Some(PredefinedBucketAcl::BucketAclAuthenticatedRead),
            location: "ASIA-NORTHEAST1".to_string(),
            storage_class: "STANDARD".to_string(),
            ..Default::default()
        };
        let result = do_create(&config).await;
        assert!(result.acl.is_some());
        assert!(!result.acl.unwrap().is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn create_public_uniform() {
        let config = BucketCreationConfig {
            predefined_acl: Some(PredefinedBucketAcl::BucketAclPublicRead),
            iam_configuration: Some(IamConfiguration {
                uniform_bucket_level_access: Some(UniformBucketLevelAccess {
                    enabled: true,
                    locked_time: None
                }),
                public_access_prevention: Some(PublicAccessPrevention::Enforced)
            }),
            location: "ASIA-NORTHEAST1".to_string(),
            storage_class: "STANDARD".to_string(),
            ..Default::default()
        };
        let result = do_create(&config).await;
        assert!(result.acl.is_some());
        assert!(!result.acl.unwrap().is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn create_private_uniform() {
        let config = BucketCreationConfig {
            iam_configuration: Some(IamConfiguration {
                uniform_bucket_level_access: Some(UniformBucketLevelAccess {
                    enabled: true,
                    locked_time: None
                }),
                public_access_prevention: Some(PublicAccessPrevention::Enforced)
            }),
            location: "ASIA-NORTHEAST1".to_string(),
            storage_class: "STANDARD".to_string(),
            ..Default::default()
        };
        let result = do_create(&config).await;
        assert!(result.acl.is_none());
    }

    #[tokio::test]
    #[serial]
    async fn create_objectacl_versioned() {
        let config = BucketCreationConfig {
            location: "ASIA-NORTHEAST1".to_string(),
            storage_class: "STANDARD".to_string(),
            versioning: Some(Versioning {
                enabled: true
            }),
            ..Default::default()
        };
        let result = do_create(&config).await;
        assert!(result.acl.is_none());
    }

    async fn do_create(config : &BucketCreationConfig) -> Bucket {
        let bucket_name = format!("rust-test-{}", chrono::Utc::now().timestamp());
        let client = client::Client::new().await.unwrap();
        let bucket = client.bucket(&bucket_name).await;
        let result = bucket
            .create(&config, Some(CancellationToken::default()))
            .await.unwrap();
        println!("{:?}", serde_json::to_string(&result));
       // bucket.delete(Some(CancellationToken::default())).await;
        assert_eq!(result.name, bucket_name);
        assert_eq!(result.storage_class, config.storage_class);
        assert_eq!(result.location, config.location);
        return result
    }
}
