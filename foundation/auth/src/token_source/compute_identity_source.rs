use async_trait::async_trait;
use jsonwebtoken::Validation;
use serde::Deserialize;
use time::OffsetDateTime;
use urlencoding::encode;

use google_cloud_metadata::{METADATA_FLAVOR_KEY, METADATA_GOOGLE, METADATA_HOST_ENV, METADATA_IP};

use crate::error::Error;
use crate::token::Token;
use crate::token_source::{default_http_client, TokenSource};

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
    decoding_key: jsonwebtoken::DecodingKey,
    validation: jsonwebtoken::Validation,
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

        // Only used to extract the expiry without checking the signature.
        let mut validation = Validation::default();
        validation.insecure_disable_signature_validation();
        validation.set_audience(&[audience]);
        let decoding_key = jsonwebtoken::DecodingKey::from_secret(b"");

        Ok(ComputeIdentitySource {
            token_url: format!(
                "http://{}/computeMetadata/v1/instance/service-accounts/default/identity?audience={}&format=full",
                host,
                encode(audience)
            ),
            client: default_http_client(),
            decoding_key,
            validation,
        })
    }
}

#[derive(Deserialize)]
struct ExpClaim {
    exp: i64,
}

#[async_trait]
impl TokenSource for ComputeIdentitySource {
    async fn token(&self) -> Result<Token, Error> {
        let jwt = self
            .client
            .get(self.token_url.to_string())
            .header(METADATA_FLAVOR_KEY, METADATA_GOOGLE)
            .send()
            .await?
            .text()
            .await?;

        let exp = jsonwebtoken::decode::<ExpClaim>(&jwt, &self.decoding_key, &self.validation)?
            .claims
            .exp;

        Ok(Token {
            access_token: jwt,
            token_type: "Bearer".into(),
            expiry: OffsetDateTime::from_unix_timestamp(exp).ok(),
        })
    }
}
