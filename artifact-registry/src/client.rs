use crate::grpc::apiv1::artifact_registry_client::Client as ArtifactRegistryGrpcClient;
use google_cloud_gax::conn::{ConnectionManager, ConnectionOptions, Environment, Error};
use std::ops::{Deref, DerefMut};
use std::time::Duration;
use token_source::{NoopTokenSourceProvider, TokenSourceProvider};

use crate::grpc::apiv1::{ARTIFACT_REGISTRY, AUDIENCE, SCOPES};

use google_cloud_googleapis::devtools::artifact_registry::v1::artifact_registry_client::ArtifactRegistryClient;
use google_cloud_longrunning::autogen::operations_client::OperationsClient;

#[derive(Debug)]
pub struct ClientConfig {
    pub artifact_registry_endpoint: String,
    pub token_source_provider: Box<dyn TokenSourceProvider>,
    pub timeout: Option<Duration>,
    pub connect_timeout: Option<Duration>,
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
            artifact_registry_endpoint: ARTIFACT_REGISTRY.to_string(),
            token_source_provider: Box::new(NoopTokenSourceProvider {}),
            timeout: Some(Duration::from_secs(30)),
            connect_timeout: Some(Duration::from_secs(30)),
        }
    }
}

#[derive(Clone)]
pub struct Client {
    artifact_registry_client: ArtifactRegistryGrpcClient,
}

impl Client {
    pub async fn new(config: ClientConfig) -> Result<Self, Error> {
        let conn_options = ConnectionOptions {
            timeout: config.timeout,
            connect_timeout: config.connect_timeout,
        };
        let conn_pool = ConnectionManager::new(
            1,
            config.artifact_registry_endpoint,
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

#[cfg(test)]
mod tests {
    use crate::client::{Client, ClientConfig};

    use google_cloud_googleapis::devtools::artifact_registry::v1::repository::Format;
    use google_cloud_googleapis::devtools::artifact_registry::v1::{
        CreateRepositoryRequest, DeleteRepositoryRequest, GetRepositoryRequest, ListRepositoriesRequest, Repository,
        UpdateRepositoryRequest,
    };
    use prost_types::FieldMask;
    use serial_test::serial;
    use std::time::{SystemTime, UNIX_EPOCH};

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
    async fn test_crud_repository() {
        let (mut client, project) = new_client().await;
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let repository_id = format!("gcrar{now}");

        // create
        let create_request = CreateRepositoryRequest {
            parent: format!("projects/{project}/locations/us-central1"),
            repository_id,
            repository: Some(Repository {
                name: "".to_string(),
                format: Format::Docker as i32,
                description: "test repository".to_string(),
                labels: Default::default(),
                create_time: None,
                update_time: None,
                kms_key_name: "".to_string(),
                mode: 0,
                cleanup_policies: Default::default(),
                size_bytes: 0,
                satisfies_pzs: false,
                cleanup_policy_dry_run: false,
                format_config: None,
                mode_config: None,
                vulnerability_scanning_config: None,
                disallow_unspecified_mode: false,
                satisfies_pzi: false,
                registry_uri: "".to_string(),
            }),
        };
        let mut created_repository = client.create_repository(create_request.clone(), None).await.unwrap();
        let result = created_repository.wait(None).await.unwrap().unwrap();
        assert_eq!(
            format!("{}/repositories/{}", create_request.parent, create_request.repository_id),
            result.name
        );

        // get
        let get_request = GetRepositoryRequest {
            name: result.name.to_string(),
        };
        let get_repository = client.get_repository(get_request.clone(), None).await.unwrap();
        assert_eq!(get_repository.name, get_request.name);

        // update
        let update_request = UpdateRepositoryRequest {
            repository: Some(Repository {
                description: "update test".to_string(),
                ..get_repository.clone()
            }),
            update_mask: Some(FieldMask {
                paths: vec!["description".to_string()],
            }),
        };
        let update_repository = client.update_repository(update_request.clone(), None).await.unwrap();
        assert_eq!(update_repository.description, update_request.repository.unwrap().description);

        // list
        let list_request = ListRepositoriesRequest {
            parent: create_request.parent.to_string(),
            page_size: 0,
            page_token: "".to_string(),
            order_by: "".to_string(),
            filter: "".to_string(),
        };
        let list_result = client.list_repositories(list_request, None).await.unwrap();
        assert!(!list_result.repositories.is_empty());

        // delete
        let delete_request = DeleteRepositoryRequest {
            name: get_repository.name.to_string(),
        };
        client.delete_repository(delete_request, None).await.unwrap();
    }
}
