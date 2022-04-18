pub mod credentials;
pub mod error;
mod misc;
pub mod token;
pub mod token_source;

use crate::credentials::{CredentialSource, CredentialsFile};
use crate::error::Error;
use crate::misc::EMPTY;
use crate::token_source::authorized_user_token_source::UserAccountTokenSource;
use crate::token_source::compute_token_source::ComputeTokenSource;
use crate::token_source::reuse_token_source::ReuseTokenSource;
use crate::token_source::service_account_token_source::OAuth2ServiceAccountTokenSource;
use crate::token_source::service_account_token_source::ServiceAccountTokenSource;
use crate::token_source::TokenSource;
use google_cloud_metadata::{on_gce, project_id};
use std::sync::atomic::Ordering::AcqRel;

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

pub struct Credentials {
    pub from_metadata_server: bool,
    pub project_id: Option<String>,
    pub file: Option<CredentialsFile>,
}

pub async fn get_credentials() -> Result<Credentials, error::Error> {
    let credentials = credentials::CredentialsFile::new().await;
    return match credentials {
        Ok(cred) => {
            let project_id = cred.project_id.clone();
            Ok(Credentials {
                project_id,
                file: Some(cred),
                from_metadata_server: false,
            })
        }
        Err(e) => {
            // use metadata server on gce
            if on_gce().await {
                let project_id = project_id().await;
                Ok(Credentials {
                    project_id: Some(project_id),
                    file: None,
                    from_metadata_server: true,
                })
            } else {
                Err(e)
            }
        }
    };
}

pub async fn create_token_source_from_credentials(
    credentials: &Credentials,
    config: Config<'_>,
) -> Result<Box<dyn TokenSource>, error::Error> {
    let ts = if credentials.from_metadata_server {
        Box::new(ComputeTokenSource::new(&config.scopes_to_string(","))?)
    } else {
        match &credentials.file {
            Some(file) => credentials_from_json_with_params(file, &config)?,
            None => return Err(Error::NoCredentialsFileFound),
        }
    };
    let token = ts.token().await?;
    Ok(Box::new(ReuseTokenSource::new(ts, token)))
}

pub async fn create_token_source(config: Config<'_>) -> Result<Box<dyn TokenSource>, error::Error> {
    let credentials = credentials::CredentialsFile::new().await;

    return match credentials {
        Ok(cred) => {
            let ts = credentials_from_json_with_params(&cred, &config)?;
            let token = ts.token().await?;
            Ok(Box::new(ReuseTokenSource::new(ts, token)))
        }
        Err(e) => {
            // use metadata server on gce
            if on_gce().await {
                let ts = ComputeTokenSource::new(&config.scopes_to_string(","))?;
                let token = ts.token().await?;
                Ok(Box::new(ReuseTokenSource::new(Box::new(ts), token)))
            } else {
                Err(e)
            }
        }
    };
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
