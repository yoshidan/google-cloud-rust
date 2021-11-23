use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;

use tonic::transport::{Certificate, Channel, ClientTlsConfig, Endpoint};

use google_cloud_auth::token_source::TokenSource;
use google_cloud_auth::{create_token_source, Config};
use google_cloud_googleapis::spanner::v1::spanner_client::SpannerClient;

use crate::apiv1::spanner_client::Client;

const SPANNER: &str = "spanner.googleapis.com";
const AUDIENCE: &str = "https://spanner.googleapis.com/";
const SCOPES: [&str; 2] = [
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/spanner.data",
];

const TLS_CERTS: &[u8] = include_bytes!("roots.pem");

pub struct ConnectionManager {
    index: AtomicI64,
    token_source: Option<Arc<dyn TokenSource>>,
    conns: Vec<SpannerClient<Channel>>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    AuthInitialize(#[from] google_cloud_auth::error::Error),

    #[error(transparent)]
    TonicTransport(#[from] tonic::transport::Error),

    #[error("invalid spanner host {0}")]
    InvalidSpannerHOST(String),
}

impl ConnectionManager {
    pub async fn new(pool_size: usize, emulator_host: Option<String>) -> Result<Self, Error> {
        let (conns, token_source) = match emulator_host {
            None => {
                let tls_config = ClientTlsConfig::new()
                    .ca_certificate(Certificate::from_pem(TLS_CERTS))
                    .domain_name(SPANNER);
                let mut conns = Vec::with_capacity(pool_size);
                for _i_ in 0..pool_size {
                    let endpoint = Channel::from_static(AUDIENCE).tls_config(tls_config.clone())?;
                    let con = ConnectionManager::connect(endpoint).await?;
                    conns.push(con);
                }
                let token_source = create_token_source(Config {
                    audience: Some(AUDIENCE),
                    scopes: Some(&SCOPES),
                })
                .await?;
                (conns, Some(Arc::from(token_source)))
            }
            // use local spanner emulator
            Some(host) => {
                let mut conns = Vec::with_capacity(1);
                let endpoint = Channel::from_shared(format!("http://{}", host).into_bytes())
                    .map_err(|_| Error::InvalidSpannerHOST(host))?;
                let con = ConnectionManager::connect(endpoint).await?;
                conns.push(con);
                (conns, None)
            }
        };
        Ok(ConnectionManager {
            index: AtomicI64::new(0),
            token_source,
            conns,
        })
    }

    async fn connect(
        endpoint: Endpoint,
    ) -> Result<SpannerClient<Channel>, tonic::transport::Error> {
        let channel = endpoint.connect().await?;
        log::debug!("gRPC Connection Created");
        Ok(SpannerClient::new(channel))
    }

    pub fn num(&self) -> usize {
        self.conns.len()
    }

    pub fn conn(&self) -> Client {
        let current = self.index.fetch_add(1, Ordering::SeqCst) as usize;
        //clone() reuses http/s connection
        Client::new(
            self.conns[current % self.conns.len()].clone(),
            match self.token_source.as_ref() {
                Some(s) => Some(Arc::clone(s)),
                None => None,
            },
        )
    }
}
