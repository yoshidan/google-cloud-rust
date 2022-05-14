use crate::bucket::BucketHandle;
use crate::http;
use crate::http::old_entity::{Bucket, ListBucketsRequest};
use crate::http::storage_client::StorageClient;
use google_cloud_auth::credentials::CredentialsFile;
use google_cloud_auth::{create_token_source_from_credentials, Config};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

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

    pub fn bucket(&self, name: &str) -> BucketHandle<'_> {
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

    pub async fn buckets(
        &self,
        prefix: Option<String>,
        cancel: Option<CancellationToken>,
    ) -> Result<Vec<Bucket>, http::storage_client::Error> {
        let mut result: Vec<Bucket> = vec![];
        let mut page_token = None;
        loop {
            let req = ListBucketsRequest {
                max_results: None,
                prefix: prefix.clone(),
                page_token,
                projection: None,
            };
            let response = self
                .storage_client
                .list_buckets(self.project_id.as_str(), &req, cancel.clone())
                .await?;
            result.extend(response.items);
            if response.next_page_token.is_none() {
                break;
            } else {
                page_token = response.next_page_token;
            }
        }
        return Ok(result);
    }
}

#[cfg(test)]
mod test {
    use serial_test::serial;
    use crate::client::Client;

    #[ctor::ctor]
    fn init() {
        tracing_subscriber::fmt::try_init();
    }

    #[tokio::test]
    #[serial]
    async fn buckets() {
        let prefix = Some("rust-bucket-test".to_string());
        let client = Client::new().await.unwrap();
        let result = client.buckets(prefix, None).await.unwrap();
        assert_eq!(result.len(), 1);
        let result2 = client.buckets(None, None).await.unwrap();
        assert!(result2.len() > 1);
    }

}
