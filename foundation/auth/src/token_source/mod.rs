pub mod authorized_user_token_source;
pub mod compute_token_source;
pub mod reuse_token_source;
pub mod service_account_token_source;

use crate::error::Error;
use crate::token::Token;
use async_trait::async_trait;
use serde::Deserialize;
use std::fmt::Debug;
use std::time::Duration;

#[async_trait]
pub trait TokenSource: Send + Sync + Debug {
    async fn token(&self) -> Result<Token, Error>;
}

fn default_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap()
}

#[allow(dead_code)]
#[derive(Clone, Deserialize)]
struct InternalToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<i64>,
    pub id_token: Option<String>,
}

impl InternalToken {
    fn to_token(&self, now: time::OffsetDateTime) -> Token {
        //TODO support use ID token
        Token {
            access_token: self.access_token.clone(),
            token_type: self.token_type.clone(),
            expiry: self.expires_in.map(|s| now + time::Duration::seconds(s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::credentials::CredentialsFile;
    use crate::error::Error;

    use crate::token_source::service_account_token_source::{
        OAuth2ServiceAccountTokenSource, ServiceAccountTokenSource,
    };
    use crate::token_source::TokenSource;

    #[tokio::test]
    async fn test_jwt_token_source() -> Result<(), Error> {
        let credentials = CredentialsFile::new().await?;
        let audience = "https://spanner.googleapis.com/";
        let ts = ServiceAccountTokenSource::new(&credentials, audience)?;
        let token = ts.token().await?;
        assert_eq!("Bearer", token.token_type);
        assert!(token.expiry.unwrap().unix_timestamp() > 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_oauth2_token_source() -> Result<(), Error> {
        let credentials = CredentialsFile::new().await?;
        let scope = "https://www.googleapis.com/auth/cloud-platform https://www.googleapis.com/auth/spanner.data";
        let ts = OAuth2ServiceAccountTokenSource::new(&credentials, scope)?;
        let token = ts.token().await?;
        assert_eq!("Bearer", token.token_type);
        assert!(token.expiry.unwrap().unix_timestamp() > 0);
        Ok(())
    }
}
