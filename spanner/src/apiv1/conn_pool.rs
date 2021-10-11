use async_trait::async_trait;
use internal::spanner::v1 as internal;
use std::ops::DerefMut;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{
    metadata::MetadataValue,
    transport::{Certificate, Channel, ClientTlsConfig},
    IntoRequest, Request, Response, Status,
};

pub const TLS_CERTS: &[u8] = include_bytes!("../../roots.pem");

pub trait ConnPool {
    fn num(&self) -> usize;
    fn conn(&self) -> internal::spanner_client::SpannerClient<Channel>;
}

#[derive(Debug)]
pub struct ConnectionManager {
    index: AtomicI64,
    conns: Vec<internal::spanner_client::SpannerClient<Channel>>,
}

impl ConnectionManager {
    pub async fn new(pool_size: usize) -> Result<Self, tonic::transport::Error> {
        let tls_config = ClientTlsConfig::new()
            .ca_certificate(Certificate::from_pem(TLS_CERTS))
            .domain_name("spanner.googleapis.com");
        let mut conns = Vec::with_capacity(pool_size);
        for i in 0..pool_size {
            let con = ConnectionManager::connect(tls_config.clone()).await?;
            conns.push(con);
        }
        return Ok(ConnectionManager {
            index: AtomicI64::new(0),
            conns,
        });
    }

    async fn connect(
        tls_config: ClientTlsConfig,
    ) -> Result<internal::spanner_client::SpannerClient<Channel>, tonic::transport::Error> {
        let channel = Channel::from_static("https://spanner.googleapis.com")
            .tls_config(tls_config)
            .unwrap()
            .connect()
            .await?;
        log::debug!("gRPC Connection Created");
        return Ok(internal::spanner_client::SpannerClient::new(channel));
    }
}

impl ConnPool for ConnectionManager {
    fn num(&self) -> usize {
        self.conns.len()
    }

    fn conn(&self) -> internal::spanner_client::SpannerClient<Channel> {
        let current = self.index.fetch_add(1, Ordering::SeqCst) as usize;
        //clone() reuses http/s connection
        return self.conns[current % self.conns.len()].clone();
    }
}
