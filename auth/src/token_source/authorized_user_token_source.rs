use crate::credentials;
use crate::error::Error;
use crate::misc::{UnwrapOrEmpty, EMPTY};
use crate::token::{Token, TOKEN_URL};
use crate::token_source::token_source::TokenSource;
use crate::token_source::{default_https_client, InternalToken, ResponseExtension};
use async_trait::async_trait;
use hyper::client::HttpConnector;
use hyper::http::{Method, Request};
use hyper::{Body, Client};

pub struct UserAccountTokenSource {
    client_id: String,
    client_secret: String,
    token_url: String,
    #[allow(dead_code)]
    redirect_url: String,
    refresh_token: String,

    client: Client<hyper_tls::HttpsConnector<HttpConnector>>,
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
            client: default_https_client(),
        };
        return Ok(ts);
    }
}

#[async_trait]
impl TokenSource for UserAccountTokenSource {
    async fn token(&self) -> Result<Token, Error> {
        let data = json::json!({
            "client_id": self.client_id,
            "client_secret": self.client_secret,
            "grant_type": "refresh_token".to_string(),
            "refresh_token": self.refresh_token,
        })
        .to_string();

        let request = Request::builder()
            .method(Method::POST)
            .uri(self.token_url.to_string())
            .header("content-type", "application/json")
            .body(Body::from(data))?;

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
