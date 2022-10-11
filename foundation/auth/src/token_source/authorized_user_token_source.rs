use crate::credentials;
use crate::error::Error;
use crate::misc::{UnwrapOrEmpty, EMPTY};
use crate::token::{Token, TOKEN_URL};
use crate::token_source::TokenSource;
use crate::token_source::{default_http_client, InternalToken};
use async_trait::async_trait;

#[allow(dead_code)]
#[derive(Debug)]
pub struct UserAccountTokenSource {
    client_id: String,
    client_secret: String,
    token_url: String,
    redirect_url: String,
    refresh_token: String,

    client: reqwest::Client,
}

impl UserAccountTokenSource {
    pub(crate) fn new(cred: &credentials::CredentialsFile) -> Result<UserAccountTokenSource, Error> {
        if cred.refresh_token.is_none() {
            return Err(Error::RefreshTokenIsRequired);
        }

        let ts = UserAccountTokenSource {
            client_id: cred.client_id.unwrap_or_empty(),
            client_secret: cred.client_secret.unwrap_or_empty(),
            token_url: match &cred.token_uri {
                None => TOKEN_URL.to_string(),
                Some(s) => s.to_string(),
            },
            redirect_url: EMPTY.to_string(),
            refresh_token: cred.refresh_token.unwrap_or_empty(),
            client: default_http_client(),
        };
        Ok(ts)
    }
}

#[derive(serde::Serialize)]
struct RequestBody<'a> {
    pub client_id: &'a str,
    pub client_secret: &'a str,
    pub grant_type: &'a str,
    pub refresh_token: &'a str,
}

#[async_trait]
impl TokenSource for UserAccountTokenSource {
    async fn token(&self) -> Result<Token, Error> {
        let data = RequestBody {
            client_id: &self.client_id,
            client_secret: &self.client_secret,
            grant_type: "refresh_token",
            refresh_token: &self.refresh_token,
        };

        let it = self
            .client
            .post(self.token_url.to_string())
            .json(&data)
            .send()
            .await?
            .json::<InternalToken>()
            .await?;

        return Ok(it.to_token(time::OffsetDateTime::now_utc()));
    }
}
