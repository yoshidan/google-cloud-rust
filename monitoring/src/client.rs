use google_cloud_gax::conn::{ConnectionOptions, Environment};
use google_cloud_gax::grpc::Status;
use google_cloud_gax::retry::RetrySetting;
use google_cloud_googleapis::monitoring::v3::{CreateTimeSeriesRequest, TimeSeries};
use google_cloud_token::NopeTokenSourceProvider;

use crate::apiv1::conn_pool::{ConnectionManager, MONITORING};
use crate::apiv1::metric_service_client::MetricServiceClient;

#[derive(Debug)]
pub struct ClientConfig {
    /// gRPC channel pool size
    pub pool_size: Option<usize>,
    /// Monitoring project_id
    pub project_id: Option<String>,
    /// Runtime project info
    pub environment: Environment,
    /// Overriding service endpoint
    pub endpoint: String,
    /// gRPC connection option
    pub connection_option: ConnectionOptions,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            pool_size: Some(4),
            environment: Environment::GoogleCloud(Box::new(NopeTokenSourceProvider {})),
            project_id: None,
            endpoint: MONITORING.to_string(),
            connection_option: ConnectionOptions::default(),
        }
    }
}

#[cfg(feature = "auth")]
pub use google_cloud_auth;

#[cfg(feature = "auth")]
impl ClientConfig {
    pub async fn with_auth(mut self) -> Result<Self, google_cloud_auth::error::Error> {
        if let Environment::GoogleCloud(_) = self.environment {
            let ts = google_cloud_auth::token::DefaultTokenSourceProvider::new(Self::auth_config()).await?;
            self.project_id = self.project_id.or(ts.project_id.clone());
            self.environment = Environment::GoogleCloud(Box::new(ts))
        }
        Ok(self)
    }

    pub async fn with_credentials(
        mut self,
        credentials: google_cloud_auth::credentials::CredentialsFile,
    ) -> Result<Self, google_cloud_auth::error::Error> {
        if let Environment::GoogleCloud(_) = self.environment {
            let ts = google_cloud_auth::token::DefaultTokenSourceProvider::new_with_credentials(
                Self::auth_config(),
                Box::new(credentials),
            )
            .await?;
            self.project_id = self.project_id.or(ts.project_id.clone());
            self.environment = Environment::GoogleCloud(Box::new(ts))
        }
        Ok(self)
    }

    fn auth_config() -> google_cloud_auth::project::Config<'static> {
        google_cloud_auth::project::Config::default()
            .with_audience(crate::apiv1::conn_pool::AUDIENCE)
            .with_scopes(&crate::apiv1::conn_pool::SCOPES)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    GAX(#[from] google_cloud_gax::conn::Error),
    #[error("Project ID was not found")]
    ProjectIdNotFound,
}

/// Client is a Google Cloud Monitoring client scoped to a single project.
///
/// Clients should be reused rather than being created as needed.
/// A Client may be shared by multiple tasks.
#[derive(Clone, Debug)]
pub struct Client {
    project_id: String,
    msc: MetricServiceClient,
}

impl Client {
    /// new creates a monitoring client. See [`ClientConfig`] for more information.
    pub async fn new(config: ClientConfig) -> Result<Self, Error> {
        let pool_size = config.pool_size.unwrap_or_default();

        let msc = MetricServiceClient::new(
            ConnectionManager::new(
                pool_size,
                config.endpoint.as_str(),
                &config.environment,
                &config.connection_option,
            )
            .await?,
        );
        Ok(Self {
            project_id: config.project_id.ok_or(Error::ProjectIdNotFound)?,
            msc,
        })
    }

    /// create_time_series creates or adds data to one or more time series.
    /// If any time series could not be written, a corresponding failure message is
    /// included in the error response.
    pub async fn create_time_series(
        &self,
        time_series: Vec<TimeSeries>,
        retry: Option<RetrySetting>,
    ) -> Result<(), Status> {
        let req = CreateTimeSeriesRequest {
            name: self.fully_qualified_project_name(),
            time_series,
        };
        self.msc.create_time_series(req, retry).await.map(|_| ())
    }

    fn fully_qualified_project_name(&self) -> String {
        format!("projects/{}", self.project_id)
    }
}
