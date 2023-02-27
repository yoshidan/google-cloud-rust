use std::env::var;
use std::time::Duration;

use google_cloud_gax::conn::Environment;
use google_cloud_gax::grpc::Code;
use google_cloud_gax::retry::RetrySetting;
use google_cloud_token::NopeTokenSourceProvider;

pub mod client;
pub mod database;
pub mod instance;

pub const SCOPES: [&str; 2] = [
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/spanner.admin",
];

pub struct AdminClientConfig {
    /// Runtime project
    pub environment: Environment,
}

impl Default for AdminClientConfig {
    fn default() -> Self {
        AdminClientConfig {
            environment: match var("SPANNER_EMULATOR_HOST").ok() {
                Some(v) => Environment::Emulator(v),
                None => Environment::GoogleCloud(Box::new(NopeTokenSourceProvider {})),
            },
        }
    }
}

pub fn default_retry_setting() -> RetrySetting {
    RetrySetting {
        from_millis: 50,
        max_delay: Some(Duration::from_secs(10)),
        factor: 1u64,
        take: 20,
        codes: vec![Code::Unavailable, Code::Unknown, Code::DeadlineExceeded],
    }
}
