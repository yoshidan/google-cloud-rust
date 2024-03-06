use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use url::Url;

use crate::credentials::{CredentialSource, Format};
use crate::token_source::default_http_client;
use crate::token_source::external_account_source::error::Error;
use crate::token_source::external_account_source::subject_token_source::SubjectTokenSource;

pub struct UrlSubjectTokenSource {
    url: Url,
    headers: HashMap<String, String>,
    format: Format,
}

impl UrlSubjectTokenSource {
    pub async fn new(value: CredentialSource) -> Result<Self, Error> {
        let url = value.url.ok_or(Error::MissingTokenURL)?;
        let url = Url::parse(&url).map_err(Error::URLError)?;
        let headers = value.headers.ok_or(Error::MissingHeaders)?;
        let format = value.format.ok_or(Error::MissingFormat)?;

        Ok(Self { url, headers, format })
    }

    async fn create_subject_token(&self) -> Result<String, Error> {
        let client = default_http_client();
        let mut request = client.get(self.url.clone());

        for (key, val) in &self.headers {
            request = request.header(key, val);
        }

        let response = request.send().await.map_err(Error::HttpError)?;

        if !response.status().is_success() {
            return Err(Error::UnexpectedStatusOnGetSessionToken(response.status().as_u16()));
        }

        let body = response.text_with_charset("utf-8").await?;
        let limit = body.chars().take(1 << 20).collect::<String>(); // Limiting the response body to 1MB

        let format_type = self.format.tp.as_str();
        match format_type {
            "json" => {
                let data: Value = serde_json::from_str(&limit).map_err(Error::JsonError)?;
                if let Some(token) = data[&self.format.subject_token_field_name].as_str() {
                    Ok(token.to_string())
                } else {
                    Err(Error::MissingSubjectTokenFieldName)
                }
            }
            "text" | "" => Ok(limit),
            _ => Err(Error::UnsupportedFormatType),
        }
    }
}

#[async_trait]
impl SubjectTokenSource for UrlSubjectTokenSource {
    async fn subject_token(&self) -> Result<String, Error> {
        self.create_subject_token().await
    }
}
