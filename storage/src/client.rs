use crate::http::storage_client::StorageClient;
use crate::http::{storage_client, BASE_URL};

use google_cloud_auth::{create_token_source_from_project, Config, Project};
use std::ops::Deref;
use std::sync::Arc;

use crate::sign::{signed_url, SignBy, SignedURLError, SignedURLOptions};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Auth(#[from] google_cloud_auth::error::Error),
    #[error(transparent)]
    Metadata(#[from] google_cloud_metadata::Error),
    #[error("error: {0}")]
    Other(&'static str),
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
    /// New client
    pub async fn default() -> Result<Self, Error> {
        let project = google_cloud_auth::project().await?;
        Self::new(&project, None).await
    }

    /// New client from project
    pub async fn new(project: &google_cloud_auth::Project, endpoint: Option<&str>) -> Result<Self, Error> {
        let endpoint = endpoint.unwrap_or(BASE_URL);
        let ts = create_token_source_from_project(
            project,
            Config {
                audience: None,
                scopes: Some(&storage_client::SCOPES),
            },
        )
        .await?;
        match project {
            Project::FromFile(cred) => Ok(Client {
                private_key: cred.private_key.clone(),
                service_account_email: cred
                    .client_email
                    .as_ref()
                    .ok_or(Error::Other("no client_email was found"))?
                    .to_string(),
                project_id: cred
                    .project_id
                    .as_ref()
                    .ok_or(Error::Other("no project_id was found"))?
                    .to_string(),
                storage_client: StorageClient::new(Arc::from(ts), endpoint),
            }),
            Project::FromMetadataServer(info) => Ok(Client {
                private_key: None,
                service_account_email: google_cloud_metadata::email("default").await?,
                project_id: info
                    .project_id
                    .as_ref()
                    .ok_or(Error::Other("no project_id was found"))?
                    .to_string(),
                storage_client: StorageClient::new(Arc::from(ts), endpoint),
            }),
        }
    }

    /// Gets the project_id from Credentials
    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    /// Get signed url.
    /// SignedURL returns a URL for the specified object. Signed URLs allow anyone
    /// access to a restricted resource for a limited time without needing a
    /// Google account or signing in. For more information about signed URLs, see
    /// https://cloud.google.com/storage/docs/accesscontrol#signed_urls_query_string_authentication
    ///
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::sign::{SignedURLOptions, SignedURLMethod};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = Client::default().await.unwrap();
    ///     let url_for_download = client.signed_url("bucket", "file.txt", SignedURLOptions::default()).await;
    ///     let url_for_upload = client.signed_url("bucket", "file.txt", SignedURLOptions {
    ///         method: SignedURLMethod::PUT,
    ///         ..Default::default()
    ///     }).await;
    /// }
    /// ```
    #[cfg(not(feature = "trace"))]
    pub async fn signed_url(
        &self,
        bucket: &str,
        object: &str,
        opts: SignedURLOptions,
    ) -> Result<String, SignedURLError> {
        self._signed_url(bucket, object, opts).await
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn signed_url(
        &self,
        bucket: &str,
        object: &str,
        opts: SignedURLOptions,
    ) -> Result<String, SignedURLError> {
        self._signed_url(bucket, object, opts).await
    }

    #[inline(always)]
    async fn _signed_url(&self, bucket: &str, object: &str, opts: SignedURLOptions) -> Result<String, SignedURLError> {
        let signable = match &opts.sign_by {
            SignBy::PrivateKey(v) => !v.is_empty(),
            _ => true,
        };
        if !opts.google_access_id.is_empty() && signable {
            return signed_url(bucket, object, opts);
        }

        let mut opts = opts;
        if let Some(private_key) = &self.private_key {
            opts.sign_by = SignBy::PrivateKey(private_key.as_bytes().to_vec());
        }
        if !self.service_account_email.is_empty() && opts.google_access_id.is_empty() {
            opts.google_access_id = self.service_account_email.to_string();
        }
        signed_url(bucket, object, opts)
    }
}

#[cfg(test)]
mod test {
    use crate::client::Client;

    use crate::http::buckets::delete::DeleteBucketRequest;
    use crate::http::buckets::iam_configuration::{PublicAccessPrevention, UniformBucketLevelAccess};
    use crate::http::buckets::insert::{
        BucketCreationConfig, InsertBucketParam, InsertBucketRequest, RetentionPolicyCreationConfig,
    };
    use crate::http::buckets::{lifecycle, Billing, Cors, IamConfiguration, Lifecycle, Website};
    use serial_test::serial;
    use std::collections::HashMap;

    use crate::http::buckets::list::ListBucketsRequest;
    use crate::sign::{SignedURLMethod, SignedURLOptions};

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt::try_init();
    }

    #[tokio::test]
    #[serial]
    async fn buckets() {
        let prefix = Some("rust-bucket-test".to_string());
        let client = Client::default().await.unwrap();
        let result = client
            .list_buckets(
                &ListBucketsRequest {
                    project: client.project_id().to_string(),
                    prefix,
                    ..Default::default()
                },
                None,
            )
            .await
            .unwrap();
        assert_eq!(result.items.len(), 1);
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
            billing: Some(Billing { requester_pays: false }),
            retention_policy: Some(RetentionPolicyCreationConfig {
                retention_period: 10000,
            }),
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
            rpo: None,
            ..Default::default()
        };

        let client = Client::default().await.unwrap();
        let bucket_name = format!("rust-test-{}", chrono::Utc::now().timestamp());
        let req = InsertBucketRequest {
            name: bucket_name.clone(),
            param: InsertBucketParam {
                project: client.project_id().to_string(),
                ..Default::default()
            },
            bucket: config,
        };
        let result = client.insert_bucket(&req, None).await.unwrap();
        client
            .delete_bucket(
                &DeleteBucketRequest {
                    bucket: result.name.to_string(),
                    ..Default::default()
                },
                None,
            )
            .await
            .unwrap();
        assert_eq!(result.name, bucket_name);
        assert_eq!(result.storage_class, req.bucket.storage_class.unwrap());
        assert_eq!(result.location, req.bucket.location);
        assert!(result.iam_configuration.is_some());
        assert!(
            result
                .iam_configuration
                .unwrap()
                .uniform_bucket_level_access
                .unwrap()
                .enabled
        );
    }

    #[tokio::test]
    #[serial]
    async fn sign() {
        let client = Client::default().await.unwrap();
        let bucket_name = "rust-object-test";
        let data = "aiueo";
        let content_type = "application/octet-stream";

        // upload
        let option = SignedURLOptions {
            method: SignedURLMethod::PUT,
            content_type: Some(content_type.to_string()),
            ..SignedURLOptions::default()
        };
        let url = client
            .signed_url(bucket_name, "signed_uploadtest", option)
            .await
            .unwrap();
        println!("uploading={:?}", url);
        let request = reqwest::Client::default()
            .put(url)
            .header("content-type", content_type)
            .body(data.as_bytes());
        let result = request.send().await.unwrap();
        let status = result.status();
        assert!(status.is_success(), "{:?}", result.text().await.unwrap());

        //download
        let option = SignedURLOptions {
            content_type: Some(content_type.to_string()),
            ..SignedURLOptions::default()
        };
        let url = client
            .signed_url(bucket_name, "signed_uploadtest", option)
            .await
            .unwrap();
        println!("downloading={:?}", url);
        let result = reqwest::Client::default()
            .get(url)
            .header("content-type", content_type)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        assert_eq!(result, data);
    }
}
