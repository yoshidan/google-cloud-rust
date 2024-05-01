use std::ops::{Deref, DerefMut};
use std::time::Duration;

#[cfg(feature = "auth")]
pub use google_cloud_auth;
use google_cloud_gax::conn::{ConnectionManager, ConnectionOptions, Environment, Error};
use google_cloud_googleapis::cloud::kms::v1::key_management_service_client::KeyManagementServiceClient;
use google_cloud_token::{NopeTokenSourceProvider, TokenSourceProvider};

use crate::grpc::apiv1::kms_client::Client as KmsGrpcClient;
use crate::grpc::apiv1::{AUDIENCE, KMS, SCOPES};

#[derive(Debug)]
pub struct ClientConfig {
    pub kms_endpoint: String,
    pub token_source_provider: Box<dyn TokenSourceProvider>,
    pub timeout: Option<Duration>,
    pub connect_timeout: Option<Duration>,
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
            kms_endpoint: KMS.to_string(),
            token_source_provider: Box::new(NopeTokenSourceProvider {}),
            timeout: Some(Duration::from_secs(30)),
            connect_timeout: Some(Duration::from_secs(30)),
        }
    }
}

#[derive(Clone)]
pub struct Client {
    kms_client: KmsGrpcClient,
}

impl Client {
    pub async fn new(config: ClientConfig) -> Result<Self, Error> {
        let conn_options = ConnectionOptions {
            timeout: config.timeout,
            connect_timeout: config.connect_timeout,
        };
        let conn_pool = ConnectionManager::new(
            1,
            config.kms_endpoint,
            AUDIENCE,
            &Environment::GoogleCloud(config.token_source_provider),
            &conn_options,
        )
        .await?;
        let conn = conn_pool.conn();

        Ok(Self {
            kms_client: KmsGrpcClient::new(KeyManagementServiceClient::new(conn)),
        })
    }
}

impl Deref for Client {
    type Target = KmsGrpcClient;

    fn deref(&self) -> &Self::Target {
        &self.kms_client
    }
}

impl DerefMut for Client {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.kms_client
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use serial_test::serial;

    use google_cloud_googleapis::cloud::kms::v1::{
        CreateKeyRingRequest, GenerateRandomBytesRequest, GetKeyRingRequest, ListKeyRingsRequest, ProtectionLevel,
    };

    use crate::client::{Client, ClientConfig};

    async fn new_client() -> (Client, String) {
        let cred = google_cloud_auth::credentials::CredentialsFile::new().await.unwrap();
        let project = cred.project_id.clone().unwrap();
        let config = ClientConfig::default().with_credentials(cred).await.unwrap();
        (Client::new(config).await.unwrap(), project)
    }

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt().try_init();
    }

    #[tokio::test]
    #[serial]
    async fn test_key_ring() {
        let (mut client, project) = new_client().await;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let key_ring_id = format!("gcpkmskr{now:}");

        // create
        let create_request = CreateKeyRingRequest {
            parent: format!("projects/{project}/locations/us-west1"),
            key_ring_id: key_ring_id.clone(),
            key_ring: None,
        };
        let created_key_ring = client.create_key_ring(create_request.clone(), None).await.unwrap();
        assert_eq!(
            format!("{}/keyRings/{}", create_request.parent, create_request.key_ring_id),
            created_key_ring.name
        );

        // get
        let get_request = GetKeyRingRequest {
            name: created_key_ring.name.clone(),
        };
        let get_key_ring = client.get_key_ring(get_request.clone(), None).await.unwrap();
        assert_eq!(get_key_ring.name, get_request.name);

        // list
        let list_request = ListKeyRingsRequest {
            parent: create_request.parent.to_string(),
            page_size: 0,
            page_token: "".to_string(),
            filter: "".to_string(),
            order_by: "".to_string(),
        };
        let list_result = client.list_key_rings(list_request, None).await.unwrap();
        assert!(!list_result.key_rings.is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_generate_random_bytes() {
        let (mut client, project) = new_client().await;

        // create
        let create_request = GenerateRandomBytesRequest {
            location: format!("projects/{project}/locations/us-west1"),
            length_bytes: 128,
            protection_level: ProtectionLevel::Hsm.into(),
        };
        let random_bytes = client.generate_random_bytes(create_request.clone(), None).await;
        assert!(
            random_bytes.is_ok(),
            "Error when generating random bytes: {:?}",
            random_bytes.unwrap_err()
        );
        let random_bytes = random_bytes.unwrap();
        assert_eq!(
            random_bytes.data.len(),
            128,
            "Returned data length was {:?} when it should have been 128",
            random_bytes.data.len()
        );
        assert_ne!(
            random_bytes.data, [0; 128],
            "Data returned was all zeros: {:?}",
            random_bytes.data
        )
    }
}
