use google_cloud_metadata::on_gce;

use crate::credentials::CredentialsFile;
use crate::misc::EMPTY;
use crate::token_source::authorized_user_token_source::UserAccountTokenSource;
use crate::token_source::compute_token_source::ComputeTokenSource;
use crate::token_source::external_account_source::ExternalAccountTokenSource;
use crate::token_source::reuse_token_source::ReuseTokenSource;
use crate::token_source::service_account_token_source::OAuth2ServiceAccountTokenSource;
use crate::token_source::service_account_token_source::ServiceAccountTokenSource;
use crate::token_source::TokenSource;
use crate::{credentials, error};

pub(crate) const SERVICE_ACCOUNT_KEY: &str = "service_account";
const USER_CREDENTIALS_KEY: &str = "authorized_user";
const EXTERNAL_ACCOUNT_KEY: &str = "external_account";

#[derive(Debug, Clone, Default)]
pub struct Config<'a> {
    pub audience: Option<&'a str>,
    pub scopes: Option<&'a [&'a str]>,
    pub sub: Option<&'a str>,
}

impl Config<'_> {
    pub fn scopes_to_string(&self, sep: &str) -> String {
        match self.scopes {
            Some(s) => s.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(sep),
            None => EMPTY.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct ProjectInfo {
    pub project_id: Option<String>,
}

#[derive(Clone)]
pub enum Project {
    FromFile(Box<CredentialsFile>),
    FromMetadataServer(ProjectInfo),
}

// Possible sensitive info in debug messages
impl std::fmt::Debug for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Project::FromFile(_) => write!(f, "Project::FromFile"),
            Project::FromMetadataServer(_) => write!(f, "Project::FromMetadataServer"),
        }
    }
}

impl Project {
    pub fn project_id(&self) -> Option<&String> {
        match self {
            Self::FromFile(file) => file.project_id.as_ref(),
            Self::FromMetadataServer(info) => info.project_id.as_ref(),
        }
    }
}

/// project() returns the project credentials or project info from metadata server.
pub async fn project() -> Result<Project, error::Error> {
    let credentials = credentials::CredentialsFile::new().await;
    match credentials {
        Ok(credentials) => Ok(Project::FromFile(Box::new(credentials))),
        Err(e) => {
            if on_gce().await {
                let project_id = google_cloud_metadata::project_id().await;
                Ok(Project::FromMetadataServer(ProjectInfo {
                    project_id: if project_id.is_empty() { None } else { Some(project_id) },
                }))
            } else {
                Err(e)
            }
        }
    }
}

/// Creates token source using provided credentials file
pub async fn create_token_source_from_credentials(
    credentials: &CredentialsFile,
    config: &Config<'_>,
) -> Result<Box<dyn TokenSource>, error::Error> {
    let ts = credentials_from_json_with_params(credentials, config).await?;
    let token = ts.token().await?;
    Ok(Box::new(ReuseTokenSource::new(ts, token)))
}

/// create_token_source_from_project creates the token source.
pub async fn create_token_source_from_project(
    project: &Project,
    config: Config<'_>,
) -> Result<Box<dyn TokenSource>, error::Error> {
    match project {
        Project::FromFile(file) => create_token_source_from_credentials(file, &config).await,
        Project::FromMetadataServer(_) => {
            let ts = ComputeTokenSource::new(&config.scopes_to_string(","))?;
            let token = ts.token().await?;
            Ok(Box::new(ReuseTokenSource::new(Box::new(ts), token)))
        }
    }
}

/// [Deprecated] create_token_source creates the token source
/// use `google_cloud_auth:bridge::DefaultTokenSourceProvider` instead.
pub async fn create_token_source(config: Config<'_>) -> Result<Box<dyn TokenSource>, error::Error> {
    let project = project().await?;
    create_token_source_from_project(&project, config).await
}

async fn credentials_from_json_with_params(
    credentials: &CredentialsFile,
    config: &Config<'_>,
) -> Result<Box<dyn TokenSource>, error::Error> {
    match credentials.tp.as_str() {
        SERVICE_ACCOUNT_KEY => {
            match config.audience {
                None => {
                    if config.scopes.is_none() {
                        return Err(error::Error::ScopeOrAudienceRequired);
                    }

                    // use Standard OAuth 2.0 Flow
                    let source = OAuth2ServiceAccountTokenSource::new(
                        credentials,
                        config.scopes_to_string(" ").as_str(),
                        config.sub,
                    )?;
                    Ok(Box::new(source))
                }
                Some(audience) => {
                    // use self-signed JWT.
                    let source = ServiceAccountTokenSource::new(credentials, audience)?;
                    Ok(Box::new(source))
                }
            }
        }
        USER_CREDENTIALS_KEY => Ok(Box::new(UserAccountTokenSource::new(credentials)?)),
        //TODO support GDC https://console.developers.google.com,
        #[cfg(feature = "external-account")]
        EXTERNAL_ACCOUNT_KEY => Ok(Box::new(
            ExternalAccountTokenSource::new(config.scopes_to_string(" ").as_str(), credentials).await?,
        )),
        _ => Err(error::Error::UnsupportedAccountType(credentials.tp.to_string())),
    }
}
