use google_cloud_gax::conn::{ConnectionManager as GRPCConnectionManager, Error};
use google_cloud_gax::conn::Channel;

const AUDIENCE: &str = "https://storage.googleapis.com/";
const STORAGE: &str = "storage.googleapis.com";
const SCOPES: [&str; 5] = [
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/cloud-platform.read-only",
    "https://www.googleapis.com/auth/devstorage.full_control",
    "https://www.googleapis.com/auth/devstorage.read_only",
    "https://www.googleapis.com/auth/devstorage.read_write",
];

pub struct ConnectionManager {
    inner: GRPCConnectionManager,
}

impl ConnectionManager {
    pub async fn new(pool_size: usize, emulator_host: Option<String>) -> Result<Self, Error> {
        Ok(ConnectionManager {
            inner: GRPCConnectionManager::new(pool_size, STORAGE, AUDIENCE, Some(&SCOPES), emulator_host).await?,
        })
    }

    pub fn num(&self) -> usize {
        self.inner.num()
    }

    pub fn conn(&self) -> Channel {
        self.inner.conn()
    }
}
