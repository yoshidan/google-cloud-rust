use std::sync::atomic::{AtomicI64, Ordering};

use tonic::transport::{Certificate, Channel, ClientTlsConfig, Endpoint};

const TLS_CERTS: &[u8] = include_bytes!("roots.pem");

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    TonicTransport(#[from] tonic::transport::Error),

    #[error("invalid spanner host {0}")]
    InvalidSpannerHOST(String),
}

pub struct ConnectionManager {
    index: AtomicI64,
    conns: Vec<Channel>,
}

impl ConnectionManager {
    pub async fn new(
        pool_size: usize,
        domain_name: &'static str,
        audience: &'static str,
        emulator_host: Option<String>,
    ) -> Result<Self, Error> {
        let conns = match emulator_host {
            None => {
                let tls_config = ClientTlsConfig::new()
                    .ca_certificate(Certificate::from_pem(TLS_CERTS))
                    .domain_name(domain_name);
                let mut conns = Vec::with_capacity(pool_size);
                for _i_ in 0..pool_size {
                    let endpoint = Channel::from_static(audience).tls_config(tls_config.clone())?;
                    let con = Self::connect(endpoint).await?;
                    conns.push(con);
                }
                conns
            }
            // use local spanner emulator
            Some(host) => {
                let mut conns = Vec::with_capacity(1);
                let endpoint = Channel::from_shared(format!("http://{}", host).into_bytes())
                    .map_err(|_| Error::InvalidSpannerHOST(host))?;
                let con = Self::connect(endpoint).await?;
                conns.push(con);
                conns
            }
        };
        Ok(Self {
            index: AtomicI64::new(0),
            conns,
        })
    }

    async fn connect(endpoint: Endpoint) -> Result<Channel, tonic::transport::Error> {
        let channel = endpoint.connect().await?;
        Ok(channel)
    }

    pub fn num(&self) -> usize {
        self.conns.len()
    }

    pub fn conn(&self) -> Channel {
        let current = self.index.fetch_add(1, Ordering::SeqCst) as usize;
        //clone() reuses http/2 connection
        self.conns[current % self.conns.len()].clone()
    }
}
