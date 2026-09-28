use google_cloud_gax::conn::{ConnectionManager as GRPCConnectionManager, ConnectionOptions, Environment, Error};
use google_cloud_googleapis::spanner::v1::spanner_client::SpannerClient;

use crate::apiv1::spanner_client::Client;

pub const AUDIENCE: &str = "https://spanner.googleapis.com/";
pub const SPANNER: &str = "spanner.googleapis.com";
pub const SCOPES: [&str; 2] = [
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/spanner.data",
];

pub struct ConnectionManager {
    inner: GRPCConnectionManager,
    enable_gzip: bool,
}

impl ConnectionManager {
    pub async fn new(
        pool_size: usize,
        environment: &Environment,
        domain: &str,
        conn_options: &ConnectionOptions,
    ) -> Result<Self, Error> {
        // Support Private Service Connect (PSC) endpoints for Spanner.
        //
        // When `domain` is a full HTTPS URL (e.g. "https://spanner-nonprod.p.googleapis.com")
        // we use it as both the TLS SNI hostname and the gRPC connection target, mirroring
        // the Java SDK's `SpannerOptions.setHost("https://...")` behaviour.
        //
        // When `domain` is a plain hostname (default: "spanner.googleapis.com") the existing
        // behaviour is preserved: the public AUDIENCE constant is used as the target.
        let (sni_domain, audience): (String, &str) = if domain.starts_with("https://") {
            let host = domain.trim_start_matches("https://").trim_end_matches('/');
            (host.to_string(), domain)
        } else {
            (domain.to_string(), AUDIENCE)
        };

        Ok(ConnectionManager {
            inner: GRPCConnectionManager::new(pool_size, sni_domain, audience, environment, conn_options).await?,
            enable_gzip: false,
        })
    }

    pub fn num(&self) -> usize {
        self.inner.num()
    }

    pub(crate) fn with_gzip(mut self, enabled: bool) -> Self {
        self.enable_gzip = enabled;
        self
    }

    pub fn conn(&self) -> Client {
        let conn = self.inner.conn();
        Client::new(SpannerClient::new(conn)).with_gzip(self.enable_gzip)
    }
}
