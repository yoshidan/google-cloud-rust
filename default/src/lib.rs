//! # google-cloud-default
//!
//! Google Cloud Platform default configuration for google-cloud-rust.
//!
//! ## Quick Start
//!
//! * [pubsub](https://github.com/yoshidan/google-cloud-rust/tree/main/pubsub)
//! * [spanner](https://github.com/yoshidan/google-cloud-rust/tree/main/spanner)
//! * [storage](https://github.com/yoshidan/google-cloud-rust/tree/main/storage)
//! * [bigquery](https://github.com/yoshidan/google-cloud-rust/tree/main/bigquery)
//!
use async_trait::async_trait;

use google_cloud_auth::error::Error;

#[async_trait]
pub trait WithAuthExt {
    async fn with_auth(mut self) -> Result<Self, Error>
    where
        Self: Sized;

    async fn with_credentials(
        self,
        credentials: google_cloud_auth::credentials::CredentialsFile,
    ) -> Result<Self, Error>
    where
        Self: Sized;
}

#[cfg(feature = "pubsub")]
#[async_trait]
impl WithAuthExt for google_cloud_pubsub::client::ClientConfig {
    async fn with_auth(mut self) -> Result<Self, Error> {
        if let google_cloud_gax::conn::Environment::GoogleCloud(_) = self.environment {
            let ts = google_cloud_auth::token::DefaultTokenSourceProvider::new(google_cloud_auth::project::Config {
                audience: Some(google_cloud_pubsub::apiv1::conn_pool::AUDIENCE),
                scopes: Some(&google_cloud_pubsub::apiv1::conn_pool::SCOPES),
                sub: None,
            })
            .await?;
            self.project_id = ts.project_id.clone();
            self.environment = google_cloud_gax::conn::Environment::GoogleCloud(Box::new(ts))
        }
        Ok(self)
    }

    async fn with_credentials(
        mut self,
        credentials: google_cloud_auth::credentials::CredentialsFile,
    ) -> Result<Self, Error> {
        if let google_cloud_gax::conn::Environment::GoogleCloud(_) = self.environment {
            let ts = google_cloud_auth::token::DefaultTokenSourceProvider::new_with_credentials(
                google_cloud_auth::project::Config {
                    audience: Some(google_cloud_pubsub::apiv1::conn_pool::AUDIENCE),
                    scopes: Some(&google_cloud_pubsub::apiv1::conn_pool::SCOPES),
                    sub: None,
                },
                Box::new(credentials),
            )
            .await?;
            self.project_id = ts.project_id.clone();
            self.environment = google_cloud_gax::conn::Environment::GoogleCloud(Box::new(ts))
        }

        Ok(self)
    }
}

#[cfg(feature = "spanner")]
#[async_trait]
impl WithAuthExt for google_cloud_spanner::client::ClientConfig {
    async fn with_auth(mut self) -> Result<Self, Error> {
        if let google_cloud_gax::conn::Environment::GoogleCloud(_) = self.environment {
            let ts = google_cloud_auth::token::DefaultTokenSourceProvider::new(google_cloud_auth::project::Config {
                audience: Some(google_cloud_spanner::apiv1::conn_pool::AUDIENCE),
                scopes: Some(&google_cloud_spanner::apiv1::conn_pool::SCOPES),
                sub: None,
            })
            .await?;
            self.environment = google_cloud_gax::conn::Environment::GoogleCloud(Box::new(ts))
        }
        Ok(self)
    }

    async fn with_credentials(
        mut self,
        credentials: google_cloud_auth::credentials::CredentialsFile,
    ) -> Result<Self, Error> {
        if let google_cloud_gax::conn::Environment::GoogleCloud(_) = self.environment {
            let ts = google_cloud_auth::token::DefaultTokenSourceProvider::new_with_credentials(
                google_cloud_auth::project::Config {
                    audience: Some(google_cloud_spanner::apiv1::conn_pool::AUDIENCE),
                    scopes: Some(&google_cloud_spanner::apiv1::conn_pool::SCOPES),
                    sub: None,
                },
                Box::new(credentials),
            )
            .await?;
            self.environment = google_cloud_gax::conn::Environment::GoogleCloud(Box::new(ts))
        }
        Ok(self)
    }
}

#[cfg(feature = "storage")]
#[async_trait]
impl WithAuthExt for google_cloud_storage::client::ClientConfig {
    async fn with_auth(mut self) -> Result<Self, Error> {
        let ts = google_cloud_auth::token::DefaultTokenSourceProvider::new(google_cloud_auth::project::Config {
            audience: None,
            scopes: Some(&google_cloud_storage::http::storage_client::SCOPES),
            sub: None,
        })
        .await?;

        match &ts.source_credentials {
            // Credential file is used.
            Some(cred) => {
                self.project_id = cred.project_id.clone();
                if let Some(pk) = &cred.private_key {
                    self.default_sign_by =
                        Some(google_cloud_storage::sign::SignBy::PrivateKey(pk.clone().into_bytes()));
                }
                self.default_google_access_id = cred.client_email.clone();
            }
            // On Google Cloud
            None => {
                self.project_id = Some(google_cloud_metadata::project_id().await);
                self.default_sign_by = Some(google_cloud_storage::sign::SignBy::SignBytes);
                self.default_google_access_id = google_cloud_metadata::email("default").await.ok();
            }
        }

        self.token_source_provider = Box::new(ts);
        Ok(self)
    }

