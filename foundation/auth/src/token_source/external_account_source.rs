use crate::credentials;
use crate::credentials::{CredentialSource, ServiceAccountImpersonationInfo};
use crate::error::Error;
use crate::misc::UnwrapOrEmpty;
use crate::token::Token;
use crate::token_source::{default_http_client, InternalToken, TokenSource};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use serde_json::json;
use time::OffsetDateTime;

#[allow(dead_code)]
#[derive(Debug)]
pub struct ExternalAccountTokenSource {
    audience: Option<String>,
    subject_token_type: Option<String>,
    token_url_external: Option<String>,
    token_info_url: Option<String>,
    service_account_impersonation_url: Option<String>,
    service_account_impersonation_lifetime_seconds: Option<i32>,
    client_secret: Option<String>,
    client_id: Option<String>,
    credential_source: CredentialSource,
    quota_project_id: Option<String>,
    workforce_pool_user_project: Option<String>,
    scopes: String,
    client: reqwest::Client,
}

impl ExternalAccountTokenSource {
    pub(crate) fn new(scopes: &str, cred: &credentials::CredentialsFile) -> Result<ExternalAccountTokenSource, Error> {
        Ok(ExternalAccountTokenSource {
            audience: cred.audience.clone(),
            subject_token_type: cred.subject_token_type.clone(),
            token_url_external: cred.token_url_external.clone(),
            token_info_url: cred.token_info_url.clone(),
            service_account_impersonation_url: cred.service_account_impersonation_url.clone(),
            service_account_impersonation_lifetime_seconds: None, //TODO impersonate token source
            client_id: cred.client_id.clone(),
            client_secret: cred.client_secret.clone(),
            credential_source: cred.credential_source.clone().ok_or(Error::NoCredentialsSource)?,
            quota_project_id: cred.quota_project_id.clone(),
            workforce_pool_user_project: None, //TODO workforce identity
            scopes: scopes.to_string(),
            client: default_http_client(),
        })
    }
}

#[async_trait]
impl TokenSource for ExternalAccountTokenSource {
    async fn token(&self) -> Result<Token, Error> {
        let subject_token = self.credential_source.subject_token().await?;
        let sts_request = [
            ("grant_type", "urn:ietf:params:oauth:grant-type:token-exchange"),
            ("audience", self.audience.as_str()),
            ("scope", self.scopes.as_str()),
            ("subject_token_type", self.subject_token_type.unwrap_or_empty()),
            ("subject_token", &subject_token),
            ("requested_token_type", "urn:ietf:params:oauth:token-type:access_token"),
        ];

        let mut builder = self.client.post(&self.token_url_external);

        if self.client_id.is_some() && self.client_secret.is_some() {
            let plain_text = format!("{}:{}", self.client_id.unwrap(), self.client_secret.unwrap());
            let auth_header = format!("Basic: {}", BASE64_STANDARD.encode(plain_text));
            builder = builder.header(reqwest::header::AUTHORIZATION, auth_header)
        }

        let it = builder.form(&sts_request)
            .send()
            .await?
            .json::<InternalToken>()
            .await?;
        Ok(it.to_token(OffsetDateTime::now_utc()))
    }
}
