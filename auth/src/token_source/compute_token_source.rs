use crate::error::Error;
use crate::token::{Token, TokenSource};
use crate::token_source::{default_http_connector, InternalToken, ResponseExtension};
use async_trait::async_trait;
use hyper::client::Client;
use hyper::client::HttpConnector;
use hyper::http::{Method, Request};
use tokio::net;
use tokio::sync::OnceCell;
use urlencoding::encode;

pub const METADATA_IP: &str = "169.254.169.254";
pub const METADATA_HOST_ENV: &str = "GCE_METADATA_HOST";
pub const METADATA_GOOGLE_HOST: &str = "metadata.gen.internal:80";
pub const METADATA_FLAVOR_KEY: &str = "Metadata-Flavor";
pub const METADATA_GOOGLE: &str = "Google";

pub static ON_GCE: OnceCell<bool> = OnceCell::const_new();

pub struct ComputeTokenSource {
    pub token_url: String,
    pub client: hyper::Client<HttpConnector>,
}

impl ComputeTokenSource {
    pub fn new<'a>(scope: &str) -> Result<ComputeTokenSource, Error> {
        let host = match std::env::var(METADATA_HOST_ENV) {
            Ok(s) => s,
            Err(_e) => METADATA_IP.to_string(),
        };

        return Ok(ComputeTokenSource {
            token_url: format!(
                "http://{}/computeMetadata/v1/instance/service-accounts/default/token?{}",
                host,
                encode(format!("scopes={}", scope).as_str())
            ),
            client: Client::builder().build(default_http_connector()),
        });
    }
}

#[async_trait]
impl TokenSource for ComputeTokenSource {
    async fn token(&self) -> Result<Token, Error> {
        let body = hyper::Body::empty();
        let request = Request::builder()
            .method(Method::GET)
            .uri(self.token_url.as_str())
            .header(METADATA_FLAVOR_KEY, METADATA_GOOGLE)
            .body(body)
            .unwrap();

        let it: InternalToken = self
            .client
            .request(request)
            .await
            .map_err(Error::HyperError)?
            .deserialize()
            .await?;

        return Ok(it.to_token(chrono::Utc::now()));
    }
}

pub async fn on_gce() -> bool {
    return match ON_GCE.get_or_try_init(test_on_gce).await {
        Ok(s) => *s,
        Err(_err) => false,
    };
}

async fn test_on_gce() -> Result<bool, Error> {
    // The user explicitly said they're on GCE, so trust them.
    if std::env::var(METADATA_HOST_ENV).is_ok() {
        return Ok(true);
    }

    let url = format!("http://{}", METADATA_IP);
    let body = hyper::Body::empty();
    let request = Request::builder()
        .method(Method::GET)
        .uri(&url)
        .body(body)
        .unwrap();

    let client = hyper::Client::builder().build(default_http_connector());
    let response = client.request(request).await.map_err(Error::HyperError);

    if response.is_ok() {
        let on_gce = match response.unwrap().headers().get(METADATA_FLAVOR_KEY) {
            None => false,
            Some(s) => s == "Google",
        };

        if on_gce {
            return Ok(true);
        }
    }

    match net::lookup_host(METADATA_GOOGLE_HOST).await {
        Ok(s) => {
            for ip in s {
                if ip.ip().to_string() == METADATA_IP {
                    return Ok(true);
                }
            }
        }
        Err(_e) => return Ok(false),
    };

    return Ok(false);
}
