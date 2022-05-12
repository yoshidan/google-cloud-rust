use crate::http::storage_client::StorageClient;
use crate::bucket::BucketHandle;
use google_cloud_auth::credentials::CredentialsFile;
use google_cloud_auth::{create_token_source_from_credentials, Config};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use crate::http;
use crate::http::entity::{Bucket, ListBucketsRequest};

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

    pub async fn buckets(&self, prefix: Option<String>, cancel: Option<CancellationToken>) -> Result<Vec<Bucket>, http::storage_client::Error> {
        let mut result :Vec<Bucket> = vec![];
        let mut page_token = None;
        loop {
            let req = ListBucketsRequest {
                max_results: None,
                prefix: prefix.clone(),
                page_token,
                projection: None,
            };
            let response = self.storage_client.list_buckets(self.project_id.as_str(), &req, cancel.clone()).await?;
            result.extend(response.items);
            if response.next_page_token.is_none() {
                break;
            }else {
                page_token = response.next_page_token;
            }
        }
        return Ok(result)
    }

}

#[cfg(test)]
mod test {
    use crate::client;
    use chrono::{DateTime, Utc};
    use google_cloud_auth::credentials::CredentialsFile;
    use crate::http::CancellationToken;
    use serial_test::serial;
    use std::collections::HashMap;
    use std::time;
    use std::time::Duration;
    use tracing::Level;
    use serde_json;
    use crate::bucket::BucketHandle;
    use crate::http::entity::bucket::iam_configuration::{PublicAccessPrevention, UniformBucketLevelAccess};
    use crate::http::entity::bucket::{Billing, Cors, Encryption, IamConfiguration, Lifecycle, Logging, RetentionPolicy, Versioning, Website};
    use crate::http::entity::{Bucket, BucketAccessControl, BucketCreationConfig, BucketPatchConfig, InsertBucketRequest, ObjectAccessControl, ObjectAccessControlsCreationConfig, PatchBucketRequest, RetentionPolicyCreationConfig};
    use crate::http::entity::bucket::lifecycle::Rule;
    use crate::http::entity::bucket::lifecycle::rule::{Action, ActionType, Condition};
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
    async fn create_with_setting() {
        let mut labels = HashMap::new();
        labels.insert("labelkey".to_string(), "labelvalue".to_string());
        let config = BucketCreationConfig {
            location: "US".to_string(),
            storage_class: "STANDARD".to_string(),
            default_event_based_hold: true,
            labels: Some(labels),
            website: Some(Website {
                main_page_suffix: "_suffix".to_string(),
                not_found_page: "notfound.html".to_string()
            }),
            billing: Some(Billing {
                requester_pays: true
            }),
            retention_policy: Some(RetentionPolicyCreationConfig {
                retention_period: 10000
            }),
            default_object_acl: Some(vec![
                ObjectAccessControlsCreationConfig {
                    entity: "allUsers".to_string(),
                    role: "READER".to_string(),
                }
            ]),
            cors: Some(vec![Cors {
                origin: vec!["*".to_string()],
                method: vec!["GET".to_string(), "HEAD".to_string()],
                response_header: vec!["200".to_string()],
                max_age_seconds: 100
            }]),
            lifecycle: Some(Lifecycle {
                rule: vec![Rule {
                    action: Some(Action {
                        r#type: ActionType::Delete,
                        storage_class: None
                    }),
                    condition: Some(Condition {
                        age: 365,
                        created_before: None,
                        is_live: Some(true),
                        num_newer_versions: None,
                        matches_storage_class: None,
                        days_since_custom_time: None,
                        custom_time_before: None,
                        days_since_noncurrent_time: None,
                        noncurrent_time_before: None,
                    })
                }]
            }),
            rpo: Some("DEFAULT".to_string()),
            ..Default::default()
        };
        let result = do_create(&mut InsertBucketRequest {
            bucket: config,
            ..Default::default()
        }).await;
        assert!(result.acl.is_some());
        assert!(!result.acl.unwrap().is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn create_authenticated() {
        let mut req = InsertBucketRequest {
            //認証ユーザのみ、きめ細かい管理
            predefined_acl: Some(PredefinedBucketAcl::BucketAclAuthenticatedRead),
            bucket: BucketCreationConfig {
                location: "ASIA-NORTHEAST1".to_string(),
                storage_class: "STANDARD".to_string(),
                ..Default::default()
            },
            ..Default::default()
        };
        let result = do_create(&mut req).await;
        assert!(result.acl.is_some());
        assert!(!result.acl.unwrap().is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn create_public_uniform() {
        let mut req = InsertBucketRequest {
            predefined_acl: Some(PredefinedBucketAcl::BucketAclPublicRead),
            bucket: BucketCreationConfig {
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
            },
            ..Default::default()
        };
        let result = do_create(&mut req).await;
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
        let result = do_create(&mut InsertBucketRequest {
            bucket: config,
            ..Default::default()
        }).await;
        assert!(result.acl.is_none());
    }

    #[tokio::test]
    #[serial]
    async fn create_objectacl_versioned() {
        let mut req = InsertBucketRequest {
            bucket: BucketCreationConfig {
                location: "ASIA-NORTHEAST1".to_string(),
                storage_class: "STANDARD".to_string(),
                versioning: Some(Versioning
                {
                    enabled: true
                }),
                ..Default::default()
            },
            ..Default::default()
        };
        let result = do_create(&mut req).await;
        assert!(result.acl.is_none());
    }

    async fn do_create(req: &mut InsertBucketRequest) -> Bucket {
        let bucket_name = format!("rust-test-{}", chrono::Utc::now().timestamp());
        let client = client::Client::new().await.unwrap();
        let bucket = client.bucket(&bucket_name).await;
        let result = bucket
            .insert(req, Some(CancellationToken::default()))
            .await.unwrap();
        bucket.delete(None).await;
        assert_eq!(result.name, bucket_name);
        assert_eq!(result.storage_class, req.bucket.storage_class);
        assert_eq!(result.location, req.bucket.location);
        return result
    }

    #[tokio::test]
    #[serial]
    async fn get_bucket() {
        let bucket_name = "atl-dev1-test";
        let client = client::Client::new().await.unwrap();
        let bucket = client.bucket(bucket_name).await;
        let result = bucket.get(None).await.unwrap();
        assert_eq!(result.name, bucket_name);
    }

    #[tokio::test]
    #[serial]
    async fn buckets() {
        let prefix = Some("atl-dev1-test".to_string());
        let client = client::Client::new().await.unwrap();
        let result = client.buckets(prefix, None).await.unwrap();
        assert_eq!(result.len(), 1);
        let result2= client.buckets(None, None).await.unwrap();
        assert!(result2.len() > 1);
    }

    #[tokio::test]
    #[serial]
    async fn patch_bucket() {
        let bucket_name = "atl-dev1-test";
        let client = client::Client::new().await.unwrap();
        let bucket = client.bucket(bucket_name).await;
        let req = BucketPatchConfig {
            retention_policy: Some(RetentionPolicyCreationConfig {
                retention_period: 1000
            }),
            ..Default::default()
        };
        let result = bucket.patch(&PatchBucketRequest {
            metadata: Some(req),
            ..Default::default()
        }, None).await.unwrap();
        assert_eq!(result.name, bucket_name);
    }
}
