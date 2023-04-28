use std::ops::Deref;

use ring::{rand, signature};
use rsa::pkcs8::{DecodePrivateKey, EncodePrivateKey};

use google_cloud_token::{NopeTokenSourceProvider, TokenSourceProvider};

use crate::http::service_account_client::ServiceAccountClient;
use crate::http::storage_client::StorageClient;
use crate::sign::SignBy::PrivateKey;
use crate::sign::{create_signed_buffer, SignBy, SignedURLError, SignedURLOptions};

#[derive(Debug)]
pub struct ClientConfig {
    pub http: Option<reqwest::Client>,
    pub storage_endpoint: String,
    pub service_account_endpoint: String,
    pub token_source_provider: Box<dyn TokenSourceProvider>,
    pub default_google_access_id: Option<String>,
    pub default_sign_by: Option<SignBy>,
    pub project_id: Option<String>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            http: None,
            storage_endpoint: "https://storage.googleapis.com".to_string(),
            token_source_provider: Box::new(NopeTokenSourceProvider {}),
            service_account_endpoint: "https://iamcredentials.googleapis.com".to_string(),
            default_google_access_id: None,
            default_sign_by: None,
            project_id: None,
        }
    }
}

pub struct Client {
    default_google_access_id: Option<String>,
    default_sign_by: Option<SignBy>,
    storage_client: StorageClient,
    service_account_client: ServiceAccountClient,
}

impl Deref for Client {
    type Target = StorageClient;

    fn deref(&self) -> &Self::Target {
        &self.storage_client
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new(ClientConfig::default())
    }
}

impl Client {
    /// New client
    pub fn new(config: ClientConfig) -> Self {
        let ts = config.token_source_provider.token_source();
        let http = config.http.unwrap_or_default();

        let service_account_client =
            ServiceAccountClient::new(ts.clone(), config.service_account_endpoint.as_str(), http.clone());
        let storage_client = StorageClient::new(ts, config.storage_endpoint.as_str(), http);

        Self {
            default_google_access_id: config.default_google_access_id,
            default_sign_by: config.default_sign_by,
            storage_client,
            service_account_client,
        }
    }

