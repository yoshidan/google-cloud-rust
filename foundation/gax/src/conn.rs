use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use http::header::AUTHORIZATION;
use http::{HeaderValue, Request};
use tonic::body::Body;
use tonic::transport::{Channel as TonicChannel, ClientTlsConfig, Endpoint};
use tonic::{Code, Status};
use tower::filter::{AsyncFilter, AsyncFilterLayer, AsyncPredicate};
use tower::util::Either;
use tower::{BoxError, ServiceBuilder};

use token_source::{TokenSource, TokenSourceProvider};

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

impl AsyncPredicate<Request<Body>> for AsyncAuthInterceptor {
    type Future = Pin<Box<dyn Future<Output = Result<Self::Request, BoxError>> + Send>>;
    type Request = Request<Body>;

    fn check(&mut self, request: Request<Body>) -> Self::Future {
        let ts = self.token_source.clone();
        Box::pin(async move {
            let token = ts
                .token()
                .await
                .map_err(|e| Status::new(Code::Unauthenticated, format!("token error: {e:?}")))?;
            let token_header = HeaderValue::from_str(token.as_str())
                .map_err(|e| Status::new(Code::Unauthenticated, format!("token error: {e:?}")))?;
            let (mut parts, body) = request.into_parts();
            parts.headers.insert(AUTHORIZATION, token_header);
            Ok(Request::from_parts(parts, body))
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Auth(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("tonic error : {0}")]
    TonicTransport(#[from] tonic::transport::Error),

    #[error("invalid emulator host: {0}")]
    InvalidEmulatorHOST(String),
}

#[derive(Debug)]
pub enum Environment {
    Emulator(String),
    GoogleCloud(Box<dyn TokenSourceProvider>),
}

#[derive(Debug)]
struct AtomicRing<T>
where
    T: Clone + Debug,
{
    index: AtomicUsize,
    values: Vec<T>,
}

impl<T> AtomicRing<T>
where
    T: Clone + Debug,
{
    fn next(&self) -> T {
        let current = self.index.fetch_add(1, Ordering::SeqCst);
        //clone() reuses http/2 connection
        self.values[current % self.values.len()].clone()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ConnectionOptions {
    pub timeout: Option<Duration>,
    pub connect_timeout: Option<Duration>,
}

impl ConnectionOptions {
    fn apply(&self, mut endpoint: Endpoint) -> Endpoint {
        endpoint = match self.timeout {
            Some(t) => endpoint.timeout(t),
            None => endpoint,
        };
        endpoint = match self.connect_timeout {
            Some(t) => endpoint.connect_timeout(t),
            None => endpoint,
        };
        endpoint
    }
}

#[derive(Debug)]
pub struct ConnectionManager {
    inner: AtomicRing<Channel>,
}

impl<'a> ConnectionManager {
    pub async fn new(
        pool_size: usize,
        domain_name: impl Into<String>,
        audience: &'static str,
        environment: &Environment,
        conn_options: &'a ConnectionOptions,
    ) -> Result<Self, Error> {
        let conns = match environment {
            Environment::GoogleCloud(ts_provider) => {
                Self::create_connections(pool_size, domain_name, audience, ts_provider.as_ref(), conn_options).await?
            }
            Environment::Emulator(host) => Self::create_emulator_connections(host, conn_options).await?,
        };
        Ok(Self {
            inner: AtomicRing {
                index: AtomicUsize::new(0),
                values: conns,
            },
        })
    }

    async fn create_connections(
        pool_size: usize,
        domain_name: impl Into<String>,
        audience: &'static str,
        ts_provider: &dyn TokenSourceProvider,
        conn_options: &'a ConnectionOptions,
    ) -> Result<Vec<Channel>, Error> {
        let tls_config = ClientTlsConfig::new().with_webpki_roots().domain_name(domain_name);
        let mut conns = Vec::with_capacity(pool_size);

        let ts = ts_provider.token_source();

        for _i_ in 0..pool_size {
            let endpoint = TonicChannel::from_static(audience).tls_config(tls_config.clone())?;
            let endpoint = conn_options.apply(endpoint);

            let con = Self::connect(endpoint).await?;
            // use GCP token per call
            let auth_layer = Some(AsyncFilterLayer::new(AsyncAuthInterceptor::new(Arc::clone(&ts))));
            let auth_con = ServiceBuilder::new().option_layer(auth_layer).service(con);
            conns.push(auth_con);
        }
        Ok(conns)
    }

    async fn create_emulator_connections(
        host: &str,
        conn_options: &'a ConnectionOptions,
    ) -> Result<Vec<Channel>, Error> {
        let mut conns = Vec::with_capacity(1);
        let endpoint = TonicChannel::from_shared(format!("http://{host}").into_bytes())
            .map_err(|_| Error::InvalidEmulatorHOST(host.to_string()))?;
        let endpoint = conn_options.apply(endpoint);

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
        self.inner.values.len()
    }

    pub fn conn(&self) -> Channel {
        self.inner.next()
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use crate::conn::AtomicRing;

    #[test]
    fn test_atomic_ring() {
        let cm = AtomicRing::<&str> {
            index: AtomicUsize::new(usize::MAX - 1),
            values: vec!["a", "b", "c", "d"],
        };
        let mut values = HashSet::new();
        assert_eq!(usize::MAX - 1, cm.index.load(Ordering::SeqCst));
        assert!(values.insert(cm.next()));
        assert_eq!(usize::MAX, cm.index.load(Ordering::SeqCst));
        assert!(values.insert(cm.next()));
        assert_eq!(0, cm.index.load(Ordering::SeqCst));
        assert!(values.insert(cm.next()));
        assert_eq!(1, cm.index.load(Ordering::SeqCst));
        assert!(values.insert(cm.next()));
        assert_eq!(2, cm.index.load(Ordering::SeqCst));
        assert!(!values.insert(cm.next()));
        assert_eq!(3, cm.index.load(Ordering::SeqCst));
    }
}
