use std::collections::HashMap;

use crate::{
    credentials::CredentialsFile,
    error,
    project::{create_token_source_from_credentials, project, Config, Project, SERVICE_ACCOUNT_KEY},
    token_source::{
        compute_identity_source::ComputeIdentitySource, reuse_token_source::ReuseTokenSource,
        service_account_token_source::OAuth2ServiceAccountTokenSource, TokenSource,
    },
};

#[derive(Clone)]
pub struct IdTokenConfig {
    credentials: Option<CredentialsFile>,
    custom_claims: Option<HashMap<String, serde_json::Value>>,
}

impl std::fmt::Debug for IdTokenConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IdTokenConfig")
            .field("custom_claims", &self.custom_claims)
            .finish_non_exhaustive()
    }
}

impl IdTokenConfig {
    pub fn with_credentials(mut self, creds: CredentialsFile) -> Self {
        self.credentials.replace(creds);
        self
    }
}

pub async fn create_id_token_source(
    config: IdTokenConfig,
    audience: &str,
) -> Result<Box<dyn TokenSource>, error::Error> {
    if audience.is_empty() {
        return Err(error::Error::ScopeOrAudienceRequired);
    }

    if let Some(credentials) = config.credentials {
        return create_token_source_from_credentials(
            &credentials,
            &Config {
                audience: audience.into(),
                ..Default::default()
            },
        )
        .await;
    }

    match project().await? {
        Project::FromFile(credentials) => {
            let ts = id_token_source_from_credentials(config, &credentials, audience).await?;
            let token = ts.token().await?;
            Ok(Box::new(ReuseTokenSource::new(ts, token)))
        }
        Project::FromMetadataServer(_) => {
            let ts = ComputeIdentitySource::new(audience)?;
            let token = ts.token().await?;
            Ok(Box::new(ReuseTokenSource::new(Box::new(ts), token)))
        }
    }
}

async fn id_token_source_from_credentials(
    config: IdTokenConfig,
    credentials: &CredentialsFile,
    audience: &str,
) -> Result<Box<dyn TokenSource>, error::Error> {
    match credentials.tp.as_str() {
        SERVICE_ACCOUNT_KEY => {
            let mut claims = config.custom_claims.unwrap_or_default();
            claims.insert("target_audience".into(), audience.into());

            let source = OAuth2ServiceAccountTokenSource::new(credentials, "", None)?
                .with_use_id_token()
                .with_private_claims(claims);

            Ok(Box::new(source))
        }
        // TODO: support impersonation and external account
        _ => Err(error::Error::UnsupportedAccountType(credentials.tp.to_string())),
    }
}
