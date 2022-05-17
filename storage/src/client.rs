use std::ops::Deref;
use crate::http;
use crate::http::old_entity::{Bucket, ListBucketsRequest};
use crate::http::storage_client::StorageClient;
use google_cloud_auth::credentials::CredentialsFile;
use google_cloud_auth::{create_token_source_from_credentials, Config};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use google_cloud_metadata::project_id;
use crate::sign::{SignBy, signed_url, SignedURLError, SignedURLOptions};

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

impl Deref for Client {
    type Target = StorageClient;

    fn deref(&self) -> &Self::Target {
        &self.storage_client
    }
}

impl Client {
    /// Creates the client from Credentials
    pub async fn from_credentials(cred: &CredentialsFile) -> Result<Self, Error> {
        let ts = create_token_source_from_credentials(
            cred,
            Config {
                audience: None,
                scopes: Some(&StorageClient::SCOPES),
            },
        )
            .await?;
        Ok(Client {
            private_key: cred.private_key.clone(),
            service_account_email: match &cred.client_email {
                Some(email) => email.clone(),
                None => {
                    if google_cloud_metadata::on_gce().await {
                        google_cloud_metadata::email("default").await?
                    } else {
                        "".to_string()
                    }
                }
            },
            project_id: match &cred.project_id {
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

    /// New client
    pub async fn new() -> Result<Self, Error> {
        let cred = CredentialsFile::new().await?;
        Self::from_credentials(&cred).await
    }


    /// Gets the project_id from Credentials
    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    /// Get signed url.
    /// https://github.com/googleapis/google-cloud-go/blob/a33861fe46be42ae150d6015ad39dae6e35e04e8/storage/bucket.go#L271
    ///
    /// SignedURL returns a URL for the specified object. Signed URLs allow anyone
    /// access to a restricted resource for a limited time without needing a
    /// Google account or signing in. For more information about signed URLs, see
    /// https://cloud.google.com/storage/docs/accesscontrol#signed_urls_query_string_authentication
    pub async fn signed_url(&self, bucket: &str, object: &str, opts: SignedURLOptions) -> Result<String, SignedURLError> {
        let signable = match &opts.sign_by {
            SignBy::PrivateKey(v) => !v.is_empty(),
            _ => true,
        };
        if !opts.google_access_id.is_empty() && signable {
            return signed_url(self.bucket.into(), object, opts);
        }

        let mut opts = opts;
        if !self.private_key.is_empty() {
            opts.sign_by = SignBy::PrivateKey(self.private_key.into());
        }
        if !self.service_account_email.is_empty() && opts.google_access_id.is_empty() {
            opts.google_access_id = self.service_account_email.to_string();
        }
        return signed_url(bucket, object, opts);
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use serial_test::serial;
    use crate::client::Client;
    use crate::http::bucket_access_controls::PredefinedBucketAcl;
    use crate::http::buckets::insert::{BucketCreationConfig, InsertBucketParam, InsertBucketRequest, RetentionPolicyCreationConfig};
    use crate::http::buckets::{Billing, Condition, Cors, IamConfiguration, Lifecycle, lifecycle, Website};
    use crate::http::buckets::delete::DeleteBucketRequest;
    use crate::http::buckets::iam_configuration::{PublicAccessPrevention, UniformBucketLevelAccess};
    use crate::http::buckets::lifecycle::Rule;
    use crate::http::buckets::lifecycle::rule::{Action, ActionType};
    use crate::http::buckets::list::ListBucketsRequest;

    #[ctor::ctor]
    fn init() {
        tracing_subscriber::fmt::try_init();
    }

    #[tokio::test]
    #[serial]
    async fn buckets() {
        let prefix = Some("rust-bucket-test".to_string());
        let client = Client::new().await.unwrap();
        let result = client.list_buckets(&ListBucketsRequest {
            project: client.project_id().to_string(),
            prefix,
            ..Default::default()
        }, None).await.unwrap();
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    #[serial]
    async fn create_bucket() {
        let mut labels = HashMap::new();
        labels.insert("labelkey".to_string(), "labelvalue".to_string());
        let config = BucketCreationConfig {
            location: "ASIA-NORTHEAST1".to_string(),
            storage_class: Some("STANDARD".to_string()),
            default_event_based_hold: true,
            labels: Some(labels),
            website: Some(Website {
                main_page_suffix: "_suffix".to_string(),
                not_found_page: "notfound.html".to_string(),
            }),
            iam_configuration: Some(IamConfiguration {
                uniform_bucket_level_access: Some(UniformBucketLevelAccess {
                    enabled: true,
                    locked_time: None,
                }),
                public_access_prevention: Some(PublicAccessPrevention::Enforced),
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
                rule: vec![lifecycle::Rule {
                    action: Some(lifecycle::rule::Action {
                        r#type: lifecycle::rule::ActionType::Delete,
                        storage_class: None,
                    }),
                    condition: Some(lifecycle::rule::Condition {
                        age: 365,
                        is_live: Some(true),
                        ..Default::default()
                    }),
                }],
            }),
            rpo: Some("DEFAULT".to_string()),
            ..Default::default()
        };

        let bucket_name = format!("rust-test-{}", chrono::Utc::now().timestamp());
        let req = InsertBucketRequest {
            name: bucket_name,
            param : InsertBucketParam {
                predefined_acl: Some(PredefinedBucketAcl::BucketAclPublicRead),
                ..Default::default()
            },
            bucket: config,
            ..Default::default()
        };
        let client = Client::new().await.unwrap();
        let result = client.insert_bucket(&req, None).await.unwrap();
        client.delete_bucket(&DeleteBucketRequest {
            bucket: result.lifecycle.to_string(),
            ..Default::default()
        }, None).await.unwrap();
        assert_eq!(result.name, bucket_name);
        assert_eq!(result.storage_class, req.bucket.storage_class);
        assert_eq!(result.location, req.bucket.location);
        assert!(result.acl.is_some());
        assert!(!result.acl.unwrap().is_empty());
    }

}
