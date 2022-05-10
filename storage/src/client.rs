use crate::apiv1::storage_client::StorageClient;
use crate::bucket::BucketHandle;
use google_cloud_auth::credentials::CredentialsFile;
use google_cloud_auth::{create_token_source_from_credentials, Config};
use std::sync::Arc;

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
    project_id: String,
    storage_client: StorageClient,
}

impl Client {
    pub async fn new() -> Result<Self, Error> {
        const SCOPES: [&str; 2] = [
            "https://www.googleapis.com/auth/cloud-platform",
            "https://www.googleapis.com/auth/devstorage.full_control",
        ];
        let cred = CredentialsFile::new().await?;
        let ts = create_token_source_from_credentials(
            &cred,
            Config {
                audience: None,
                scopes: Some(&SCOPES),
            },
        )
        .await?;
        Ok(Client {
            private_key: cred.private_key,
            service_account_email: match cred.client_email {
                Some(email) => email,
                None => {
                    if google_cloud_metadata::on_gce().await {
                        google_cloud_metadata::email("default").await?
                    } else {
                        "".to_string()
                    }
                }
            },
            project_id: match cred.project_id {
                Some(project_id) => project_id.to_string(),
                None => {
                    if google_cloud_metadata::on_gce().await {
                        google_cloud_metadata::project_id().await.to_string()
                    } else {
                        "".to_string()
                    }
                }
            },
            storage_client: StorageClient::new(Arc::from(ts)),
        })
    }

    pub async fn bucket(&self, name: &str) -> BucketHandle<'_> {
        BucketHandle::new(
            //format!("projects/{}/buckets/{}", self.project_id, name), <- for v2 gRPC API
            name.to_string(),
            match &self.private_key {
                Some(v) => v,
                None => "",
            },
            &self.service_account_email,
            &self.project_id,
            self.storage_client.clone(),
        )
    }
}

#[cfg(test)]
mod test {
    use crate::apiv1::partial::BucketCreationConfig;
    use crate::client;
    use chrono::{DateTime, Utc};
    use google_cloud_auth::credentials::CredentialsFile;
    use google_cloud_gax::cancel::CancellationToken;
    use google_cloud_gax::retry::RetrySetting;
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

    #[tokio::test]
    #[serial]
    async fn delete() {
        let client = client::Client::new().await.unwrap();
        let bucket = client.bucket("atl-dev1-test").await;
        let result = bucket.delete(Some(CancellationToken::default())).await;
        assert!(result.is_ok(), "{}", result.unwrap_err())
    }

    #[tokio::test]
    #[serial]
    async fn create() {
        let client = client::Client::new().await.unwrap();
        let bucket = client.bucket("atl-dev1-testx43").await;
        let result = bucket
            .create(&BucketCreationConfig::default(), Some(CancellationToken::default()))
            .await.unwrap();
        println!("{:?}", result);
        assert_eq!(result.name, "atl-dev1-testx43");
    }
}
