use crate::error::Error;
use crate::token::{Token, TokenSource};
use crate::token_source::{InternalToken, ResponseExtension};
use async_trait::async_trait;
use hyper::client::Client;
use hyper::client::HttpConnector;
use hyper::http::{Method, Request};
use tokio::net;
use tokio::sync::OnceCell;
use urlencoding::encode;
use metadata::{METADATA_HOST_ENV, METADATA_IP, METADATA_FLAVOR_KEY, METADATA_GOOGLE, default_http_connector};

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
            .map_err(Error::HttpError)?;

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
