use std::sync::Arc;

use google_cloud_token::TokenSource;

use crate::http::{check_response_status, Error};

pub struct ServiceAccountClient {
    ts: Arc<dyn TokenSource>,
    v1_endpoint: String,
    http: reqwest::Client,
}

impl ServiceAccountClient {
    pub(crate) fn new(ts: Arc<dyn TokenSource>, endpoint: &str, http: reqwest::Client) -> Self {
        Self {
            ts,
            v1_endpoint: format!("{endpoint}/v1"),
            http,
        }
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn sign_blob(&self, name: &str, payload: &[u8]) -> Result<Vec<u8>, Error> {
        let url = format!("{}/{}:signBlob", self.v1_endpoint, name);
        let request = SignBlobRequest { payload };
        let token = self.ts.token().await.map_err(Error::TokenSource)?;
        let request = self
            .http
            .post(url)
            .json(&request)
            .header("X-Goog-Api-Client", "rust")
            .header(reqwest::header::USER_AGENT, "google-cloud-storage")
            .header(reqwest::header::AUTHORIZATION, token);
        let response = request.send().await?;
        let response = check_response_status(response).await?;
        Ok(response.json::<SignBlobResponse>().await?.signed_blob)
    }
}

#[derive(serde::Serialize)]
struct SignBlobRequest<'a> {
    #[serde(with = "super::base64")]
    payload: &'a [u8],
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct SignBlobResponse {
    #[serde(with = "super::base64")]
    signed_blob: Vec<u8>,
}

#[cfg(test)]
mod test {
    use reqwest::Client;
    use serial_test::serial;

    use google_cloud_auth::project::Config;
    use google_cloud_auth::token::DefaultTokenSourceProvider;
    use google_cloud_token::TokenSourceProvider;

    use crate::http::service_account_client::ServiceAccountClient;

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt::try_init();
    }

    async fn client() -> ServiceAccountClient {
        let ts = DefaultTokenSourceProvider::new(Config {
            audience: None,
            scopes: Some(&["https://www.googleapis.com/auth/cloud-platform"]),
        })
        .await
        .unwrap()
        .token_source();
        ServiceAccountClient::new(ts, "https://iamcredentials.googleapis.com", Client::default())
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
