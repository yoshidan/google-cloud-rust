use hyper::client::HttpConnector;
use hyper::header::USER_AGENT;
use hyper::http::{HeaderValue, Method, Request};
use hyper::{Client, StatusCode};
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

pub fn default_http_connector() -> HttpConnector {
    let mut connector = HttpConnector::new();
    connector.enforce_http(false);
    connector.set_connect_timeout(Some(Duration::from_secs(2)));
    connector.set_keepalive(Some(Duration::from_secs(30)));
    connector
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Hyper(#[from] hyper::Error),
    #[error(transparent)]
    Http(#[from] hyper::http::Error),
    #[error("invalid response code: {0}")]
    InvalidResponse(StatusCode),
    #[error(transparent)]
    FromUTF8Error(#[from] string::FromUtf8Error),
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

    let url = format!("http://{}", METADATA_IP);
    let body = hyper::Body::empty();
    let request = Request::builder().method(Method::GET).uri(&url).body(body)?;

    let client = Client::builder().build(default_http_connector());
    let response = client.request(request).await;

    if response.is_ok() {
        let on_gce = match response.unwrap().headers().get(METADATA_FLAVOR_KEY) {
            None => false,
            Some(s) => s == METADATA_GOOGLE,
        };

        if on_gce {
            return Ok(true);
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
    let host = match std::env::var(METADATA_HOST_ENV) {
        Ok(host) => host,
        Err(_e) => METADATA_IP.to_string(),
    };

    let url = format!("http://{}//computeMetadata/v1/{}", host, suffix);
    let body = hyper::Body::empty();
    let mut request = Request::builder().method(Method::GET).uri(&url).body(body)?;
    request
        .headers_mut()
        .insert(METADATA_FLAVOR_KEY, HeaderValue::from_str(METADATA_GOOGLE).unwrap());
    request
        .headers_mut()
        .insert(USER_AGENT, HeaderValue::from_str("gcloud-rust/0.1").unwrap());

    let client = Client::builder().build(default_http_connector());
    let maybe_response = client.request(request).await;

    match maybe_response {
        Ok(response) => {
            if response.status() == StatusCode::OK {
                let bytes = hyper::body::to_bytes(response.into_body()).await?;
                return String::from_utf8(bytes.to_vec()).map_err(|e| e.into());
            }
            return Err(Error::InvalidResponse(response.status()));
        }
        Err(e) => Err(e.into()),
    }
}
