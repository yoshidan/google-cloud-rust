use crate::{AUDIENCE, SPANNER};
use google_cloud_gax::conn::{Channel, ConnectionManager, Error};
use google_cloud_gax::grpc::Code;
use google_cloud_gax::retry::RetrySetting;
use google_cloud_longrunning::autogen::operations_client::OperationsClient;
use std::time::Duration;

pub mod database;
pub mod instance;

const SCOPES: [&str; 2] = [
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/spanner.admin",
];

pub fn default_retry_setting() -> RetrySetting {
    RetrySetting {
        from_millis: 50,
        max_delay: Some(Duration::from_secs(10)),
        factor: 1u64,
        take: 20,
        codes: vec![Code::Unavailable, Code::Unknown, Code::DeadlineExceeded],
    }
}

pub async fn default_internal_client() -> Result<(Channel, OperationsClient), Error> {
    let emulator_host = match std::env::var("SPANNER_EMULATOR_HOST") {
        Ok(s) => Some(s),
        Err(_) => None,
    };
    let conn_pool = ConnectionManager::new(1, SPANNER, AUDIENCE, Some(&SCOPES), emulator_host).await?;
    let conn = conn_pool.conn();
    let lro_client = OperationsClient::new(conn).await?;
    Ok((conn_pool.conn(), lro_client))
}
