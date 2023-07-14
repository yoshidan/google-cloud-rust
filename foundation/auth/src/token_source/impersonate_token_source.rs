use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;

use crate::error::Error;
use crate::token::Token;
use crate::token_source::{default_http_client, TokenSource};

#[derive(Debug)]
pub struct ImpersonateTokenSource {
    target: Box<dyn TokenSource>,
    lifetime: Option<i32>,
    scopes: Vec<String>,
    delegates: Vec<String>,
    url: String,
    client: reqwest::Client,
}

impl ImpersonateTokenSource {
    pub(crate) fn new(
        url: String,
        delegates: Vec<String>,
        scopes: Vec<String>,
        lifetime: Option<i32>,
        target: Box<dyn TokenSource>,
    ) -> Self {
        ImpersonateTokenSource {
            lifetime,
            target,
            scopes,
            delegates,
            url,
            client: default_http_client(),
        }
    }
}

#[async_trait]
impl TokenSource for ImpersonateTokenSource {
    async fn token(&self) -> Result<Token, Error> {
        let body = ImpersonateTokenRequest {
            lifetime: format!("{}s", self.lifetime.unwrap_or(3600)),
            scope: self.scopes.clone(),
            delegates: self.delegates.clone(),
        };

        let auth_token = self.target.token().await?;
        let response = self
            .client
            .post(&self.url)
            .json(&body)
            .header(
                "Authorization",
                format!("{} {}", auth_token.token_type, auth_token.access_token),
            )
            .send()
            .await?;
        let response = if !response.status().is_success() {
            let status = response.status().as_u16();
            return Err(Error::UnexpectedImpersonateTokenResponse(status, response.text().await?));
        } else {
            response.json::<ImpersonateTokenResponse>().await?
        };

        let expiry = time::OffsetDateTime::parse(&response.expire_time, &Rfc3339)?;
        Ok(Token {
            access_token: response.access_token,
            token_type: "Bearer".to_string(),
            expiry: Some(expiry),
        })
    }
}

#[derive(Serialize)]
struct ImpersonateTokenRequest {
    pub delegates: Vec<String>,
    pub lifetime: String,
    pub scope: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImpersonateTokenResponse {
    pub access_token: String,
    pub expire_time: String,
}
