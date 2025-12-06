use google_cloud_gax::conn::{Channel, Environment};
use google_cloud_gax::conn::{ConnectionManager as GRPCConnectionManager, ConnectionOptions, Error};

/// OAuth audience for token requests (global, works for all regional endpoints)
pub const AUDIENCE: &str = "https://pubsub.googleapis.com/";
/// Default Pub/Sub endpoint domain
pub const PUBSUB: &str = "pubsub.googleapis.com";
pub const SCOPES: [&str; 2] = [
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/pubsub",
];

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
        // Derive endpoint URL from domain to support regional endpoints
        let endpoint_url = format!("https://{}/", domain);
        Ok(ConnectionManager {
            inner: GRPCConnectionManager::new(pool_size, domain, &endpoint_url, environment, conn_options).await?,
        })
    }

    pub fn num(&self) -> usize {
        self.inner.num()
    }

    pub fn conn(&self) -> Channel {
        self.inner.conn()
    }
}
