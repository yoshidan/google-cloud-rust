use crate::bucket::BucketHandle;
use google_cloud_auth::credentials::CredentialsFile;
use crate::apiv2::storage_client::StorageClient;
use crate::apiv2::conn_pool::ConnectionManager;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    AuthError(#[from] google_cloud_auth::error::Error),
    #[error(transparent)]
    MetadataError(#[from] google_cloud_metadata::Error),
    #[error(transparent)]
    GRPCError(#[from] google_cloud_gax::conn::Error),
}

pub struct Client {
    private_key: Option<String>,
    service_account_email: String,
    project_id: String,
    storage_client: StorageClient
}

impl Client {
    pub async fn new() -> Result<Self, Error> {
        let cred = CredentialsFile::new().await?;
        Ok(Client {
            private_key: cred.private_key,
            service_account_email: match cred.client_email {
                Some(email) => email,
                None => if google_cloud_metadata::on_gce().await {
                    google_cloud_metadata::email("default").await?
                } else {
                    "".to_string()
                }
            },
            project_id: match cred.project_id {
                Some(project_id) => project_id.to_string(),
                None => if google_cloud_metadata::on_gce().await {
                    google_cloud_metadata::project_id().await.to_string()
                } else {
                    "".to_string()
                }
            },
            storage_client:  StorageClient::new(ConnectionManager::new(4, None).await?)
        })
    }

    pub async fn bucket(&self, name: &str) -> BucketHandle<'_> {
        BucketHandle::new(
            format!("projects/{}/buckets/{}", self.project_id, name),
            match &self.private_key {
                Some(v) => v,
                None => "",
            },
            &self.service_account_email,
            self.storage_client.clone()
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
    use google_cloud_gax::cancel::CancellationToken;
    use google_cloud_gax::retry::RetrySetting;

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

    #[tokio::test]
    #[serial]
    async fn delete() {
        let client = client::Client::new().await.unwrap();
        let bucket = client.bucket("atl-dev1-test").await;
        let result = bucket.delete(Some(CancellationToken::default()), Some(RetrySetting::default())).await;
        assert!(result.is_ok())
    }
}
