use crate::bucket;
use crate::http::entity::bucket::{Versioning, Website};
use crate::http::entity::common_enums::{PredefinedBucketAcl, PredefinedObjectAcl, Projection};
use crate::http::entity::{
    Bucket, BucketAccessControl, BucketCreationConfig, DeleteBucketRequest, GetBucketRequest, InsertBucketRequest,
    ListBucketsRequest, ObjectAccessControl, ObjectAccessControlsCreationConfig, PatchBucketRequest,
    RetentionPolicyCreationConfig,
};
use crate::http::storage_client::{Error, StorageClient};
use crate::iam::IAMHandle;
use crate::sign::{signed_url, SignBy, SignedURLError, SignedURLOptions};
use chrono::{DateTime, SecondsFormat, Timelike, Utc};
use google_cloud_auth::credentials::CredentialsFile;
use std::collections::HashMap;
use tokio_util::sync::CancellationToken;
use crate::acl::BucketACLHandle;

pub struct BucketHandle<'a> {
    name: String,
    private_key: &'a str,
    service_account_email: &'a str,
    project_id: &'a str,
    storage_client: StorageClient,
}

impl<'a> BucketHandle<'a> {
    pub(crate) fn new(
        name: String,
        private_key: &'a str,
        service_account_email: &'a str,
        project_id: &'a str,
        storage_client: StorageClient,
    ) -> Self {
        Self {
            name,
            private_key,
            service_account_email,
            project_id,
            storage_client,
        }
    }

    pub async fn signed_url(&self, object: String, opts: &mut SignedURLOptions) -> Result<String, SignedURLError> {
        let signable = match &opts.sign_by {
            SignBy::PrivateKey(v) => !v.is_empty(),
            _ => true,
        };
        if !opts.google_access_id.is_empty() && signable {
            return signed_url(self.name.to_string(), object, opts);
        }

        if !self.private_key.is_empty() {
            opts.sign_by = SignBy::PrivateKey(self.private_key.into());
        }
        if !self.service_account_email.is_empty() && opts.google_access_id.is_empty() {
            opts.google_access_id = self.service_account_email.to_string();
        }
        return signed_url(self.name.to_string(), object, opts);
    }

    pub async fn delete(&self, cancel: Option<CancellationToken>) -> Result<(), Error> {
        let req = DeleteBucketRequest {
            bucket: self.name.to_string(),
            ..Default::default()
        };
        self.storage_client.delete_bucket(req, cancel).await
    }

    pub async fn insert(
        &self,
        req: &mut InsertBucketRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Bucket, Error> {
        req.bucket.name = self.name.to_string();
        self.storage_client.insert_bucket(self.project_id, req, cancel).await
    }

    pub async fn get(&self, cancel: Option<CancellationToken>) -> Result<Bucket, Error> {
        let req = GetBucketRequest {
            bucket: self.name.to_string(),
            ..Default::default()
        };
        self.storage_client.get_bucket(&req, cancel).await
    }

    pub async fn patch(&self, req: &PatchBucketRequest, cancel: Option<CancellationToken>) -> Result<Bucket, Error> {
        self.storage_client
            .patch_bucket(self.name.as_str(), self.project_id, &req, cancel)
            .await
    }

    pub fn iam(&self) -> IAMHandle {
        IAMHandle::new(self.name.as_str(), &self.storage_client)
    }

    pub fn acl<'b>(&self, entity: &'b str) -> BucketACLHandle<'_, 'b> {
        BucketACLHandle::new(self.name.as_str(), entity, &self.storage_client)
    }

    pub async fn acls<'b>(&self, cancel: Option<CancellationToken>) -> Result<Vec<BucketAccessControl>, Error> {
        self.storage_client.list_bucket_acls(self.name.as_str(), cancel).await
    }
}


