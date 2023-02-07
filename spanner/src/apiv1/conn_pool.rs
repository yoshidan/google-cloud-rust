use google_cloud_gax::conn::{ConnectionManager as GRPCConnectionManager, Environment, Error};
use google_cloud_googleapis::spanner::v1::spanner_client::SpannerClient;

use crate::apiv1::spanner_client::Client;

pub struct ConnectionManager {
    inner: GRPCConnectionManager,
}

impl ConnectionManager {
    pub async fn new(pool_size: usize, environment: &Environment, domain: &str) -> Result<Self, Error> {
        Ok(ConnectionManager {
            inner: GRPCConnectionManager::new(pool_size, domain, environment).await?,
        })
    }

    pub fn num(&self) -> usize {
        self.inner.num()
    }

    pub fn conn(&self) -> Client {
        let conn = self.inner.conn();
        Client::new(SpannerClient::new(conn))
    }
}
