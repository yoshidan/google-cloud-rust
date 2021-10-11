pub mod authorized_user_token_source;
pub mod compute_token_source;
pub mod reuse_token_source;
pub mod service_account_token_source;

use crate::error::Error;
use crate::token::Token;
use async_trait::async_trait;
use chrono::Duration;
use hyper::client::HttpConnector;
use hyper::http::Response;
use hyper_tls::HttpsConnector;
use serde::{de, Deserialize};

fn default_http_connector() -> HttpConnector {
    let mut connector = HttpConnector::new();
    connector.enforce_http(false);
    connector.set_connect_timeout(Some(Duration::seconds(2).to_std().unwrap()));
    connector.set_keepalive(Some(Duration::seconds(30).to_std().unwrap()));
    connector
}

fn default_https_client() -> hyper::Client<HttpsConnector<HttpConnector>> {
    hyper::Client::builder().build(HttpsConnector::new_with_connector(default_http_connector()))
}

#[async_trait]
trait ResponseExtension {
    async fn deserialize<T>(self) -> Result<T, Error>
    where
        T: de::DeserializeOwned;
}

#[async_trait]
impl ResponseExtension for Response<hyper::body::Body> {
    async fn deserialize<T>(self) -> Result<T, Error>
    where
        T: de::DeserializeOwned,
    {
        if !self.status().is_success() {
            return Err(Error::StringError(format!(
                "Server responded with error status is {}",
                self.status().as_str()
            )));
        }
        let (_, body) = self.into_parts();
        let body = hyper::body::to_bytes(body)
            .await
            .map_err(Error::HyperError)?;
        let token = json::from_slice(&body).map_err(Error::JsonError)?;

        Ok(token)
    }
}

#[derive(Clone, Deserialize)]
struct InternalToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<i64>,
    pub id_token: Option<String>,
}

impl InternalToken {
    fn to_token(&self, now: chrono::DateTime<chrono::Utc>) -> Token {
        //TODO support use ID token
        Token {
            access_token: self.access_token.clone(),
            token_type: self.token_type.clone(),
            expiry: match self.expires_in {
                Some(s) => Some(now + chrono::Duration::seconds(s)),
                None => None,
            },
        }
    }
}
