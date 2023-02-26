//! # google-cloud-default
//!
//! Google Cloud Platform default configuration for google-cloud-rust.
//!
//! ## Quick Start
//!
//! * [pubsub](https://github.com/yoshidan/google-cloud-rust/tree/main/pubsub)
//! * [spanner](https://github.com/yoshidan/google-cloud-rust/tree/main/spanner)
//! * [storage](https://github.com/yoshidan/google-cloud-rust/tree/main/storage)
//!
use async_trait::async_trait;

use google_cloud_auth::error::Error;

#[async_trait]
pub trait WithAuthExt {
    async fn with_auth(mut self) -> Result<Self, Error>
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
            })
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
            })
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
        })
        .await?;

        match &ts.source_credentials {
            //Credential file is used.
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
}