    async fn with_credentials(
        mut self,
        credentials: google_cloud_auth::credentials::CredentialsFile,
    ) -> Result<Self, Error> {
        let ts = google_cloud_auth::token::DefaultTokenSourceProvider::new_with_credentials(
            google_cloud_auth::project::Config {
                audience: None,
                scopes: Some(&google_cloud_storage::http::storage_client::SCOPES),
                sub: None,
            },
            Box::new(credentials),
        )
        .await?;

        match &ts.source_credentials {
            // Credential file is used.
            Some(cred) => {
                self.project_id = cred.project_id.clone();
                if let Some(pk) = &cred.private_key {
                    self.default_sign_by =
                        Some(google_cloud_storage::sign::SignBy::PrivateKey(pk.clone().into_bytes()));
                }
                self.default_google_access_id = cred.client_email.clone();
            }
            // On Google Cloud
            None => {
                self.project_id = Some(google_cloud_metadata::project_id().await);
                self.default_sign_by = Some(google_cloud_storage::sign::SignBy::SignBytes);
                self.default_google_access_id = google_cloud_metadata::email("default").await.ok();
            }
        }

        self.token_source_provider = Box::new(ts);
        Ok(self)
    }
}

#[cfg(feature = "bigquery")]
pub mod bigquery {
    use async_trait::async_trait;
    use google_cloud_auth::credentials::CredentialsFile;
    use google_cloud_auth::error::Error;
    use google_cloud_auth::project::Config;
    use google_cloud_auth::token::DefaultTokenSourceProvider;
    use google_cloud_bigquery::client::ClientConfig;

    #[async_trait]
    pub trait CreateAuthExt {
        async fn new_with_auth() -> Result<(Self, Option<String>), Error>
        where
            Self: Sized;

        async fn new_with_credentials(credentials: CredentialsFile) -> Result<(Self, Option<String>), Error>
        where
            Self: Sized;
    }

    #[async_trait]
    impl CreateAuthExt for ClientConfig {
        async fn new_with_auth() -> Result<(Self, Option<String>), Error> {
            let ts_http = DefaultTokenSourceProvider::new(bigquery_http_auth_config()).await?;
            let ts_grpc = DefaultTokenSourceProvider::new(bigquery_grpc_auth_config()).await?;
            let project_id = ts_grpc.project_id.clone();
            let config = Self::new(Box::new(ts_http), Box::new(ts_grpc));
            Ok((config, project_id))
        }
        async fn new_with_credentials(credentials: CredentialsFile) -> Result<(Self, Option<String>), Error>
        where
            Self: Sized,
        {
            let ts_http = DefaultTokenSourceProvider::new_with_credentials(
                bigquery_http_auth_config(),
                Box::new(credentials.clone()),
            )
            .await?;
            let ts_grpc =
                DefaultTokenSourceProvider::new_with_credentials(bigquery_grpc_auth_config(), Box::new(credentials))
                    .await?;
            let project_id = ts_grpc.project_id.clone();
            let config = Self::new(Box::new(ts_http), Box::new(ts_grpc));
            Ok((config, project_id))
        }
    }

    #[cfg(feature = "bigquery")]
    fn bigquery_http_auth_config() -> Config<'static> {
        Config {
            audience: None,
            scopes: Some(&google_cloud_bigquery::http::bigquery_client::SCOPES),
            sub: None,
        }
    }

    #[cfg(feature = "bigquery")]
    fn bigquery_grpc_auth_config() -> Config<'static> {
        Config {
            audience: Some(google_cloud_bigquery::grpc::apiv1::conn_pool::AUDIENCE),
            scopes: Some(&google_cloud_bigquery::grpc::apiv1::conn_pool::SCOPES),
            sub: None,
        }
    }
}

#[cfg(test)]
mod test {
    use google_cloud_gax::conn::Environment;

    use crate::WithAuthExt;

    #[tokio::test]
    async fn test_spanner() {
        let config = google_cloud_spanner::client::ClientConfig::default()
            .with_auth()
            .await
            .unwrap();
        if let Environment::Emulator(_) = config.environment {
            unreachable!()
        }
    }

    #[tokio::test]
    async fn test_pubsub() {
        let config = google_cloud_pubsub::client::ClientConfig::default()
            .with_auth()
            .await
            .unwrap();
        if let Environment::Emulator(_) = config.environment {
            unreachable!()
        }
    }

    #[tokio::test]
    async fn test_storage() {
        let config = google_cloud_storage::client::ClientConfig::default()
            .with_auth()
            .await
            .unwrap();
        assert!(config.default_google_access_id.is_some());
        assert!(config.default_sign_by.is_some());
    }

    #[tokio::test]
    async fn test_bigquery() {
        use crate::bigquery::CreateAuthExt;
        let (_config, project_id) = google_cloud_bigquery::client::ClientConfig::new_with_auth()
            .await
            .unwrap();
        assert!(project_id.is_some())
    }
}
