use std::fmt::{Debug, Formatter};
use std::sync::Arc;

use async_trait::async_trait;

use token_source::{TokenSource, TokenSourceProvider};

use crate::credentials::CredentialsFile;
use crate::error::Error;
use crate::project::{
    create_token_source_from_credentials, create_token_source_from_project, project, Config, Project,
};
use crate::token_source::TokenSource as InternalTokenSource;

pub const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

#[derive(Debug, Clone)]
pub struct Token {
    pub access_token: String,
    pub token_type: String,
    pub expiry: Option<time::OffsetDateTime>,
}

impl Token {
    pub fn value(&self) -> String {
        format!("Bearer {}", self.access_token)
    }

    pub fn valid(&self) -> bool {
        !self.access_token.is_empty() && !self.expired()
    }

    fn expired(&self) -> bool {
        match self.expiry {
            None => false,
            Some(s) => {
                let now = time::OffsetDateTime::now_utc();
                let exp = s + time::Duration::seconds(-10);
                now > exp
            }
        }
    }
}

pub struct DefaultTokenSourceProvider {
    ts: Arc<DefaultTokenSource>,
    pub project_id: Option<String>,
    pub source_credentials: Option<Box<CredentialsFile>>,
}

impl Debug for DefaultTokenSourceProvider {
    fn fmt(&self, _: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl DefaultTokenSourceProvider {
    pub async fn new(config: Config<'_>) -> Result<Self, Error> {
        let project = project().await?;
        let internal_token_source = create_token_source_from_project(&project, config).await?;

        let (project_id, source_credentials) = match project {
            Project::FromMetadataServer(info) => (info.project_id, None),
            Project::FromFile(cred) => {
                (cred.project_id.as_ref().or(cred.quota_project_id.as_ref()).cloned(), Some(cred))
            }
        };
        Ok(Self {
            ts: Arc::new(DefaultTokenSource {
                inner: internal_token_source.into(),
            }),
            project_id,
            source_credentials,
        })
    }

    ///Creates source using existing credentials file
    pub async fn new_with_credentials(config: Config<'_>, credentials: Box<CredentialsFile>) -> Result<Self, Error> {
        let inner = create_token_source_from_credentials(&credentials, &config)
            .await?
            .into();
        let project_id = credentials.project_id.clone();
        let ts = Arc::new(DefaultTokenSource { inner });
        let source_credentials = Some(credentials);
        Ok(Self {
            ts,
            project_id,
            source_credentials,
        })
    }
}

impl TokenSourceProvider for DefaultTokenSourceProvider {
    fn token_source(&self) -> Arc<dyn TokenSource> {
        self.ts.clone()
    }
}

#[derive(Debug, Clone)]
pub struct DefaultTokenSource {
    inner: Arc<dyn InternalTokenSource>,
}

#[async_trait]
impl TokenSource for DefaultTokenSource {
    async fn token(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let token = self.inner.token().await?;
        Ok(format!("Bearer {0}", token.access_token))
    }
}
