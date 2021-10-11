use crate::credentials;
use crate::error::Error;
use crate::token::{Token, TokenSource, TOKEN_URL};
use crate::token_source::{default_https_client, InternalToken, ResponseExtension};
use async_trait::async_trait;
use hyper::client::HttpConnector;
use hyper::http::{Method, Request};
use hyper::{Body, Client};

pub struct UserAccountTokenSource {
    pub client_id: String,
    pub client_secret: String,
    pub token_url: String,
    pub redirect_url: String,
    pub refresh_token: String,

    pub client: Client<hyper_tls::HttpsConnector<HttpConnector>>,
}

impl UserAccountTokenSource {
    pub fn new(cred: &credentials::CredentialsFile) -> Result<UserAccountTokenSource, Error> {
        if cred.refresh_token.is_none() {
            return Err(Error::StringError(
                "refresh token is required for user account credentials".to_string(),
            ));
        }

        let ts = UserAccountTokenSource {
            client_id: cred.client_id.as_ref().unwrap().to_string(),
            client_secret: cred.client_secret.as_ref().unwrap().to_string(),
            token_url: match &cred.token_uri {
                None => TOKEN_URL.to_string(),
                Some(s) => s.to_string(),
            },
            redirect_url: "".to_string(),
            refresh_token: cred.refresh_token.as_ref().unwrap().to_string(),
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
        });
        let request = Request::builder()
            .method(Method::POST)
            .uri(self.token_url.to_string())
            .header("content-type", "application/json")
            .body(Body::from(json::to_string(&data).unwrap()))
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
