use google_cloud_auth::token_source::TokenSource;
use google_cloud_auth::{create_token_source, Config};
use google_cloud_googleapis::spanner::v1::spanner_client::SpannerClient;
use google_cloud_grpc::conn::{
    ConnectionManager as InternalConnectionManager, Error as InternalConnectionError,
};
use std::sync::Arc;

use crate::apiv1::spanner_client::Client;

pub const AUDIENCE: &str = "https://spanner.googleapis.com/";
pub const SPANNER: &str = "spanner.googleapis.com";
const SCOPES: [&str; 2] = [
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/spanner.data",
];

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    AuthInitialize(#[from] google_cloud_auth::error::Error),

    #[error(transparent)]
    InternalConnection(#[from] InternalConnectionError),
}

pub struct ConnectionManager {
    inner: InternalConnectionManager,
    token_source: Option<Arc<dyn TokenSource>>,
}

impl ConnectionManager {
    pub async fn new(pool_size: usize, emulator_host: Option<String>) -> Result<Self, Error> {
        Ok(ConnectionManager {
            inner: InternalConnectionManager::new(pool_size, SPANNER, AUDIENCE, emulator_host)
                .await?,
            token_source: Some(Arc::from(
                create_token_source(Config {
                    audience: Some(AUDIENCE),
                    scopes: Some(&SCOPES),
                })
                .await?,
            )),
        })
    }

    pub fn num(&self) -> usize {
        self.inner.num()
    }

    pub fn conn(&self) -> Client {
        //clone() reuses http/s connection
        Client::new(
            SpannerClient::new(self.inner.conn()),
            match self.token_source.as_ref() {
                Some(s) => Some(Arc::clone(s)),
                None => None,
            },
        )
    }
}
