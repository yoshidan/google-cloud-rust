use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use tonic::transport::{Certificate, Channel, ClientTlsConfig, Endpoint};

use crate::grpc::conn_pool::{InternalConnectionManager, AUDIENCE};
use google_cloud_auth::token_source::TokenSource;
use google_cloud_auth::{create_token_source, Config};

use crate::apiv1::conn_pool::Error;

const SCOPES: [&str; 2] = [
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/spanner.admin",
];

pub struct AdminConnectionManager {
    inner: InternalConnectionManager,
    token_source: Option<Arc<dyn TokenSource>>,
}

impl AdminConnectionManager {
    pub async fn new(pool_size: usize, emulator_host: Option<String>) -> Result<Self, Error> {
        Ok(AdminConnectionManager {
            inner: InternalConnectionManager::new(pool_size, emulator_host).await?,
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

    pub fn conn(&self) -> (Channel, Option<Arc<dyn TokenSource>>) {
        let token_source = match self.token_source.as_ref() {
            Some(s) => Some(Arc::clone(s)),
            None => None,
        };
        //clone() reuses http/s connection
        (self.inner.conn(), token_source)
    }
}
