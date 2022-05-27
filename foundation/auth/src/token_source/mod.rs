pub mod authorized_user_token_source;
pub mod compute_token_source;
pub mod reuse_token_source;
pub mod service_account_token_source;

use crate::error::Error;
use crate::token::Token;
use async_trait::async_trait;
use serde::Deserialize;
use std::fmt::{Debug, Formatter};
use std::time::Duration;

#[async_trait]
pub trait TokenSource: Send + Sync {
    async fn token(&self) -> Result<Token, Error>;
}

impl Debug for dyn TokenSource {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let token_source_string = format!("{:?}", self);
        write!(f, "{}", token_source_string.as_str())
    }
}

fn default_http_client() -> reqwest::Client {
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
    pub id_token: Option<String>,
}

impl InternalToken {
    fn to_token(&self, now: chrono::DateTime<chrono::Utc>) -> Token {
        //TODO support use ID token
        Token {
            access_token: self.access_token.clone(),
            token_type: self.token_type.clone(),
            expiry: self.expires_in.map(|s| now + chrono::Duration::seconds(s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::credentials::CredentialsFile;
    use crate::error::Error;
    use crate::token_source::authorized_user_token_source::UserAccountTokenSource;
    use crate::token_source::compute_token_source::ComputeTokenSource;
    use crate::token_source::reuse_token_source::ReuseTokenSource;
    use crate::token_source::service_account_token_source::{
        OAuth2ServiceAccountTokenSource, ServiceAccountTokenSource,
    };
    use crate::token_source::TokenSource;
    use std::fs::File;
    use std::io::Write;

    #[tokio::test]
    async fn test_user_account_token_source() -> Result<(), Error> {
        let authorized_user_credentials = std::env::var("TEST_USER_CREDENTIALS").map_err(Error::VarError)?;

        let json = base64::decode(authorized_user_credentials).unwrap();
        let mut file = File::create(".cred.json")?;
        file.write_all(json.as_slice())?;

        std::env::set_var("GOOGLE_APPLICATION_CREDENTIALS", ".cred.json");
        let credentials = CredentialsFile::new().await?;
        let ts = UserAccountTokenSource::new(&credentials)?;
        let token = ts.token().await?;
        assert_eq!("Bearer", token.token_type);
        assert_eq!(true, token.expiry.unwrap().timestamp() > 0);
        Ok(())
    }

    #[tokio::test]
    //  available on GCE only
    async fn test_compute_token_source() -> Result<(), Error> {
        let scope = "https://www.googleapis.com/auth/cloud-platform,https://www.googleapis.com/auth/spanner.data";
        let ts = ComputeTokenSource::new(scope);
        assert_eq!(true, ts.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn test_reuse_token_source() -> Result<(), Error> {
        let credentials = CredentialsFile::new().await?;
        let audience = "https://spanner.googleapis.com/";
        let ts = ServiceAccountTokenSource::new(&credentials, audience)?;
        let token = ts.token().await?;
        assert_eq!(true, token.expiry.unwrap().timestamp() > 0);
        let old_token_value = token.access_token.clone();
        let rts = ReuseTokenSource::new(Box::new(ts), token);
        let new_token = rts.token().await?;
        assert_eq!(old_token_value, new_token.access_token);
        Ok(())
    }

    #[tokio::test]
    async fn test_jwt_token_source() -> Result<(), Error> {
        let credentials = CredentialsFile::new().await?;
        let audience = "https://spanner.googleapis.com/";
        let ts = ServiceAccountTokenSource::new(&credentials, audience)?;
        let token = ts.token().await?;
        assert_eq!("Bearer", token.token_type);
        assert_eq!(true, token.expiry.unwrap().timestamp() > 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_oauth2_token_source() -> Result<(), Error> {
        let credentials = CredentialsFile::new().await?;
        let scope = "https://www.googleapis.com/auth/cloud-platform https://www.googleapis.com/auth/spanner.data";
        let ts = OAuth2ServiceAccountTokenSource::new(&credentials, scope)?;
        let token = ts.token().await?;
        assert_eq!("Bearer", token.token_type);
        assert_eq!(true, token.expiry.unwrap().timestamp() > 0);
        Ok(())
    }
}
