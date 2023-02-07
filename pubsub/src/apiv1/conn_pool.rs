
use google_cloud_gax::conn::{Channel, Environment};
use google_cloud_gax::conn::{ConnectionManager as GRPCConnectionManager, Error};

pub const AUDIENCE: &str = "https://pubsub.googleapis.com/";
pub const PUBSUB: &str = "pubsub.googleapis.com";
pub const SCOPES: [&str; 2] = [
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/pubsub.data",
];


#[derive(Debug)]
pub struct ConnectionManager {
    inner: GRPCConnectionManager,
}

impl ConnectionManager {
    pub async fn new(pool_size: usize, domain: &str, environment: &Environment) -> Result<Self, Error> {
        Ok(ConnectionManager {
            inner: GRPCConnectionManager::new(pool_size, domain, AUDIENCE, environment).await?,
        })
    }

    pub fn num(&self) -> usize {
        self.inner.num()
    }

    pub fn conn(&self) -> Channel {
        self.inner.conn()
    }
}