    /// Get signed url.
    /// SignedURL returns a URL for the specified object. Signed URLs allow anyone
    /// access to a restricted resource for a limited time without needing a
    /// Google account or signing in. For more information about signed URLs, see
    /// https://cloud.google.com/storage/docs/accesscontrol#signed_urls_query_string_authentication
    ///
    /// Using the client defaults:
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::sign::{SignedURLOptions, SignedURLMethod};
    ///
    /// async fn run(client: Client) {
    ///     let url_for_download = client.signed_url("bucket", "file.txt", None, None, SignedURLOptions::default()).await;
    ///     let url_for_upload = client.signed_url("bucket", "file.txt", None, None, SignedURLOptions {
    ///         method: SignedURLMethod::PUT,
    ///         ..Default::default()
    ///     }).await;
    /// }
    /// ```
    ///
    /// Overwriting the client defaults:
    /// ```
    /// use google_cloud_storage::client::Client;
    /// use google_cloud_storage::sign::{SignBy, SignedURLOptions, SignedURLMethod};
    ///
    /// async fn run(client: Client) {
    /// #   let private_key = SignBy::PrivateKey(vec![]);
    ///
    ///     let url_for_download = client.signed_url("bucket", "file.txt", Some("google_access_id".to_string()), Some(private_key.clone()), SignedURLOptions::default()).await;
    ///     let url_for_upload = client.signed_url("bucket", "file.txt", Some("google_access_id".to_string()), Some(private_key.clone()), SignedURLOptions {
    ///         method: SignedURLMethod::PUT,
    ///         ..Default::default()
    ///     }).await;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn signed_url(
        &self,
        bucket: &str,
        object: &str,
        google_access_id: Option<String>,
        sign_by: Option<SignBy>,
        opts: SignedURLOptions,
    ) -> Result<String, SignedURLError> {
        // use the one from the options or the default one or error out

        let google_access_id = match &google_access_id {
            Some(overwritten_gai) => overwritten_gai.to_owned(),
            None => {
                let default_gai = &self
                    .default_google_access_id
                    .clone()
                    .ok_or(SignedURLError::InvalidOption("No default google_access_id is found"))?;

                default_gai.to_owned()
            }
        };

        // use the one from the options or the default one or error out
        let sign_by = match &sign_by {
            Some(overwritten_sign_by) => overwritten_sign_by.to_owned(),
            None => {
                let default_sign_by = &self
                    .default_sign_by
                    .clone()
                    .ok_or(SignedURLError::InvalidOption("No default google_access_id is found"))?;

                default_sign_by.to_owned()
            }
        };

        let (signed_buffer, mut builder) = create_signed_buffer(bucket, object, &google_access_id, &opts)?;
        tracing::trace!("signed_buffer={:?}", String::from_utf8_lossy(&signed_buffer));

        // create signature
        let signature = match &sign_by {
            PrivateKey(private_key) => {
                // if sign_by is a collection of private keys we check that at least one is present
                if private_key.is_empty() {
                    return Err(SignedURLError::InvalidOption("No keys present"));
                }

                let str = String::from_utf8_lossy(private_key);
                let pkcs = rsa::RsaPrivateKey::from_pkcs8_pem(str.as_ref())
                    .map_err(|e| SignedURLError::CertError(e.to_string()))?;
                let der = pkcs
                    .to_pkcs8_der()
                    .map_err(|e| SignedURLError::CertError(e.to_string()))?;
                let key_pair = ring::signature::RsaKeyPair::from_pkcs8(der.as_ref())
                    .map_err(|e| SignedURLError::CertError(e.to_string()))?;
                let mut signed = vec![0; key_pair.public_modulus_len()];
                key_pair
                    .sign(
                        &signature::RSA_PKCS1_SHA256,
                        &rand::SystemRandom::new(),
                        signed_buffer.as_slice(),
                        &mut signed,
                    )
                    .map_err(|e| SignedURLError::CertError(e.to_string()))?;
                signed
            }
            SignBy::SignBytes => {
                let path = format!("projects/-/serviceAccounts/{}", google_access_id);
                self.service_account_client
                    .sign_blob(&path, signed_buffer.as_slice())
                    .await
                    .map_err(SignedURLError::SignBlob)?
            }
        };
        builder
            .query_pairs_mut()
            .append_pair("X-Goog-Signature", &hex::encode(signature));
        Ok(builder.to_string())
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use serial_test::serial;
    use time::OffsetDateTime;

    use google_cloud_auth::project::Config;
    use google_cloud_auth::token::DefaultTokenSourceProvider;

    use crate::client::{Client, ClientConfig};
    use crate::http::buckets::delete::DeleteBucketRequest;
    use crate::http::buckets::iam_configuration::{PublicAccessPrevention, UniformBucketLevelAccess};
    use crate::http::buckets::insert::{
        BucketCreationConfig, InsertBucketParam, InsertBucketRequest, RetentionPolicyCreationConfig,
    };
    use crate::http::buckets::list::ListBucketsRequest;
    use crate::http::buckets::{lifecycle, Billing, Cors, IamConfiguration, Lifecycle, Website};
    use crate::http::storage_client::SCOPES;
    use crate::sign::{SignBy, SignedURLMethod, SignedURLOptions};

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt::try_init();
    }

    async fn create_client() -> (Client, String) {
        let mut config = ClientConfig::default();
        let ts = DefaultTokenSourceProvider::new(Config {
            audience: None,
            scopes: Some(&SCOPES),
        })
        .await
        .unwrap();

        let cred = &ts.source_credentials.clone().unwrap();
        config.project_id = cred.project_id.clone();
        config.token_source_provider = Box::new(ts);
        config.default_google_access_id = cred.client_email.clone();
        config.default_sign_by = Some(SignBy::PrivateKey(cred.private_key.clone().unwrap().into_bytes()));
        let project_id = config.project_id.clone();
        (Client::new(config), project_id.unwrap())
    }

    #[tokio::test]
    #[serial]
    async fn test_buckets() {
        let prefix = Some("rust-bucket-test".to_string());
        let (client, project) = create_client().await;
        let result = client
            .list_buckets(&ListBucketsRequest {
                project,
                prefix,
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(result.items.len(), 1);
    }

    #[tokio::test]
    #[serial]
    async fn test_create_bucket() {
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

        let (client, project) = create_client().await;
        let bucket_name = format!("rust-test-{}", OffsetDateTime::now_utc().unix_timestamp());
        let req = InsertBucketRequest {
            name: bucket_name.clone(),
            param: InsertBucketParam {
                project,
                ..Default::default()
            },
            bucket: config,
        };
        let result = client.insert_bucket(&req).await.unwrap();
        client
            .delete_bucket(&DeleteBucketRequest {
                bucket: result.name.to_string(),
                ..Default::default()
            })
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
    async fn test_sign() {
        let (client, _) = create_client().await;
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
            .signed_url(bucket_name, "signed_uploadtest", None, None, option)
            .await
            .unwrap();
        println!("uploading={url:?}");
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
            .signed_url(bucket_name, "signed_uploadtest", None, None, option)
            .await
            .unwrap();
        println!("downloading={url:?}");
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

    #[tokio::test]
    #[serial]
    async fn test_sign_with_overwrites() {
        let (client, _) = create_client().await;
        let bucket_name = "rust-object-test";
        let data = "aiueo";
        let content_type = "application/octet-stream";
        let overwritten_gai = client.default_google_access_id.as_ref().unwrap();
        let overwritten_sign_by = client.default_sign_by.as_ref().unwrap();

        // upload
        let option = SignedURLOptions {
            method: SignedURLMethod::PUT,
            content_type: Some(content_type.to_string()),
            ..SignedURLOptions::default()
        };
        let url = client
            .signed_url(
                bucket_name,
                "signed_uploadtest",
                Some(overwritten_gai.to_owned()),
                Some(overwritten_sign_by.to_owned()),
                option,
            )
            .await
            .unwrap();
        println!("uploading={url:?}");
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
            .signed_url(
                bucket_name,
                "signed_uploadtest",
                Some(overwritten_gai.to_owned()),
                Some(overwritten_sign_by.to_owned()),
                option,
            )
            .await
            .unwrap();
        println!("downloading={url:?}");
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
