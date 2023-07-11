use crate::credentials;
use crate::credentials::{CredentialSource, ServiceAccountImpersonationInfo};
use crate::error::Error;
use crate::misc::UnwrapOrEmpty;
use crate::token::Token;
use crate::token_source::{default_http_client, InternalToken, TokenSource};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use serde_json::json;
use std::collections::HashMap;
use time::OffsetDateTime;
use crate::subject_token_source::SubjectTokenSource;

#[allow(dead_code)]
#[derive(Debug)]
pub struct ExternalAccountTokenSource {
    audience: Option<String>,
    subject_token_type: Option<String>,
    token_url_external: Option<String>,
    credential_source: CredentialSource,
    scopes: String,
    auth_header: Option<String>,
    workforce_options: Option<String>,
    subject_token_source: Box<dyn SubjectTokenSource>,
    client: reqwest::Client,
}

impl ExternalAccountTokenSource {
    pub(crate) async fn new(scopes: &str, cred: &credentials::CredentialsFile) -> Result<ExternalAccountTokenSource, Error> {
        let auth_header = if cred.client_id.is_some() && cred.client_secret.is_some() {
            let plain_text = format!("{}:{}", cred.client_id.unwrap(), cred.client_secret.unwrap());
            Some(format!("Basic: {}", BASE64_STANDARD.encode(plain_text)))
        } else {
            None
        };
        // Do not pass workforce_pool_user_project when client authentication is used.
        // The client ID is sufficient for determining the user project.
        let workforce_options = if cred.workforce_pool_user_project.is_some() && cred.client_id.is_none() {
            let mut option = HashMap::with_capacity(1);
            option.insert("userProject", cred.workforce_pool_user_project.unwrap());
            Some(serde_json::to_string(&option)?)
        } else {
            None
        };
        Ok(ExternalAccountTokenSource {
            audience: cred.audience.clone(),
            subject_token_type: cred.subject_token_type.clone(),
            token_url_external: cred.token_url_external.clone(),
            credential_source: cred.credential_source.clone().ok_or(Error::NoCredentialsSource)?,
            scopes: scopes.to_string(),
            auth_header,
            workforce_options,
            subject_token_source: cred.subject_token_source().await?,
            client: default_http_client(),
        })
    }
}

#[async_trait]
impl TokenSource for ExternalAccountTokenSource {
    async fn token(&self) -> Result<Token, Error> {
        let mut builder = self.client.post(&self.token_url_external);

        if let Some(auth_header) = &self.auth_header {
            builder = builder.header(reqwest::header::AUTHORIZATION, auth_header);
        }

        let mut sts_request = [
            ("grant_type", "urn:ietf:params:oauth:grant-type:token-exchange"),
            ("audience", self.audience.as_str()),
            ("scope", self.scopes.as_str()),
            ("subject_token_type", self.subject_token_type.unwrap_or_empty()),
            ("subject_token", &self.subject_token_source.subject_token().await?),
            ("requested_token_type", "urn:ietf:params:oauth:token-type:access_token"),
        ];
        if let Some(options) = &self.workforce_options {
            sts_request.push(("options", options));
        }

        let it = builder.form(&sts_request).send().await?.json::<InternalToken>().await?;
        Ok(it.to_token(OffsetDateTime::now_utc()))
    }
}