#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use tokio_util::sync::CancellationToken;
    use crate::client::Client;
    use crate::http::entity::bucket::{Billing, Cors, IamConfiguration, Lifecycle, Versioning, Website};
    use crate::http::entity::{Bucket, BucketCreationConfig, BucketPatchConfig, InsertBucketRequest, ObjectAccessControlsCreationConfig, PatchBucketRequest, RetentionPolicyCreationConfig};
    use crate::http::entity::bucket::iam_configuration::{PublicAccessPrevention, UniformBucketLevelAccess};
    use crate::http::entity::bucket::lifecycle::Rule;
    use crate::http::entity::bucket::lifecycle::rule::{Action, ActionType, Condition};
    use crate::http::entity::common_enums::PredefinedBucketAcl;
    use serial_test::serial;

    #[ctor::ctor]
    fn init() {
        tracing_subscriber::fmt::try_init();
    }

    #[tokio::test]
    #[serial]
    async fn delete() {
        let client = Client::new().await.unwrap();
        let bucket = client.bucket("atl-dev1-test");
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
                not_found_page: "notfound.html".to_string(),
            }),
            billing: Some(Billing { requester_pays: true }),
            retention_policy: Some(RetentionPolicyCreationConfig {
                retention_period: 10000,
            }),
            default_object_acl: Some(vec![ObjectAccessControlsCreationConfig {
                entity: "allUsers".to_string(),
                role: "READER".to_string(),
            }]),
            cors: Some(vec![Cors {
                origin: vec!["*".to_string()],
                method: vec!["GET".to_string(), "HEAD".to_string()],
                response_header: vec!["200".to_string()],
                max_age_seconds: 100,
            }]),
            lifecycle: Some(Lifecycle {
                rule: vec![Rule {
                    action: Some(Action {
                        r#type: ActionType::Delete,
                        storage_class: None,
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
                    }),
                }],
            }),
            rpo: Some("DEFAULT".to_string()),
            ..Default::default()
        };
        let result = do_create(&mut InsertBucketRequest {
            bucket: config,
            ..Default::default()
        })
            .await;
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
                        locked_time: None,
                    }),
                    public_access_prevention: Some(PublicAccessPrevention::Enforced),
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
                    locked_time: None,
                }),
                public_access_prevention: Some(PublicAccessPrevention::Enforced),
            }),
            location: "ASIA-NORTHEAST1".to_string(),
            storage_class: "STANDARD".to_string(),
            ..Default::default()
        };
        let result = do_create(&mut InsertBucketRequest {
            bucket: config,
            ..Default::default()
        })
            .await;
        assert!(result.acl.is_none());
    }

    #[tokio::test]
    #[serial]
    async fn create_objectacl_versioned() {
        let mut req = InsertBucketRequest {
            bucket: BucketCreationConfig {
                location: "ASIA-NORTHEAST1".to_string(),
                storage_class: "STANDARD".to_string(),
                versioning: Some(Versioning { enabled: true }),
                ..Default::default()
            },
            ..Default::default()
        };
        let result = do_create(&mut req).await;
        assert!(result.acl.is_none());
    }

    async fn do_create(req: &mut InsertBucketRequest) -> Bucket {
        let bucket_name = format!("rust-test-{}", chrono::Utc::now().timestamp());
        let client = Client::new().await.unwrap();
        let bucket = client.bucket(&bucket_name);
        let result = bucket.insert(req, Some(CancellationToken::default())).await.unwrap();
        bucket.delete(None).await.unwrap();
        assert_eq!(result.name, bucket_name);
        assert_eq!(result.storage_class, req.bucket.storage_class);
        assert_eq!(result.location, req.bucket.location);
        return result;
    }

    #[tokio::test]
    #[serial]
    async fn get_bucket() {
        let bucket_name = "rust-bucket-test";
        let client = Client::new().await.unwrap();
        let bucket = client.bucket(bucket_name);
        let result = bucket.get(None).await.unwrap();
        assert_eq!(result.name, bucket_name);
    }

    #[tokio::test]
    #[serial]
    async fn patch_bucket() {
        let bucket_name = "rust-bucket-test";
        let client = Client::new().await.unwrap();
        let bucket = client.bucket(bucket_name);
        let req = BucketPatchConfig {
            retention_policy: Some(RetentionPolicyCreationConfig { retention_period: 1000 }),
            ..Default::default()
        };
        let result = bucket
            .patch(
                &PatchBucketRequest {
                    metadata: Some(req),
                    ..Default::default()
                },
                None,
            )
            .await
            .unwrap();
        assert_eq!(result.name, bucket_name);
        assert_eq!(result.retention_policy.unwrap().retention_period, 1000);
    }

    #[tokio::test]
    #[serial]
    async fn acls() {
        let bucket_name = "rust-bucket-acl-test";
        let client = Client::new().await.unwrap();
        let bucket = client.bucket(bucket_name);
        let acls = bucket.acls(None).await.unwrap();
        assert!(!acls.is_empty());
    }
}
