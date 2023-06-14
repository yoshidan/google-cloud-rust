use google_cloud_gax::conn::{ConnectionManager as GRPCConnectionManager, Environment, Error};
use google_cloud_googleapis::cloud::bigquery::storage::v1::big_query_read_client::BigQueryReadClient;
use google_cloud_googleapis::cloud::bigquery::storage::v1::big_query_write_client::BigQueryWriteClient;

use crate::grpc::apiv1::bigquery_client::{StreamingReadClient, StreamingWriteClient};

pub const AUDIENCE: &str = "https://bigquerystorage.googleapis.com/";
pub const DOMAIN: &str = "bigquerystorage.googleapis.com";
pub const SCOPES: [&str; 3] = [
    "https://www.googleapis.com/auth/bigquery",
    "https://www.googleapis.com/auth/bigquery.insertdata",
    "https://www.googleapis.com/auth/cloud-platform",
];

pub struct ReadConnectionManager {
    inner: GRPCConnectionManager,
}

impl ReadConnectionManager {
    pub async fn new(pool_size: usize, environment: &Environment, domain: &str) -> Result<Self, Error> {
        Ok(ReadConnectionManager {
            inner: GRPCConnectionManager::new(pool_size, domain, AUDIENCE, environment).await?,
        })
    }

    pub fn num(&self) -> usize {
        self.inner.num()
    }

    pub fn conn(&self) -> StreamingReadClient {
        let conn = self.inner.conn();
        StreamingReadClient::new(BigQueryReadClient::new(conn))
    }
}

pub struct WriteConnectionManager {
    inner: GRPCConnectionManager,
}

impl WriteConnectionManager {
    pub async fn new(pool_size: usize, environment: &Environment, domain: &str) -> Result<Self, Error> {
        Ok(WriteConnectionManager {
            inner: GRPCConnectionManager::new(pool_size, domain, AUDIENCE, environment).await?,
        })
    }

    pub fn num(&self) -> usize {
        self.inner.num()
    }

    pub fn conn(&self) -> StreamingWriteClient {
        let conn = self.inner.conn();
        StreamingWriteClient::new(BigQueryWriteClient::new(conn))
    }
}
