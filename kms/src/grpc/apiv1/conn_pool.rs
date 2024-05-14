use google_cloud_gax::conn::{Channel, Environment};
use google_cloud_gax::conn::{ConnectionManager as GRPCConnectionManager, ConnectionOptions, Error};
use google_cloud_googleapis::cloud::kms::v1::key_management_service_client::KeyManagementServiceClient;

pub const AUDIENCE: &str = "https://cloudkms.googleapis.com/";
pub const KMS: &str = "cloudkms.googleapis.com";
pub const SCOPES: [&str; 1] = ["https://www.googleapis.com/auth/cloud-platform"];

#[derive(Debug)]
pub struct ConnectionManager {
    inner: GRPCConnectionManager,
}

impl ConnectionManager {
    pub async fn new(
        pool_size: usize,
        domain: &str,
        environment: &Environment,
        conn_options: &ConnectionOptions,
    ) -> Result<Self, Error> {
        Ok(ConnectionManager {
            inner: GRPCConnectionManager::new(pool_size, domain, AUDIENCE, environment, conn_options).await?,
        })
    }

    pub fn num(&self) -> usize {
        self.inner.num()
    }

    pub fn conn(&self) -> KeyManagementServiceClient<Channel> {
        KeyManagementServiceClient::new(self.inner.conn()).max_decoding_message_size(i32::MAX as usize)
    }
}
