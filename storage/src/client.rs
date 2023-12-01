use std::ops::Deref;

use ring::{rand, signature};

use google_cloud_token::{NopeTokenSourceProvider, TokenSourceProvider};

use crate::http::service_account_client::ServiceAccountClient;
use crate::http::storage_client::StorageClient;
use crate::sign::SignBy::PrivateKey;
use crate::sign::{create_signed_buffer, RsaKeyPair, SignBy, SignedURLError, SignedURLOptions};

#[derive(Debug)]
pub struct ClientConfig {
    pub http: Option<reqwest::Client>,
    pub storage_endpoint: String,
    pub service_account_endpoint: String,
    pub token_source_provider: Option<Box<dyn TokenSourceProvider>>,
    pub default_google_access_id: Option<String>,
    pub default_sign_by: Option<SignBy>,
    pub project_id: Option<String>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            http: None,
            storage_endpoint: "https://storage.googleapis.com".to_string(),
            token_source_provider: Some(Box::new(NopeTokenSourceProvider {})),
            service_account_endpoint: "https://iamcredentials.googleapis.com".to_string(),
            default_google_access_id: None,
            default_sign_by: None,
            project_id: None,
        }
    }
}

impl ClientConfig {
    pub fn anonymous(mut self) -> Self {
        self.token_source_provider = None;
        self
    }
}

#[cfg(feature = "auth")]
pub use google_cloud_auth;

#[cfg(feature = "auth")]
impl ClientConfig {
    pub async fn with_auth(self) -> Result<Self, google_cloud_auth::error::Error> {
        let ts = google_cloud_auth::token::DefaultTokenSourceProvider::new(Self::auth_config()).await?;
        Ok(self.with_token_source(ts).await)
    }

    pub async fn with_credentials(
        self,
        credentials: google_cloud_auth::credentials::CredentialsFile,
    ) -> Result<Self, google_cloud_auth::error::Error> {
        let ts = google_cloud_auth::token::DefaultTokenSourceProvider::new_with_credentials(
            Self::auth_config(),
            Box::new(credentials),
        )
        .await?;
        Ok(self.with_token_source(ts).await)
    }

    async fn with_token_source(mut self, ts: google_cloud_auth::token::DefaultTokenSourceProvider) -> Self {
        match &ts.source_credentials {
            // Credential file is used.
            Some(cred) => {
                self.project_id = cred.project_id.clone();
                if let Some(pk) = &cred.private_key {
                    self.default_sign_by = Some(PrivateKey(pk.clone().into_bytes()));
                }
                self.default_google_access_id = cred.client_email.clone();
            }
            // On Google Cloud
            None => {
                self.project_id = Some(google_cloud_metadata::project_id().await);
                self.default_sign_by = Some(SignBy::SignBytes);
                self.default_google_access_id = google_cloud_metadata::email("default").await.ok();
            }
        }
        self.token_source_provider = Some(Box::new(ts));
        self
    }

    fn auth_config() -> google_cloud_auth::project::Config<'static> {
        google_cloud_auth::project::Config {
            audience: None,
            scopes: Some(&crate::http::storage_client::SCOPES),
            sub: None,
        }
    }
}

#[derive(Clone)]
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
        let ts = match config.token_source_provider {
            Some(tsp) => Some(tsp.token_source()),
            None => {
                tracing::trace!("Use anonymous access due to lack of token");
                None
            }
        };
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
                    .ok_or(SignedURLError::InvalidOption("No default sign_by is found"))?;

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
                let key_pair = &RsaKeyPair::try_from(private_key)?;
                let mut signed = vec![0; key_pair.public().modulus_len()];
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

    use serial_test::serial;

    use crate::client::{Client, ClientConfig};
    use crate::http::buckets::get::GetBucketRequest;

    use crate::http::storage_client::test::bucket_name;
    use crate::sign::{SignedURLMethod, SignedURLOptions};

    async fn create_client() -> (Client, String) {
        let config = ClientConfig::default().with_auth().await.unwrap();
        let project_id = config.project_id.clone();
        (Client::new(config), project_id.unwrap())
    }

    #[tokio::test]
    #[serial]
    async fn test_sign() {
        let (client, project) = create_client().await;
        let bucket_name = bucket_name(&project, "object");
        let data = "aiueo";
        let content_type = "application/octet-stream";

        // upload
        let option = SignedURLOptions {
            method: SignedURLMethod::PUT,
            content_type: Some(content_type.to_string()),
            ..SignedURLOptions::default()
        };
        let url = client
            .signed_url(&bucket_name, "signed_uploadtest", None, None, option)
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
            .signed_url(&bucket_name, "signed_uploadtest", None, None, option)
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
        let (client, project) = create_client().await;
        let bucket_name = bucket_name(&project, "object");
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
                &bucket_name,
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
                &bucket_name,
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

    #[tokio::test]
    #[serial]
    async fn test_anonymous() {
        let project = ClientConfig::default().with_auth().await.unwrap().project_id.unwrap();
        let bucket = bucket_name(&project, "anonymous");

        let config = ClientConfig::default().anonymous();
        let client = Client::new(config);
        let result = client
            .get_bucket(&GetBucketRequest {
                bucket: bucket.clone(),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(result.name, bucket);
    }
}
