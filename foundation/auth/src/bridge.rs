use crate::credentials::CredentialsFile;
use crate::error::Error;
use crate::project::{create_token_source, create_token_source_from_project, project, Config, Project};
use crate::{create_token_source, Config};
use google_cloud_token::{Error as TokenError, TokenSource, TokenSourceProvider};

#[derive(Debug)]
pub struct DefaultTokenSourceProvider {
    token_source: Box<dyn TokenSource>,
    pub project_id: Option<String>,
    pub source_credentials: Option<Box<CredentialsFile>>,
}

impl DefaultTokenSourceProvider {
    pub async fn new(config: Config<'_>) -> Result<Self, Error> {
        let project = project().await?;
        let (project_id, source_credentials) = match project {
            Project::FromMetadataServer(info) => (info.project_id, None),
            Project::FromFile(cred) => (cred.project_id.clone(), Some(cred)),
        };
        let token_source = create_token_source_from_project(&project, config).await?;
        Ok(Self {
            token_source,
            project_id,
            source_credentials,
        })
    }
}

impl TokenSourceProvider for DefaultTokenSourceProvider {
    fn token_source(&self) -> &dyn TokenSource {
        self.token_source.as_ref()
    }
}

#[derive(Debug)]
pub struct DefaultTokenSource {
    inner: Box<dyn TokenSource>,
}

impl TokenSource for DefaultTokenSource {
    async fn token(&self) -> Result<String, Box<dyn TokenError>> {
        self.inner.token().await.map_err(Box::new)
    }
}

impl From<dyn InternalTokenSource> for DefaultTokenSource {
    fn from(value: Box<dyn InternalTokenSource>) -> Self {
        Self { inner: value }
    }
}
