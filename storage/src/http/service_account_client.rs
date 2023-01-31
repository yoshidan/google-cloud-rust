use std::collections::HashMap;

use crate::http::Error;

use google_cloud_auth::token_source::TokenSource;

use base64::prelude::*;
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
            v1_endpoint: format!("{endpoint}/v1"),
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
        let json_request = format!(r#"{{"payload": "{}"}}"#, BASE64_STANDARD.encode(data));
        let token = self.ts.token().await?;
        let request = Client::default()
            .post(url)
            .body(json_request)
            .header("X-Goog-Api-Client", "rust")
            .header(reqwest::header::USER_AGENT, "google-cloud-storage")
            .header(reqwest::header::AUTHORIZATION, token.value());
        let response = request.send().await?;
        let status = response.status();
        if status.is_success() {
            let body = response.json::<HashMap<String, String>>().await?;
            match body.get("signedBlob") {
                Some(v) => Ok(BASE64_STANDARD.decode(v)?),
                None => Err(Error::Response(status.as_u16(), "no signedBlob found".to_string())),
            }
        } else {
            Err(Error::Response(status.as_u16(), response.text().await?))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::http::service_account_client::ServiceAccountClient;
    use google_cloud_auth::{create_token_source, Config};
    use serial_test::serial;
    use std::sync::Arc;

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt::try_init();
    }

    async fn client() -> ServiceAccountClient {
        let ts = create_token_source(Config {
            audience: None,
            scopes: Some(&["https://www.googleapis.com/auth/cloud-platform"]),
        })
        .await
        .unwrap();
        ServiceAccountClient::new(Arc::from(ts), "https://iamcredentials.googleapis.com")
    }

    #[tokio::test]
    #[serial]
    pub async fn sign_blob_test() {
        let client = client().await;
        let body = vec![
            71, 79, 79, 71, 52, 45, 82, 83, 65, 45, 83, 72, 65, 50, 53, 54, 10, 50, 48, 50, 50, 48, 55, 48, 57, 84, 50,
            51, 52, 56, 48, 56, 90, 10, 50, 48, 50, 50, 48, 55, 48, 57, 47, 97, 117, 116, 111, 47, 115, 116, 111, 114,
            97, 103, 101, 47, 103, 111, 111, 103, 52, 95, 114, 101, 113, 117, 101, 115, 116, 10, 98, 101, 97, 48, 48,
            49, 100, 98, 48, 50, 97, 56, 98, 55, 101, 101, 54, 50, 102, 50, 54, 53, 99, 101, 50, 52, 54, 53, 51, 49,
            97, 98, 50, 54, 101, 102, 49, 97, 48, 97, 99, 100, 102, 102, 55, 99, 54, 55, 49, 100, 101, 56, 49, 100, 56,
            56, 98, 50, 56, 101, 55, 48, 98, 101,
        ];
        let data = client
            .sign_blob(
                "projects/-/serviceAccounts/rust-storage-test@atl-dev1.iam.gserviceaccount.com",
                &body,
            )
            .await
            .unwrap();
        assert_eq!(256, data.len());
    }
}
