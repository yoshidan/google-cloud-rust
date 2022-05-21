pub mod credentials;
pub mod error;
mod misc;
pub mod token;
pub mod token_source;

use crate::credentials::CredentialsFile;
use crate::misc::{UnwrapOrEmpty, EMPTY};
use crate::token_source::authorized_user_token_source::UserAccountTokenSource;
use crate::token_source::compute_token_source::ComputeTokenSource;
use crate::token_source::reuse_token_source::ReuseTokenSource;
use crate::token_source::service_account_token_source::OAuth2ServiceAccountTokenSource;
use crate::token_source::service_account_token_source::ServiceAccountTokenSource;
use crate::token_source::TokenSource;
use crate::Project::FromFile;
use google_cloud_metadata::on_gce;

const SERVICE_ACCOUNT_KEY: &str = "service_account";
const USER_CREDENTIALS_KEY: &str = "authorized_user";

pub struct Config<'a> {
    pub audience: Option<&'a str>,
    pub scopes: Option<&'a [&'a str]>,
}

impl Config<'_> {
    pub fn scopes_to_string(&self, sep: &str) -> String {
        match self.scopes {
            Some(s) => s.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(sep),
            None => EMPTY.to_string(),
        }
    }
}

pub struct ProjectInfo {
    pub project_id: Option<String>,
}

pub enum Project {
    FromFile(CredentialsFile),
    FromMetadataServer(ProjectInfo),
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
        Ok(credentials) => Ok(Project::FromFile(credentials)),
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

/// create_token_source_from_project creates the token source.
pub async fn create_token_source_from_project(
    project: &Project,
    config: Config<'_>,
) -> Result<Box<dyn TokenSource>, error::Error> {
    match project {
        Project::FromFile(file) => {
            let ts = credentials_from_json_with_params(file, &config)?;
            let token = ts.token().await?;
            Ok(Box::new(ReuseTokenSource::new(ts, token)))
        }
        Project::FromMetadataServer(_) => {
            let ts = ComputeTokenSource::new(&config.scopes_to_string(","))?;
            let token = ts.token().await?;
            Ok(Box::new(ReuseTokenSource::new(Box::new(ts), token)))
        }
    }
}

/// create_token_source creates the token source
pub async fn create_token_source(config: Config<'_>) -> Result<Box<dyn TokenSource>, error::Error> {
    let project = project().await?;
    create_token_source_from_project(&project, config).await
}

fn credentials_from_json_with_params(
    credentials: &CredentialsFile,
    config: &Config,
) -> Result<Box<dyn TokenSource>, error::Error> {
    match credentials.tp.as_str() {
        SERVICE_ACCOUNT_KEY => {
            match config.audience {
                None => {
                    if config.scopes.is_none() {
                        return Err(error::Error::ScopeOrAudienceRequired);
                    }

                    // use Standard OAuth 2.0 Flow
                    let source =
                        OAuth2ServiceAccountTokenSource::new(credentials, config.scopes_to_string(" ").as_str())?;
                    Ok(Box::new(source))
                }
                Some(audience) => {
                    // use self-signed JWT.
                    let source = ServiceAccountTokenSource::new(credentials, audience)?;
                    Ok(Box::new(source))
                }
            }
        }
        USER_CREDENTIALS_KEY => Ok(Box::new(UserAccountTokenSource::new(&credentials)?)),
        //TODO support GDC https://console.developers.google.com,
        //TODO support external account
        _ => Err(error::Error::UnsupportedAccountType(credentials.tp.to_string())),
    }
}
