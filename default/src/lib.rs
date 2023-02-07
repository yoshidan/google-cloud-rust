use google_cloud_auth::bridge::DefaultTokenSourceProvider;
use google_cloud_auth::error::Error as AuthError;
use google_cloud_auth::project::Config;
use google_cloud_gax::conn::Environment;
use google_cloud_metadata::Error as MetadataError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Auth(#[from] AuthError),

    #[error(transparent)]
    Metadata(#[from] MetadataError),
}

pub trait WithAuthExt {
    async fn with_auth(&mut self) -> Result<(), Error>;
}

#[cfg(feature = "pubsub")]
impl WithAuthExt for google_cloud_pubsub::client::ClientConfig {
    async fn with_auth(&mut self) -> Result<(), Error> {
        match self.environment {
            Environment::GoogleCloud(_) => {
                let ts = DefaultTokenSourceProvider::new(Config {
                    audience: Some(google_cloud_pubsub::client::AUDIENCE),
                    scopes: Some(&google_cloud_pubsub::client::SCOPES),
                })
                .await?;
                self.project_id = ts.project_id.clone();
                self.environment = Environment::GoogleCloud(Box::new(ts))
            }
            _ => {}
        }
        Ok(())
    }
}

#[cfg(feature = "spanner")]
impl WithAuthExt for google_cloud_spanner::client::ClientConfig {
    async fn with_auth(&mut self) -> Result<(), Error> {
        match self.environment {
            Environment::GoogleCloud(_) => {
                let ts = DefaultTokenSourceProvider::new(Config {
                    audience: Some(google_cloud_spanner::client::AUDIENCE),
                    scopes: Some(&google_cloud_spanner::client::SCOPES),
                })
                .await?;
                self.environment = Environment::GoogleCloud(Box::new(ts))
            }
            _ => {}
        }
        Ok(())
    }
}

#[cfg(feature = "storage")]
impl WithAuthExt for google_cloud_storage::client::ClientConfig {
    async fn with_auth(&mut self) -> Result<(), Error> {
        let ts = DefaultTokenSourceProvider::new(Config {
            audience: None,
            scopes: Some(&google_cloud_storage::http::storage_client::SCOPES),
        })
        .await?;

        match &ts.source_credentials {
            //Credential file is used.
            Some(cred) => {
                if let Some(ref pk) = cred.private_key {
                    self.default_sign_by = Some(google_cloud_storage::sign::SignBy::PrivateKey(pk.into_bytes()));
                }
                self.default_google_access_id = cred.client_email
            }
            // On Google Cloud
            None => {
                self.default_sign_by = Some(google_cloud_storage::sign::SignBy::SignBytes);
                self.default_google_access_id = Some(google_cloud_metadata::email("default").await?);
            }
        }

        self.token_source_provider = Box::new(ts);
        Ok(())
    }
}
