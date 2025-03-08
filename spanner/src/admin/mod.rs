use std::env::var;
use std::time::Duration;

use google_cloud_gax::conn::Environment;
use google_cloud_gax::grpc::Code;
use google_cloud_gax::retry::RetrySetting;
use token_source::NoopTokenSourceProvider;

pub mod client;
pub mod database;
pub mod instance;

pub const SCOPES: [&str; 2] = [
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/spanner.admin",
];

pub struct AdminClientConfig {
    /// Runtime project
    pub environment: Environment,
}

impl Default for AdminClientConfig {
    fn default() -> Self {
        AdminClientConfig {
            environment: match var("SPANNER_EMULATOR_HOST").ok() {
                Some(v) => Environment::Emulator(v),
                None => Environment::GoogleCloud(Box::new(NoopTokenSourceProvider {})),
            },
        }
    }
}

#[cfg(feature = "auth")]
pub use google_cloud_auth;

#[cfg(feature = "auth")]
impl AdminClientConfig {
    pub async fn with_auth(mut self) -> Result<Self, google_cloud_auth::error::Error> {
        if let Environment::GoogleCloud(_) = self.environment {
            let ts = google_cloud_auth::token::DefaultTokenSourceProvider::new(Self::auth_config()).await?;
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

pub fn default_retry_setting() -> RetrySetting {
    RetrySetting {
        from_millis: 50,
        max_delay: Some(Duration::from_secs(10)),
        factor: 1u64,
        take: 20,
        codes: vec![Code::Unavailable, Code::Unknown, Code::DeadlineExceeded],
    }
}
