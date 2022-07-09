use std::collections::HashMap;

use crate::http::Error;

use google_cloud_auth::token_source::TokenSource;

use reqwest::Client;

use std::sync::Arc;

pub struct ServiceAccountClient {
    ts: Arc<dyn TokenSource>,
    v1_endpoint: String,
}

impl ServiceAccountClient {
    pub(crate) fn new(ts: Arc<dyn TokenSource>, endpoint: &str) -> Self {
        Self {
            ts,
            v1_endpoint: format!("{}/v1", endpoint),
        }
    }

    #[cfg(feature = "trace")]
    #[tracing::instrument(skip_all)]
    pub async fn sign_blob(&self, name: &str, data: &[u8]) -> Result<Vec<u8>, Error> {
        self._sign_blob(name, data).await
    }

    #[cfg(not(feature = "trace"))]
    pub async fn sign_blob(&self, name: &str, data: &[u8]) -> Result<Vec<u8>, Error> {
        self._sign_blob(name, data).await
    }

    async fn _sign_blob(&self, name: &str, data: &[u8]) -> Result<Vec<u8>, Error> {
        let url = format!("{}/{}:signBlob", self.v1_endpoint, name);
        let payload = ("payload", base64::encode(data));
        let token = self.ts.token().await?;
        let request = Client::default()
            .post(url)
            .json(&payload)
            .header("X-Goog-Api-Client", "rust")
            .header(reqwest::header::USER_AGENT, "google-cloud-storage")
            .header(reqwest::header::AUTHORIZATION, token.value());
        let response = request.send().await?;
        let status = response.status();
        if status.is_success() {
            let body = response.json::<HashMap<String, String>>().await?;
            match body.get("signedBlob") {
                Some(v) => Ok(base64::decode(v)?),
                None => Err(Error::Response(status.as_u16(), "no signedBlob found".to_string())),
            }
        } else {
            Err(Error::Response(status.as_u16(), response.text().await?))
        }
    }
}
