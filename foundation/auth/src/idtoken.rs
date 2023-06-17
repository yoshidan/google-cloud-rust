use std::collections::HashMap;

use crate::{
    credentials::CredentialsFile,
    error,
    project::{project, Project, SERVICE_ACCOUNT_KEY},
    token_source::{
        compute_identity_source::ComputeIdentitySource, reuse_token_source::ReuseTokenSource,
        service_account_token_source::OAuth2ServiceAccountTokenSource, TokenSource,
    },
};

#[derive(Clone, Default)]
pub struct IdTokenSourceConfig {
    credentials: Option<CredentialsFile>,
    custom_claims: HashMap<String, serde_json::Value>,
}

impl std::fmt::Debug for IdTokenSourceConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IdTokenConfig")
            .field("custom_claims", &self.custom_claims)
            .finish_non_exhaustive()
    }
}

impl IdTokenSourceConfig {
    pub fn with_credentials(mut self, creds: CredentialsFile) -> Self {
        self.credentials = creds.into();
        self
    }

    pub fn with_custom_claims(mut self, custom_claims: HashMap<String, serde_json::Value>) -> Self {
        self.custom_claims = custom_claims;
        self
    }

    pub async fn build(self, audience: &str) -> Result<Box<dyn TokenSource>, error::Error> {
        create_id_token_source(self, audience).await
    }
}

pub async fn create_id_token_source(
    config: IdTokenSourceConfig,
    audience: &str,
) -> Result<Box<dyn TokenSource>, error::Error> {
    if audience.is_empty() {
        return Err(error::Error::ScopeOrAudienceRequired);
    }

    if let Some(credentials) = &config.credentials {
        return id_token_source_from_credentials(&config.custom_claims, credentials, audience).await;
    }

    match project().await? {
        Project::FromFile(credentials) => {
            id_token_source_from_credentials(&config.custom_claims, &credentials, audience).await
        }
        Project::FromMetadataServer(_) => {
            let ts = ComputeIdentitySource::new(audience)?;
            let token = ts.token().await?;
            Ok(Box::new(ReuseTokenSource::new(Box::new(ts), token)))
        }
    }
}

async fn id_token_source_from_credentials(
    custom_claims: &HashMap<String, serde_json::Value>,
    credentials: &CredentialsFile,
    audience: &str,
) -> Result<Box<dyn TokenSource>, error::Error> {
    let ts = match credentials.tp.as_str() {
        SERVICE_ACCOUNT_KEY => {
            let mut claims = custom_claims.clone();
            claims.insert("target_audience".into(), audience.into());

            let source = OAuth2ServiceAccountTokenSource::new(credentials, "", None)?
                .with_use_id_token()
                .with_private_claims(claims);

            Ok(Box::new(source))
        }
        // TODO: support impersonation and external account
        _ => Err(error::Error::UnsupportedAccountType(credentials.tp.to_string())),
    }?;
    let token = ts.token().await?;
    Ok(Box::new(ReuseTokenSource::new(ts, token)))
}
