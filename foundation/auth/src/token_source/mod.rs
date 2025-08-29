use std::fmt::Debug;
use std::time::Duration;

use async_trait::async_trait;
use jsonwebtoken;
use serde::Deserialize;

use crate::error::Error;
use crate::token::Token;
// pub use token_source::TokenSource;

pub mod authorized_user_token_source;
pub mod compute_identity_source;
pub mod compute_token_source;
pub mod impersonate_token_source;
pub mod reuse_token_source;
pub mod service_account_token_source;

#[cfg(feature = "external-account")]
pub mod external_account_source;

#[async_trait]
pub trait GoogleCloudTokenSource: Send + Sync + Debug {
    async fn token(&self) -> Result<Token, Error>;
}

pub(crate) fn default_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap()
}

#[derive(Clone, Deserialize)]
struct InternalToken {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<i64>,
}

impl InternalToken {
    fn to_token(&self, now: time::OffsetDateTime) -> Token {
        Token {
            access_token: self.access_token.clone(),
            token_type: self.token_type.clone(),
            expiry: self.expires_in.map(|s| now + time::Duration::seconds(s)),
        }
    }
}

#[derive(Clone, Deserialize)]
struct InternalIdToken {
    pub id_token: String,
}

#[derive(Deserialize)]
struct ExpClaim {
    exp: i64,
}

impl InternalIdToken {
    fn to_token(&self, audience: &str) -> Result<Token, Error> {
        Ok(Token {
            access_token: self.id_token.clone(),
            token_type: "Bearer".into(),
            expiry: time::OffsetDateTime::from_unix_timestamp(self.get_exp(audience)?).ok(),
        })
    }

    fn get_exp(&self, audience: &str) -> Result<i64, Error> {
        let mut validation = jsonwebtoken::Validation::default();
        validation.insecure_disable_signature_validation();
        validation.set_audience(&[audience]);
        let decoding_key = jsonwebtoken::DecodingKey::from_secret(b"");
        Ok(
            jsonwebtoken::decode::<ExpClaim>(self.id_token.as_str(), &decoding_key, &validation)?
                .claims
                .exp,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::credentials::CredentialsFile;
    use crate::error::Error;
    use crate::token_source::service_account_token_source::{
        OAuth2ServiceAccountTokenSource, ServiceAccountTokenSource,
    };
    use crate::token_source::GoogleCloudTokenSource;

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
        let sub = None;
        let ts = OAuth2ServiceAccountTokenSource::new(&credentials, scope, sub)?;
        let token = ts.token().await?;
        assert_eq!("Bearer", token.token_type);
        assert!(token.expiry.unwrap().unix_timestamp() > 0);
        Ok(())
    }
}
