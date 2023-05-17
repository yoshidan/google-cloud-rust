use google_cloud_gax::conn::{ConnectionManager as GRPCConnectionManager, Environment, Error};
use google_cloud_googleapis::cloud::bigquery::storage::v1::big_query_read_client::BigQueryReadClient;
use google_cloud_googleapis::cloud::bigquery::storage::v1::big_query_write_client::BigQueryWriteClient;
use crate::grpc::apiv1::bigquery_client::{ReadClient, WriteClient};

pub const AUDIENCE: &str = "https://bigquerystorage.googleapis.com/";
pub const DOMAIN: &str = "bigquerystorage.googleapis.com";

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

    pub fn conn(&self) -> ReadClient {
        let conn = self.inner.conn();
        ReadClient::new(BigQueryReadClient::new(conn))
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

    pub fn conn(&self) -> WriteClient {
        let conn = self.inner.conn();
        WriteClient::new(BigQueryWriteClient::new(conn))
    }
}
