use async_trait::async_trait;

use crate::credentials::Format;

use super::{error::Error, subject_token_source::SubjectTokenSource};

pub struct FileCredentialSource {
    file: String,
    format: Option<Format>,
}

impl FileCredentialSource {
    pub fn new(file: String, format: Option<Format>) -> Self {
        Self { file, format }
    }

    async fn read_credential(&self) -> Result<String, Error> {
        let content = tokio::fs::read_to_string(&self.file).await?;
        match self.format.as_ref().map(|f| f.tp.as_str()).unwrap_or("") {
            "json" => {
                let data: serde_json::Value = serde_json::from_str(&content)?;
                if let Some(token) = data[&self.format.as_ref().unwrap().subject_token_field_name].as_str() {
                    Ok(token.to_string())
                } else {
                    Err(Error::MissingSubjectTokenFieldName)
                }
            }
            "text" | "" => Ok(content),
            _ => Err(Error::UnsupportedFormatType),
        }
    }
}

#[async_trait]
impl SubjectTokenSource for FileCredentialSource {
    async fn subject_token(&self) -> Result<String, Error> {
        self.read_credential().await
    }
}
