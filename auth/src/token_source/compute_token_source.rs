use crate::error::Error;
use crate::token::Token;
use crate::token_source::TokenSource;
use crate::token_source::{InternalToken, ResponseExtension};
use async_trait::async_trait;
use google_cloud_metadata::{
    default_http_connector, METADATA_FLAVOR_KEY, METADATA_GOOGLE, METADATA_HOST_ENV, METADATA_IP,
};
use hyper::client::Client;
use hyper::client::HttpConnector;
use hyper::http::{Method, Request};
use urlencoding::encode;

pub struct ComputeTokenSource {
    token_url: String,
    client: hyper::Client<HttpConnector>,
}

impl ComputeTokenSource {
    pub(crate) fn new(scope: &str) -> Result<ComputeTokenSource, Error> {
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
            .body(body)?;

        let it: InternalToken = self.client.request(request).await?.deserialize().await?;

        return Ok(it.to_token(chrono::Utc::now()));
    }
}
