use crate::admin::conn_pool::AdminConnectionManager;
use crate::apiv1::conn_pool::Error;
use google_cloud_auth::token_source::TokenSource;
use google_cloud_auth::{create_token_source, Config};
use google_cloud_googleapis::spanner::admin::database::v1::database_admin_client::DatabaseAdminClient as InternalDatabaseAdminClient;
use std::sync::Arc;
use tonic::transport::Channel;

#[derive(Clone)]
pub struct DatabaseAdminClient {
    inner: InternalDatabaseAdminClient<Channel>,
    token_source: Option<Arc<dyn TokenSource>>,
}

impl DatabaseAdminClient {
    pub async fn new() -> Result<Self, Error> {
        let emulator_host = match std::env::var("SPANNER_EMULATOR_HOST") {
            Ok(s) => Some(s),
            Err(_) => None,
        };
        let conn_pool = AdminConnectionManager::new(1, emulator_host).await?;
        let (conn, token_source) = conn_pool.conn();
        Ok(DatabaseAdminClient {
            inner: InternalDatabaseAdminClient::new(conn),
            token_source,
        })
    }
}
