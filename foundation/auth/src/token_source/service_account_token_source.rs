use std::collections::HashMap;
use std::fmt::Debug;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::credentials;
use crate::error::Error;
use crate::misc::UnwrapOrEmpty;
use crate::token::{Token, TOKEN_URL};
use crate::token_source::{default_http_client, InternalIdToken, InternalToken, TokenSource};

#[derive(Clone, Serialize)]
struct Claims<'a> {
    iss: &'a str,
    sub: Option<&'a str>,
    scope: Option<&'a str>,
    aud: &'a str,
    exp: i64,
    iat: i64,
    #[serde(flatten)]
    private_claims: &'a HashMap<String, serde_json::Value>,
}

impl Claims<'_> {
    fn token(&self, pk: &jsonwebtoken::EncodingKey, pk_id: &str) -> Result<String, Error> {
        let mut header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
        header.kid = Some(pk_id.to_string());
        let v = jsonwebtoken::encode(&header, self, pk)?;
        Ok(v)
    }
}

// Does not use any OAuth2 flow but instead creates a JWT and sends that as the access token.
// The audience is typically a URL that specifies the scope of the credentials.
// see golang.org/x/oauth2/gen/jwt.go
#[allow(dead_code)]
pub struct ServiceAccountTokenSource {
    email: String,
    pk: jsonwebtoken::EncodingKey,
    pk_id: String,
    audience: String,
    use_id_token: bool,
    private_claims: HashMap<String, serde_json::Value>,
}

impl Debug for ServiceAccountTokenSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // jwt::EncodingKey does not implement Debug
        f.debug_struct("ServiceAccountTokenSource")
            .field("email", &self.email)
            .field("pk_id", &self.pk_id)
            .field("audience", &self.audience)
            .finish()
    }
}

impl ServiceAccountTokenSource {
    pub(crate) fn new(cred: &credentials::CredentialsFile, audience: &str) -> Result<ServiceAccountTokenSource, Error> {
        Ok(ServiceAccountTokenSource {
            email: cred.client_email.unwrap_or_empty(),
            pk: cred.try_to_private_key()?,
            pk_id: cred.private_key_id.unwrap_or_empty(),
            audience: match &cred.audience {
                None => audience.to_string(),
                Some(s) => s.to_string(),
            },
            use_id_token: false,
            private_claims: HashMap::new(),
        })
    }
}

#[async_trait]
impl TokenSource for ServiceAccountTokenSource {
    async fn token(&self) -> Result<Token, Error> {
        let iat = OffsetDateTime::now_utc();
        let exp = iat + time::Duration::hours(1);

        let token = Claims {
            iss: self.email.as_ref(),
            sub: Some(self.email.as_ref()),
            scope: None,
            aud: self.audience.as_ref(),
            exp: exp.unix_timestamp(),
            iat: iat.unix_timestamp(),
            private_claims: &HashMap::new(),
        }
        .token(&self.pk, &self.pk_id)?;

        return Ok(Token {
            access_token: token,
            token_type: "Bearer".to_string(),
            expiry: Some(exp),
        });
    }
}

#[allow(dead_code)]
#[derive(Clone, Deserialize)]
struct OAuth2Token {
    pub access_token: String,
    pub token_type: String,
    pub id_token: Option<String>,
    pub expires_in: Option<i64>,
}

//jwt implements the OAuth 2.0 JSON Web Token flow
pub struct OAuth2ServiceAccountTokenSource {
    pub email: String,
    pub pk: jsonwebtoken::EncodingKey,
    pub pk_id: String,
    pub scopes: String,
    pub token_url: String,
    pub sub: Option<String>,

    pub client: reqwest::Client,

    use_id_token: bool,
    private_claims: HashMap<String, serde_json::Value>,
}

impl Debug for OAuth2ServiceAccountTokenSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // jwt::EncodingKey does not implement Debug
        f.debug_struct("OAuth2ServiceAccountTokenSource")
            .field("email", &self.email)
            .field("pk_id", &self.pk_id)
            .field("scopes", &self.scopes)
            .field("token_url", &self.token_url)
            .field("sub", &self.sub)
            .field("client", &self.client)
            .field("use_id_token", &self.use_id_token)
            .field("private_claims", &self.private_claims)
            .finish()
    }
}

impl OAuth2ServiceAccountTokenSource {
    pub(crate) fn new(
        cred: &credentials::CredentialsFile,
        scopes: &str,
        sub: Option<&str>,
    ) -> Result<OAuth2ServiceAccountTokenSource, Error> {
        Ok(OAuth2ServiceAccountTokenSource {
            email: cred.client_email.unwrap_or_empty(),
            pk: cred.try_to_private_key()?,
            pk_id: cred.private_key_id.unwrap_or_empty(),
            scopes: scopes.to_string(),
            token_url: match &cred.token_uri {
                None => TOKEN_URL.to_string(),
                Some(s) => s.to_string(),
            },
            client: default_http_client(),
            sub: sub.map(|s| s.to_string()),
            use_id_token: false,
            private_claims: HashMap::new(),
        })
    }

    pub(crate) fn with_use_id_token(mut self) -> Self {
        self.use_id_token = true;
        self
    }

    pub(crate) fn with_private_claims(mut self, claims: HashMap<String, serde_json::Value>) -> Self {
        self.private_claims = claims;
        self
    }
}

#[async_trait]
impl TokenSource for OAuth2ServiceAccountTokenSource {
    async fn token(&self) -> Result<Token, Error> {
        let iat = OffsetDateTime::now_utc();
        let exp = iat + time::Duration::hours(1);

        let request_token = Claims {
            iss: self.email.as_ref(),
            sub: self.sub.as_ref().map(|s| s.as_ref()),
            scope: Some(self.scopes.as_ref()),
            aud: self.token_url.as_ref(),
            exp: exp.unix_timestamp(),
            iat: iat.unix_timestamp(),
            private_claims: &self.private_claims,
        }
        .token(&self.pk, &self.pk_id)?;

        let form = [
            ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
            ("assertion", request_token.as_str()),
        ];

        match self.use_id_token {
            true => Ok(self
                .client
                .post(self.token_url.as_str())
                .form(&form)
                .send()
                .await?
                .json::<InternalIdToken>()
                .await?
                .to_token()?),
            false => Ok(self
                .client
                .post(self.token_url.as_str())
                .form(&form)
                .send()
                .await?
                .json::<InternalToken>()
                .await?
                .to_token(iat)),
        }
    }
}
