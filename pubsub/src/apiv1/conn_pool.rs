
use google_cloud_grpc::conn::{ConnectionManager as GRPCConnectionManager, Error};
use  google_cloud_grpc::conn::Channel;

pub const AUDIENCE: &str = "https://spanner.googleapis.com/";
pub const PUBSUB: &str = "pubsub.googleapis.com";
const SCOPES: [&str; 2] = [
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/pubsub.data",
];

pub struct ConnectionManager {
    inner: GRPCConnectionManager,
}

impl ConnectionManager {
    pub async fn new(pool_size: usize, emulator_host: Option<String>) -> Result<Self, Error> {
        Ok(ConnectionManager {
            inner: GRPCConnectionManager::new(
                pool_size,
                PUBSUB,
                AUDIENCE,
                Some(&SCOPES),
                emulator_host,
            )
            .await?,
        })
    }

    pub fn num(&self) -> usize {
        self.inner.num()
    }

    pub fn conn(&self) -> Channel {
       self.inner.conn()
    }
}
