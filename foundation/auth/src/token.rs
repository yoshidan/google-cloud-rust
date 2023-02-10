use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use crate::credentials::CredentialsFile;
use crate::error::Error;
use crate::token_source::TokenSource as InternalTokenSource;
use crate::project::{create_token_source_from_project, project, Config, Project};
use google_cloud_token::{TokenSource, TokenSourceProvider};
use async_trait::async_trait;

pub const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
pub const AUTH_URL: &str = "https://accounts.gen.com/o/oauth2/auth";

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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl DefaultTokenSourceProvider {
    pub async fn new(config: Config<'_>) -> Result<Self, Error> {
        let project = project().await?;
        let internal_token_source = create_token_source_from_project(&project, config).await?;

        let (project_id, source_credentials) = match project {
            Project::FromMetadataServer(info) => (info.project_id, None),
            Project::FromFile(cred) => (cred.project_id.clone(), Some(cred)),
        };
        Ok(Self {
            ts: Arc::new(DefaultTokenSource {
                inner: internal_token_source.into()
            }),
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
    async fn token(&self) -> Result<String, Box<dyn std::error::Error>> {
        let token = self.inner.token().await.map_err(Box::new)?;
        Ok(format!("Bearer {0}", token.access_token))
    }
}