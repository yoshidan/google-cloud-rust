use std::fmt::Debug;

use async_trait::async_trait;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use time::OffsetDateTime;

use crate::credentials::CredentialsFile;
use crate::misc::UnwrapOrEmpty;
use crate::token::Token;
use crate::token_source::external_account_source::error::Error;
use crate::token_source::{default_http_client, InternalToken, TokenSource};

mod aws_subject_token_source;
pub mod error;
mod subject_token_source;

#[derive(Debug)]
pub struct ExternalAccountTokenSource {
    audience: String,
    subject_token_type: String,
    token_url: String,
    scopes: String,
    auth_header: Option<String>,
    subject_token_source: Box<dyn subject_token_source::SubjectTokenSource>,
    client: reqwest::Client,
}

impl ExternalAccountTokenSource {
    pub(crate) async fn new(scopes: &str, cred: &CredentialsFile) -> Result<ExternalAccountTokenSource, Error> {
        let auth_header = if cred.client_id.is_some() && cred.client_secret.is_some() {
            let plain_text = format!("{}:{}", cred.client_id.as_ref().unwrap(), cred.client_secret.as_ref().unwrap());
            Some(format!("Basic: {}", BASE64_STANDARD.encode(plain_text)))
        } else {
            None
        };
        Ok(ExternalAccountTokenSource {
            audience: cred.audience.clone().unwrap_or_empty(),
            subject_token_type: cred.subject_token_type.clone().ok_or(Error::MissingSubjectTokenType)?,
            token_url: cred.token_url_external.clone().ok_or(Error::MissingTokenURL)?,
            scopes: scopes.to_string(),
            auth_header,
            subject_token_source: subject_token_source(cred).await?,
            client: default_http_client(),
        })
    }
}

#[async_trait]
impl TokenSource for ExternalAccountTokenSource {
    async fn token(&self) -> Result<Token, crate::error::Error> {
        let mut builder = self.client.post(&self.token_url);

        if let Some(auth_header) = &self.auth_header {
            builder = builder.header(reqwest::header::AUTHORIZATION, auth_header);
        }

        let subject_token = self.subject_token_source.subject_token().await?;
        let sts_request = vec![
            ("grant_type", "urn:ietf:params:oauth:grant-type:token-exchange"),
            ("audience", &self.audience),
            ("scope", &self.scopes),
            ("subject_token_type", &self.subject_token_type),
            ("subject_token", &subject_token),
            ("requested_token_type", "urn:ietf:params:oauth:token-type:access_token"),
        ];
        let response = builder.form(&sts_request).send().await?;
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let detail = response.text().await?;
            return Err(Error::UnexpectedStatusOnGetSubjectToken(status, detail).into());
        }
        let it = response.json::<InternalToken>().await?;
        Ok(it.to_token(OffsetDateTime::now_utc()))
    }
}

pub(crate) async fn subject_token_source(
    credentials: &CredentialsFile,
) -> Result<Box<dyn subject_token_source::SubjectTokenSource>, Error> {
    let source = credentials
        .credential_source
        .as_ref()
        .ok_or(Error::NoCredentialsSource)?;
    let environment_id = &source.environment_id.unwrap_or_empty();
    if environment_id.len() > 3 && environment_id.starts_with("aws") {
        if environment_id != "aws1" {
            return Err(Error::UnsupportedAWSVersion(environment_id.clone()));
        }
        let ts =
            aws_subject_token_source::AWSSubjectTokenSource::new(credentials.audience.clone(), source.clone()).await?;
        Ok(Box::new(ts))
    } else {
        //TODO support file, url and executable
        Err(Error::UnsupportedSubjectTokenSource)
    }
}
