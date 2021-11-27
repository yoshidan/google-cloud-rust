use google_cloud_auth::token_source::TokenSource;
use google_cloud_auth::{create_token_source, Config};
use std::sync::Arc;

use crate::inner::{InternalConnectionManager, Error as InternalConnectionError};
use tonic::transport::Channel;

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
    pub async fn new(pool_size: usize, domain_name: &'static str, audience: &'static str, scopes: Option<&'static [&'static str]>, emulator_host: Option<String>) -> Result<Self, Error> {
        Ok(Self {
            inner: InternalConnectionManager::new(pool_size, domain_name, audience, emulator_host)
                .await?,
            token_source: Some(Arc::from(
                create_token_source(Config {
                    audience: Some(audience),
                    scopes: scopes,
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
