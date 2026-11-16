use async_trait::async_trait;
use serde::Deserialize;
use time::OffsetDateTime;
use urlencoding::encode;

use google_cloud_metadata::{METADATA_FLAVOR_KEY, METADATA_GOOGLE, METADATA_HOST_ENV, METADATA_IP};

use crate::error::Error;
use crate::token::Token;
use crate::token_source::{default_http_client, GoogleCloudTokenSource};

/// Fetches a JWT token from the metadata server.
/// using the `identity` endpoint.
///
/// This token source is useful for service-to-service authentication, notably on Cloud Run.
///
/// See <https://cloud.google.com/run/docs/authenticating/service-to-service#use_the_metadata_server>
#[derive(Clone)]
pub struct ComputeIdentitySource {
    token_url: String,
    client: reqwest::Client,
}

impl std::fmt::Debug for ComputeIdentitySource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComputeIdentitySource")
            .field("token_url", &self.token_url)
            .finish_non_exhaustive()
    }
}

impl ComputeIdentitySource {
    pub(crate) fn new(audience: &str) -> Result<ComputeIdentitySource, Error> {
        let host = match std::env::var(METADATA_HOST_ENV) {
            Ok(s) => s,
            Err(_e) => METADATA_IP.to_string(),
        };

        Ok(ComputeIdentitySource {
            token_url: format!(
                "http://{}/computeMetadata/v1/instance/service-accounts/default/identity?audience={}&format=full",
                host,
                encode(audience)
            ),
            client: default_http_client(),
        })
    }
}

#[derive(Deserialize)]
struct ExpClaim {
    exp: i64,
}

#[async_trait]
impl GoogleCloudTokenSource for ComputeIdentitySource {
    async fn token(&self) -> Result<Token, Error> {
        let jwt = self
            .client
            .get(self.token_url.to_string())
            .header(METADATA_FLAVOR_KEY, METADATA_GOOGLE)
            .send()
            .await?
            .text()
            .await?;

        // Only used to extract the expiry without checking the signature.
        let token = jsonwebtoken::dangerous::insecure_decode::<ExpClaim>(jwt.as_bytes())?;
        Ok(Token {
            access_token: jwt,
            token_type: "Bearer".into(),
            expiry: OffsetDateTime::from_unix_timestamp(token.claims.exp).ok(),
        })
    }
}
