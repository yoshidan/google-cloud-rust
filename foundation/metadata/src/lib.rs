use reqwest::header::{HeaderValue, USER_AGENT};

use std::string;
use std::time::Duration;

use tokio::net::lookup_host;
use tokio::sync::OnceCell;

pub const METADATA_IP: &str = "169.254.169.254";
pub const METADATA_HOST_ENV: &str = "GCE_METADATA_HOST";
pub const METADATA_GOOGLE_HOST: &str = "metadata.google.internal:80";
pub const METADATA_FLAVOR_KEY: &str = "Metadata-Flavor";
pub const METADATA_GOOGLE: &str = "Google";

static ON_GCE: OnceCell<bool> = OnceCell::const_new();

static PROJECT_ID: OnceCell<String> = OnceCell::const_new();

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid response code: {0}")]
    InvalidResponse(u16),
    #[error(transparent)]
    FromUTF8Error(#[from] string::FromUtf8Error),
    #[error(transparent)]
    HttpError(#[from] reqwest::Error),
}

pub async fn on_gce() -> bool {
    return match ON_GCE.get_or_try_init(test_on_gce).await {
        Ok(s) => *s,
        Err(_err) => false,
    };
}

async fn test_on_gce() -> Result<bool, Error> {
    // The user explicitly said they're on GCE, so trust them.
    if std::env::var(METADATA_HOST_ENV).is_ok() {
        return Ok(true);
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap();
    let url = format!("http://{}", METADATA_IP);

    let response = client.get(&url).send().await;
    if response.is_ok() {
        let response = response.unwrap();
        if response.status().is_success() {
            let on_gce = match response.headers().get(METADATA_FLAVOR_KEY) {
                None => false,
                Some(s) => s == METADATA_GOOGLE,
            };

            if on_gce {
                return Ok(true);
            }
        }
    }

    match lookup_host(METADATA_GOOGLE_HOST).await {
        Ok(s) => {
            for ip in s {
                if ip.ip().to_string() == METADATA_IP {
                    return Ok(true);
                }
            }
        }
        Err(_e) => return Ok(false),
    };

    Ok(false)
}

pub async fn project_id() -> String {
    return match PROJECT_ID
        .get_or_try_init(|| get_etag_with_trim("project/project-id"))
        .await
    {
        Ok(s) => s.to_string(),
        Err(_err) => "".to_string(),
    };
}

pub async fn email(service_account: &str) -> Result<String, Error> {
    get_etag_with_trim(&format!("instance/service-accounts/{}/email", service_account)).await
}

async fn get_etag_with_trim(suffix: &str) -> Result<String, Error> {
    let result = get_etag(suffix).await?;
    return Ok(result.trim().to_string());
}

async fn get_etag(suffix: &str) -> Result<String, Error> {
    let host = std::env::var(METADATA_HOST_ENV).unwrap_or_else(|_| METADATA_GOOGLE_HOST.to_string());
    let url = format!("http://{}/computeMetadata/v1/{}", host, suffix);
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap();
    let response = client
        .get(url)
        .header(METADATA_FLAVOR_KEY, HeaderValue::from_str(METADATA_GOOGLE).unwrap())
        .header(USER_AGENT, HeaderValue::from_str("gcloud-rust/0.1").unwrap())
        .send()
        .await?;

    if response.status().is_success() {
        return Ok(response.text().await?);
    }
    Err(Error::InvalidResponse(response.status().as_u16()))
}
