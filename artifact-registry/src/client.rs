use crate::grpc::apiv1::artifact_registry_client::Client as ArtifactRegistryGrpcClient;
use crate::sign::SignBy;
use google_cloud_gax::conn::{ConnectionManager, ConnectionOptions, Environment, Error};
use google_cloud_token::{NopeTokenSourceProvider, TokenSourceProvider};
use std::ops::{Deref, DerefMut};
use std::time::Duration;

#[derive(Debug)]
pub struct ClientConfig {
    pub http: Option<reqwest::Client>,
    pub artifact_registry_endpoint: String,
    pub service_account_endpoint: String,
    pub token_source_provider: Box<dyn TokenSourceProvider>,
    pub default_google_access_id: Option<String>,
    pub default_sign_by: Option<SignBy>,
    pub project_id: Option<String>,
}

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
        self.token_source_provider = Box::new(ts);
        self
    }

    fn auth_config() -> google_cloud_auth::project::Config<'static> {
        google_cloud_auth::project::Config {
            audience: None,
            scopes: Some(&SCOPES),
            sub: None,
        }
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            http: None,
            artifact_registry_endpoint: ARTIFACT_REGISTRY.to_string(),
            token_source_provider: Box::new(NopeTokenSourceProvider {}),
            service_account_endpoint: "https://iamcredentials.googleapis.com".to_string(),
            default_google_access_id: None,
            default_sign_by: None,
            project_id: None,
        }
    }
}

use crate::grpc::apiv1::{ARTIFACT_REGISTRY, AUDIENCE, SCOPES};
use crate::sign::SignBy::PrivateKey;
#[cfg(feature = "auth")]
pub use google_cloud_auth;
use google_cloud_googleapis::devtools::artifact_registry::v1::artifact_registry_client::ArtifactRegistryClient;
use google_cloud_longrunning::autogen::operations_client::OperationsClient;

#[derive(Clone)]
pub struct Client {
    artifact_registry_client: ArtifactRegistryGrpcClient,
}

impl Client {
    pub async fn new(config: ClientConfig) -> Result<Self, Error> {
        let conn_options = ConnectionOptions {
            timeout: Some(Duration::from_secs(30)),
            connect_timeout: Some(Duration::from_secs(30)),
        };
        let conn_pool = ConnectionManager::new(
            1,
            ARTIFACT_REGISTRY,
            AUDIENCE,
            &Environment::GoogleCloud(config.token_source_provider),
            &conn_options,
        )
        .await?;
        let conn = conn_pool.conn();
        let lro_client = OperationsClient::new(conn_pool.conn()).await.unwrap();

        Ok(Self {
            artifact_registry_client: ArtifactRegistryGrpcClient::new(ArtifactRegistryClient::new(conn), lro_client),
        })
    }
}

impl Deref for Client {
    type Target = ArtifactRegistryGrpcClient;

    fn deref(&self) -> &Self::Target {
        &self.artifact_registry_client
    }
}

impl DerefMut for Client {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.artifact_registry_client
    }
}
