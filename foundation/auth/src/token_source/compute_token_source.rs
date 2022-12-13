use crate::error::Error;
use crate::token::Token;
use crate::token_source::InternalToken;
use crate::token_source::{default_http_client, TokenSource};
use async_trait::async_trait;
use google_cloud_metadata::{METADATA_FLAVOR_KEY, METADATA_GOOGLE, METADATA_HOST_ENV, METADATA_IP};
use urlencoding::encode;

#[allow(dead_code)]
#[derive(Debug)]
pub struct ComputeTokenSource {
    token_url: String,
    client: reqwest::Client,
}

impl ComputeTokenSource {
    pub(crate) fn new(scope: &str) -> Result<ComputeTokenSource, Error> {
        let host = match std::env::var(METADATA_HOST_ENV) {
            Ok(s) => s,
            Err(_e) => METADATA_IP.to_string(),
        };

        Ok(ComputeTokenSource {
            token_url: format!(
                "http://{}/computeMetadata/v1/instance/service-accounts/default/token?{}",
                host,
                encode(format!("scopes={}", scope).as_str())
            ),
            client: default_http_client(),
        })
    }
}

#[async_trait]
impl TokenSource for ComputeTokenSource {
    async fn token(&self) -> Result<Token, Error> {
        let it = self
            .client
            .get(self.token_url.to_string())
            .header(METADATA_FLAVOR_KEY, METADATA_GOOGLE)
            .send()
            .await?
            .json::<InternalToken>()
            .await?;
        return Ok(it.to_token(time::OffsetDateTime::now_utc()));
    }
}
