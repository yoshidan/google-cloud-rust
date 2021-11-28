use google_cloud_auth::token_source::TokenSource as InternalTokenSource;
use google_cloud_auth::{create_token_source, Config};
use std::sync::Arc;

use crate::inner::{Error as InternalConnectionError, InternalConnectionManager};
use tonic::transport::Channel;
use tonic::Status;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    AuthInitialize(#[from] google_cloud_auth::error::Error),

    #[error(transparent)]
    InternalConnection(#[from] InternalConnectionError),
}

pub struct ConnectionManager {
    inner: InternalConnectionManager,
    token_source: Option<Arc<dyn InternalTokenSource>>,
}

#[derive(Clone)]
pub struct TokenSource {
    inner: Option<Arc<dyn InternalTokenSource>>,
}

impl TokenSource {
    pub async fn token(&self) -> Result<Option<String>, Status> {
        match self.inner.as_ref() {
            Some(token_source) => token_source
                .token()
                .await
                .map_err(|e| {
                    Status::new(
                        tonic::Code::Unauthenticated,
                        format!("token error: {:?}", e),
                    )
                })
                .map(|v| Some(v.value())),
            None => Ok(None),
        }
    }
}

impl ConnectionManager {
    pub async fn new(
        pool_size: usize,
        domain_name: &'static str,
        audience: &'static str,
        scopes: Option<&'static [&'static str]>,
        emulator_host: Option<String>,
    ) -> Result<Self, Error> {
        let token_source = match emulator_host {
            Some(_) => None,
            None => {
                let ts = create_token_source(Config {
                    audience: Some(audience),
                    scopes,
                })
                .await?;
                Some(Arc::from(ts))
            }
        };
        let inner =
            InternalConnectionManager::new(pool_size, domain_name, audience, emulator_host).await?;
        Ok(Self {
            inner,
            token_source,
        })
    }

    pub fn num(&self) -> usize {
        self.inner.num()
    }

    pub fn conn(&self) -> (Channel, TokenSource) {
        let token_source = match self.token_source.as_ref() {
            Some(s) => Some(Arc::clone(s)),
            None => None,
        };
        //clone() reuses http/s connection
        (
            self.inner.conn(),
            TokenSource {
                inner: token_source,
            },
        )
    }
}
