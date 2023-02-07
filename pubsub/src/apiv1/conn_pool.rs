use google_cloud_gax::conn::{Channel, Environment};
use google_cloud_gax::conn::{ConnectionManager as GRPCConnectionManager, Error};

#[derive(Debug)]
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

    pub fn conn(&self) -> Channel {
        self.inner.conn()
    }
}
