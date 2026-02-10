use std::ops::Deref;
use std::sync::Arc;

#[cfg(feature = "auth")]
pub use google_cloud_auth;
use google_cloud_gax::conn::{ConnectionOptions, Environment, Error};

use token_source::{NoopTokenSourceProvider, TokenSourceProvider};

use crate::grpc::apiv1::conn_pool::{ConnectionManager, KMS, SCOPES};
use crate::grpc::apiv1::kms_client::Client as KmsGrpcClient;

#[derive(Debug)]
pub struct ClientConfig {
    pub endpoint: String,
    pub token_source_provider: Box<dyn TokenSourceProvider>,
    pub pool_size: Option<usize>,
    pub connection_option: ConnectionOptions,
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
        google_cloud_auth::project::Config::default().with_scopes(&SCOPES)
    }
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            endpoint: KMS.to_string(),
            token_source_provider: Box::new(NoopTokenSourceProvider {}),
            pool_size: Some(1),
            connection_option: ConnectionOptions::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Client {
    kms_client: KmsGrpcClient,
}

impl Client {
    pub async fn new(config: ClientConfig) -> Result<Self, Error> {
        let pool_size = config.pool_size.unwrap_or_default();
        let cm = ConnectionManager::new(
            pool_size,
            config.endpoint.as_str(),
            &Environment::GoogleCloud(config.token_source_provider),
            &config.connection_option,
        )
        .await?;
        Ok(Self {
            kms_client: KmsGrpcClient::new(Arc::new(cm)),
        })
    }
}

impl Deref for Client {
    type Target = KmsGrpcClient;

    fn deref(&self) -> &Self::Target {
        &self.kms_client
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use crate::grpc::kms::v1::{
        AsymmetricSignRequest, CreateKeyRingRequest, DecryptRequest, EncryptRequest, GenerateRandomBytesRequest,
        GetKeyRingRequest, GetPublicKeyRequest, ListKeyRingsRequest, MacSignRequest, MacVerifyRequest, ProtectionLevel,
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
        let (client, project) = new_client().await;
        let key_ring_id = "gcpkmskr1714619260".to_string();

        // create
        let create_request = CreateKeyRingRequest {
            parent: format!("projects/{project}/locations/us-west1"),
            key_ring_id: key_ring_id.clone(),
            key_ring: None,
        };
        /* KeyRing can not be deleted.
        let created_key_ring = client.create_key_ring(create_request.clone(), None).await.unwrap();
        assert_eq!(
            format!("{}/keyRings/{}", create_request.parent, create_request.key_ring_id),
            created_key_ring.name
        );
         */

        let key_ring = format!("{}/keyRings/{}", create_request.parent, create_request.key_ring_id);
        // get
        let get_request = GetKeyRingRequest { name: key_ring };
        let get_key_ring = client.get_key_ring(get_request.clone(), None).await.unwrap();
        assert_eq!(get_key_ring.name, get_request.name);

        // list
        let list_request = ListKeyRingsRequest {
            parent: create_request.parent.to_string(),
            page_size: 1,
            page_token: "".to_string(),
            filter: "".to_string(),
            order_by: "".to_string(),
        };
        let list_result = client.list_key_rings(list_request, None).await.unwrap();
        assert_eq!(1, list_result.key_rings.len());

        let list_request = ListKeyRingsRequest {
            parent: create_request.parent.to_string(),
            page_size: 1,
            page_token: list_result.next_page_token.to_string(),
            filter: "".to_string(),
            order_by: "".to_string(),
        };
        let list_result2 = client.list_key_rings(list_request, None).await.unwrap();
        assert_eq!(1, list_result2.key_rings.len());

        assert_ne!(list_result.key_rings[0].name, list_result2.key_rings[0].name);
    }

    #[tokio::test]
    #[serial]
    async fn test_generate_random_bytes() {
        let (client, project) = new_client().await;

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

    #[tokio::test]
    #[serial]
    async fn test_asymmetric_sign() {
        let (client, project) = new_client().await;

        let request = AsymmetricSignRequest {
            name: format!("projects/{project}/locations/asia-northeast1/keyRings/gcr_test/cryptoKeys/eth-sign/cryptoKeyVersions/1"),
            digest: None,
            digest_crc32c: None,
            data: vec![1,2,3,4,5],
            data_crc32c: None,
        };
        let signature = client.asymmetric_sign(request.clone(), None).await.unwrap();
        assert!(!signature.signature.is_empty());
    }
    #[tokio::test]
    #[serial]
    async fn test_get_pubkey() {
        let (client, project) = new_client().await;
        let request = GetPublicKeyRequest{
            name: format!("projects/{project}/locations/asia-northeast1/keyRings/gcr_test/cryptoKeys/eth-sign/cryptoKeyVersions/1"),
            public_key_format: 0,
        };
        let pubkey = client.get_public_key(request.clone(), None).await.unwrap();
        assert!(!pubkey.pem.is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_encrypt_decrypt() {
        let (client, project) = new_client().await;

        let key = format!("projects/{project}/locations/asia-northeast1/keyRings/gcr_test/cryptoKeys/gcr_test");
        let data = [1, 2, 3, 4, 5];
        let request = EncryptRequest {
            name: key.clone(),
            plaintext: data.to_vec(),
            additional_authenticated_data: vec![],
            plaintext_crc32c: None,
            additional_authenticated_data_crc32c: None,
        };
        let encrypted = client.encrypt(request, None).await.unwrap();

        let request = DecryptRequest {
            name: key,
            ciphertext: encrypted.ciphertext.clone(),
            additional_authenticated_data: vec![],
            ciphertext_crc32c: None,
            additional_authenticated_data_crc32c: None,
        };
        let raw = client.decrypt(request.clone(), None).await.unwrap();
        assert_eq!(data.to_vec(), raw.plaintext);
    }

    #[tokio::test]
    #[serial]
    async fn test_mac_sign_verify() {
        let (client, project) = new_client().await;

        let key = format!(
            "projects/{project}/locations/asia-northeast1/keyRings/gcr_test/cryptoKeys/mac-test/cryptoKeyVersions/1"
        );
        let data = [1, 2, 3, 4, 5];
        let request = MacSignRequest {
            name: key.clone(),
            data: data.to_vec(),
            data_crc32c: None,
        };
        let signature = client.mac_sign(request, None).await.unwrap();

        let request = MacVerifyRequest {
            name: key,
            data: data.to_vec(),
            data_crc32c: None,
            mac: signature.mac,
            mac_crc32c: signature.mac_crc32c,
        };
        let raw = client.mac_verify(request, None).await.unwrap();
        assert!(raw.success);
    }
}
