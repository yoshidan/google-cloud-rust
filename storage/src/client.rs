use crate::bucket::BucketHandle;
use google_cloud_auth::credentials::CredentialsFile;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    AuthError(#[from] google_cloud_auth::error::Error),
    #[error(transparent)]
    MetadataError(#[from] google_cloud_metadata::Error),
}

pub struct Client {
    private_key: Option<String>,
    service_account_email: String,
}

impl Client {
    pub async fn new() -> Result<Self, Error> {
        let cred = CredentialsFile::new().await?;
        let service_account_email = match cred.client_email {
            Some(email) => email,
            None => {
                if google_cloud_metadata::on_gce().await {
                    google_cloud_metadata::email("default").await?
                } else {
                    "".to_string()
                }
            }
        };
        Ok(Client {
            private_key: cred.private_key,
            service_account_email,
        })
    }

    pub async fn bucket<'a, 'b>(&'b self, name: &'a str) -> BucketHandle<'a, 'b> {
        BucketHandle::new(
            name,
            match &self.private_key {
                Some(v) => v,
                None => "",
            },
            &self.service_account_email,
        )
    }
}

#[cfg(test)]
mod test {
    use crate::bucket::{BucketHandle, PathStyle, SignBy, SignedURLOptions, SigningScheme};
    use crate::client;
    use chrono::{DateTime, Utc};
    use google_cloud_auth::credentials::CredentialsFile;
    use serial_test::serial;
    use std::collections::HashMap;
    use std::time::Duration;
    use tracing::Level;

    #[ctor::ctor]
    fn init() {
        tracing_subscriber::fmt::try_init();
    }

    #[tokio::test]
    #[serial]
    async fn new() {
        let client = client::Client::new().await.unwrap();
        assert!(!client.service_account_email.is_empty());
        assert!(client.private_key.is_some());
    }
}
