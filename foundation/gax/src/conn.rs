use std::sync::atomic::{AtomicI64, Ordering};

use google_cloud_auth::token_source::TokenSource;
use google_cloud_auth::{create_token_source_from_project, Config, Project};
use http::header::AUTHORIZATION;
use http::{HeaderValue, Request};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tonic::body::BoxBody;
use tonic::transport::{Certificate, Channel as TonicChannel, ClientTlsConfig, Endpoint};
use tonic::{Code, Status};
use tower::filter::{AsyncFilter, AsyncFilterLayer, AsyncPredicate};
use tower::util::Either;
use tower::{BoxError, ServiceBuilder};

const TLS_CERTS: &[u8] = include_bytes!("roots.pem");

pub type Channel = Either<AsyncFilter<TonicChannel, AsyncAuthInterceptor>, TonicChannel>;

#[derive(Clone, Debug)]
pub struct AsyncAuthInterceptor {
    token_source: Arc<dyn TokenSource>,
}

impl AsyncAuthInterceptor {
    fn new(token_source: Arc<dyn TokenSource>) -> Self {
        Self { token_source }
    }
}

impl AsyncPredicate<Request<BoxBody>> for AsyncAuthInterceptor {
    type Future = Pin<Box<dyn Future<Output = Result<Self::Request, BoxError>> + Send>>;
    type Request = Request<BoxBody>;

    fn check(&mut self, request: Request<BoxBody>) -> Self::Future {
        let ts = self.token_source.clone();
        Box::pin(async move {
            let token = ts
                .token()
                .await
                .map_err(|e| Status::new(Code::Unauthenticated, format!("token error: {:?}", e)))?;
            let token_header = HeaderValue::from_str(token.value().as_ref())
                .map_err(|e| Status::new(Code::Unauthenticated, format!("token error: {:?}", e)))?;
            let (mut parts, body) = request.into_parts();
            parts.headers.insert(AUTHORIZATION, token_header);
            Ok(Request::from_parts(parts, body))
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Auth(#[from] google_cloud_auth::error::Error),

    #[error("tonic error : {0}")]
    TonicTransport(#[from] tonic::transport::Error),

    #[error("invalid emulator host: {0}")]
    InvalidEmulatorHOST(String),
}

pub enum Environment {
    Emulator(String),
    GoogleCloud(Project),
}

#[derive(Debug)]
pub struct ConnectionManager {
    index: AtomicI64,
    conns: Vec<Channel>,
}

impl ConnectionManager {
    pub async fn new(
        pool_size: usize,
        domain_name: &'static str,
        audience: &'static str,
        scopes: Option<&'static [&'static str]>,
        environment: &Environment,
    ) -> Result<Self, Error> {
        let conns = match environment {
            Environment::GoogleCloud(project) => {
                Self::create_connections(pool_size, domain_name, audience, scopes, &project).await?
            }
            Environment::Emulator(host) => Self::create_emulator_connections(&host).await?,
        };
        Ok(Self {
            index: AtomicI64::new(0),
            conns,
        })
    }

    async fn create_connections(
        pool_size: usize,
        domain_name: &'static str,
        audience: &'static str,
        scopes: Option<&'static [&'static str]>,
        project: &Project,
    ) -> Result<Vec<Channel>, Error> {
        let tls_config = ClientTlsConfig::new()
            .ca_certificate(Certificate::from_pem(TLS_CERTS))
            .domain_name(domain_name);
        let mut conns = Vec::with_capacity(pool_size);

        let ts = create_token_source_from_project(
            project,
            Config {
                audience: Some(audience),
                scopes,
            },
        )
        .await
        .map(|e| Arc::from(e))?;

        for _i_ in 0..pool_size {
            let endpoint = TonicChannel::from_static(audience).tls_config(tls_config.clone())?;
            let con = Self::connect(endpoint).await?;
            // use GCP token per call
            let auth_layer = Some(AsyncFilterLayer::new(AsyncAuthInterceptor::new(Arc::clone(&ts))));
            let auth_con = ServiceBuilder::new().option_layer(auth_layer).service(con);
            conns.push(auth_con);
        }
        Ok(conns)
    }

    async fn create_emulator_connections(host: &str) -> Result<Vec<Channel>, Error> {
        let mut conns = Vec::with_capacity(1);
        let endpoint = TonicChannel::from_shared(format!("http://{}", host).into_bytes())
            .map_err(|_| Error::InvalidEmulatorHOST(host.to_string()))?;
        let con = Self::connect(endpoint).await?;
        conns.push(
            ServiceBuilder::new()
                .option_layer::<AsyncFilterLayer<AsyncAuthInterceptor>>(None)
                .service(con),
        );
        Ok(conns)
    }

    async fn connect(endpoint: Endpoint) -> Result<TonicChannel, tonic::transport::Error> {
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
