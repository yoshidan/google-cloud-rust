use google_cloud_gax::conn::{
    Channel, ConnectionManager as GRPCConnectionManager, ConnectionOptions, Environment, Error,
};
use google_cloud_googleapis::cloud::bigquery::storage::v1::big_query_write_client::BigQueryWriteClient;
use crate::grpc::apiv1::bigquery_client::StreamingWriteClient;

pub const AUDIENCE: &str = "https://bigquerystorage.googleapis.com/";
pub const DOMAIN: &str = "bigquerystorage.googleapis.com";
pub const SCOPES: [&str; 3] = [
    "https://www.googleapis.com/auth/bigquery",
    "https://www.googleapis.com/auth/bigquery.insertdata",
    "https://www.googleapis.com/auth/cloud-platform",
];

#[derive(Debug)]
pub struct ConnectionManager {
    inner: GRPCConnectionManager,
}

impl ConnectionManager {
    pub async fn new(
        pool_size: usize,
        environment: &Environment,
        conn_options: &ConnectionOptions,
    ) -> Result<Self, Error> {
        Ok(ConnectionManager {
            inner: GRPCConnectionManager::new(pool_size, DOMAIN, AUDIENCE, environment, conn_options).await?,
        })
    }

    pub fn num(&self) -> usize {
        self.inner.num()
    }

    pub fn conn(&self) -> Channel {
        self.inner.conn()
    }

    pub fn writer(&self) -> StreamingWriteClient {
        StreamingWriteClient::new(BigQueryWriteClient::new(self.conn()))
    }
}
