use hyper;
use hyper::client::HttpConnector;
use hyper::http::{Method, Request};
use hyper::Client;
use std::time::Duration;
use thiserror;
use tokio::net::lookup_host;
use tokio::sync::OnceCell;

pub const METADATA_IP: &str = "169.254.169.254";
pub const METADATA_HOST_ENV: &str = "GCE_METADATA_HOST";
pub const METADATA_GOOGLE_HOST: &str = "metadata.gen.internal:80";
pub const METADATA_FLAVOR_KEY: &str = "Metadata-Flavor";
pub const METADATA_GOOGLE: &str = "Google";

static ON_GCE: OnceCell<bool> = OnceCell::const_new();

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
    Http(#[from] hyper::http::Error),
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
    let request = Request::builder()
        .method(Method::GET)
        .uri(&url)
        .body(body)?;

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

    return Ok(false);
}
