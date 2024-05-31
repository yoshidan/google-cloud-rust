use std::fmt::{Debug, Formatter};

use async_trait::async_trait;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use time::OffsetDateTime;

use crate::credentials::{CredentialSource, CredentialsFile};
use crate::misc::UnwrapOrEmpty;
use crate::token::Token;
use crate::token_source::external_account_source::error::Error;
use crate::token_source::external_account_source::subject_token_source::SubjectTokenSource;
use crate::token_source::{default_http_client, InternalToken, TokenSource};

mod aws_subject_token_source;
pub mod error;
mod file_credential_source;
mod subject_token_source;
mod url_subject_token_source;

pub struct ExternalAccountTokenSource {
    source: CredentialSource,
    subject_token_type: String,
    url: String,
    audience: Option<String>,
    auth_header: Option<String>,
    scopes: String,
    client: reqwest::Client,
}

impl Debug for ExternalAccountTokenSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExternalAccountTokenSource").finish()
    }
}

impl ExternalAccountTokenSource {
    pub(crate) async fn new(scopes: String, credentials: CredentialsFile) -> Result<ExternalAccountTokenSource, Error> {
        let auth_header = if credentials.client_id.is_some() && credentials.client_secret.is_some() {
            let plain_text = format!(
                "{}:{}",
                credentials.client_id.as_ref().unwrap(),
                credentials.client_secret.as_ref().unwrap()
            );
            Some(format!("Basic: {}", BASE64_STANDARD.encode(plain_text)))
        } else {
            None
        };
        let subject_token_type = credentials.subject_token_type.ok_or(Error::MissingSubjectTokenType)?;
        Ok(ExternalAccountTokenSource {
            source: credentials.credential_source.ok_or(Error::NoCredentialsSource)?,
            subject_token_type,
            url: credentials.token_url_external.ok_or(Error::MissingTokenURL)?,
            audience: credentials.audience,
            auth_header,
            scopes,
            client: default_http_client(),
        })
    }
}

#[async_trait]
impl TokenSource for ExternalAccountTokenSource {
    async fn token(&self) -> Result<Token, crate::error::Error> {
        let subject_token_source = subject_token_source(self.audience.clone(), self.source.clone()).await?;

        let mut builder = self.client.post(&self.url);
        if let Some(auth_header) = &self.auth_header {
            builder = builder.header(reqwest::header::AUTHORIZATION, auth_header);
        }

        let audience = match self.audience.as_ref() {
            Some(audience) => audience.as_ref(),
            None => "",
        };

        let subject_token = subject_token_source.subject_token().await?;
        let sts_request = vec![
            ("grant_type", "urn:ietf:params:oauth:grant-type:token-exchange"),
            ("audience", audience),
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

async fn subject_token_source(
    audience: Option<String>,
    source: CredentialSource,
) -> Result<Box<dyn SubjectTokenSource>, Error> {
    let environment_id = &source.environment_id.unwrap_or_empty();
    if environment_id.len() > 3 && environment_id.starts_with("aws") {
        if environment_id != "aws1" {
            return Err(Error::UnsupportedAWSVersion(environment_id.clone()));
        }
        let ts = aws_subject_token_source::AWSSubjectTokenSource::new(audience, source).await?;
        Ok(Box::new(ts))
    } else if let Some(_) = source.url {
        let ts = url_subject_token_source::UrlSubjectTokenSource::new(source).await?;
        Ok(Box::new(ts))
    } else if let Some(file) = source.file {
        let ts = file_credential_source::FileCredentialSource::new(file, source.format);
        Ok(Box::new(ts))
    } else {
        // TODO: support executable type
        Err(Error::UnsupportedSubjectTokenSource)
    }
}
