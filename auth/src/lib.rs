pub mod credentials;
pub mod error;
pub mod token;
pub mod token_source;

use crate::credentials::CredentialsFile;
use crate::token::TokenSource;
use crate::token_source::authorized_user_token_source::UserAccountTokenSource;
use crate::token_source::compute_token_source::{on_gce, ComputeTokenSource};
use crate::token_source::reuse_token_source::ReuseTokenSource;
use crate::token_source::service_account_token_source::OAuth2ServiceAccountTokenSource;
use crate::token_source::service_account_token_source::ServiceAccountTokenSource;

const SERVICE_ACCOUNT_KEY: &str = "service_account";
const USER_CREDENTIALS_KEY: &str = "authorized_user";

pub struct Config<'a> {
    pub audience: Option<&'a str>,
    pub scopes: Option<&'a [&'a str]>,
}

impl Config<'_> {
    pub fn scopes_to_string(&self, sep: &str) -> String {
        self.scopes
            .unwrap()
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(sep)
    }
}

pub async fn create_token_source(config: Config<'_>) -> Result<Box<dyn TokenSource>, error::Error> {
    let credentials = credentials::CredentialsFile::new().await;

    return match credentials {
        Ok(s) => {
            let ts = credentials_from_json_with_params(s, &config)?;
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
    credentials: CredentialsFile,
    config: &Config,
) -> Result<Box<dyn TokenSource>, error::Error> {
    match credentials.tp.as_str() {
        SERVICE_ACCOUNT_KEY => {
            if config.audience.is_none() {
                if config.scopes.is_none() {
                    return Err(error::Error::StringError(
                        "scopes is required if the audience is none".to_string(),
                    ));
                }

                // use Standard OAuth 2.0 Flow
                return match OAuth2ServiceAccountTokenSource::new(
                    &credentials,
                    config.scopes_to_string(" ").as_str(),
                ) {
                    Ok(s) => Ok(Box::new(s)),
                    Err(e) => return Err(e),
                };
            }
            // use self-signed JWT.
            match ServiceAccountTokenSource::new(&credentials, config.audience.unwrap()) {
                Ok(s) => Ok(Box::new(s)),
                Err(e) => return Err(e),
            }
        }
        USER_CREDENTIALS_KEY => match UserAccountTokenSource::new(&credentials) {
            Ok(s) => Ok(Box::new(s)),
            Err(e) => return Err(e),
        },
        //TODO support GDC https://console.developers.google.com,
        //TODO support external account
        _ => Err(error::Error::StringError(format!(
            "unsupported account type {}",
            credentials.tp
        ))),
    }
}
