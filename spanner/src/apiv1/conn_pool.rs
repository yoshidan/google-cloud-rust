use google_cloud_auth::token_source::TokenSource;
use google_cloud_auth::{create_token_source, Config};
use google_cloud_googleapis::spanner::v1::spanner_client::SpannerClient;
use google_cloud_grpc::conn::{
    ConnectionManager as GRPCConnectionManager, Error,
};
use std::sync::Arc;

use crate::apiv1::spanner_client::Client;

pub const AUDIENCE: &str = "https://spanner.googleapis.com/";
pub const SPANNER: &str = "spanner.googleapis.com";
const SCOPES: [&str; 2] = [
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/spanner.data",
];

pub struct ConnectionManager {
    inner: GRPCConnectionManager,
}

impl ConnectionManager {
    pub async fn new(pool_size: usize, emulator_host: Option<String>) -> Result<Self, Error> {
        Ok(ConnectionManager {
            inner: GRPCConnectionManager::new(pool_size, SPANNER, AUDIENCE, Some(&SCOPES), emulator_host).await?,
        })
    }

    pub fn num(&self) -> usize {
        self.inner.num()
    }

    pub fn conn(&self) -> Client {
        let (conn, ts) = self.inner.conn();
        Client::new(SpannerClient::new(conn), ts)
    }
}
