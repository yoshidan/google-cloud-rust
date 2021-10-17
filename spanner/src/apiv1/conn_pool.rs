use crate::apiv1::spanner_client::Client;
use google_cloud_auth::token_source::token_source::TokenSource;
use google_cloud_auth::{create_token_source, Config};
use google_cloud_googleapis::spanner::v1::spanner_client::SpannerClient;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use tonic::{
    transport::{Certificate, Channel, ClientTlsConfig},
};

const SPANNER: &str = "spanner.googleapis.com";
const AUDIENCE: &str = "https://spanner.googleapis.com/";
const SCOPES: [&'static str; 2] = [
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/spanner.data",
];
pub const TLS_CERTS: &[u8] = include_bytes!("../../roots.pem");

pub struct ConnectionManager {
    index: AtomicI64,
    token_source: Arc<dyn TokenSource>,
    conns: Vec<SpannerClient<Channel>>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    AuthInitialize(#[from] google_cloud_auth::error::Error),

    #[error(transparent)]
    TonicTransport(#[from] tonic::transport::Error),
}

impl ConnectionManager {
    pub async fn new(pool_size: usize) -> Result<Self, Error> {
        let tls_config = ClientTlsConfig::new()
            .ca_certificate(Certificate::from_pem(TLS_CERTS))
            .domain_name(SPANNER);
        let mut conns = Vec::with_capacity(pool_size);
        for i_ in 0..pool_size {
            let con = ConnectionManager::connect(tls_config.clone()).await?;
            conns.push(con);
        }

        let token_source = create_token_source(Config {
            audience: Some(AUDIENCE),
            scopes: Some(&SCOPES),
        })
        .await?;

        return Ok(ConnectionManager {
            index: AtomicI64::new(0),
            token_source: Arc::from(token_source),
            conns,
        });
    }

    async fn connect(
        tls_config: ClientTlsConfig,
    ) -> Result<SpannerClient<Channel>, tonic::transport::Error> {
        let channel = Channel::from_static(AUDIENCE)
            .tls_config(tls_config)
            .unwrap()
            .connect()
            .await?;
        log::debug!("gRPC Connection Created");
        return Ok(SpannerClient::new(channel));
    }

    pub fn num(&self) -> usize {
        self.conns.len()
    }

    pub fn conn(&self) -> Client {
        let current = self.index.fetch_add(1, Ordering::SeqCst) as usize;
        //clone() reuses http/s connection
        Client::new(
            self.conns[current % self.conns.len()].clone(),
            Arc::clone(&self.token_source),
        )
    }
}
