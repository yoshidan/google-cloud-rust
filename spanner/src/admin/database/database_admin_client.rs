use google_cloud_googleapis::spanner::admin::database::v1::database_admin_client::DatabaseAdminClient as InternalDatabaseAdminClient;
use tonic::transport::Channel;
use google_cloud_auth::token_source::TokenSource;
use std::sync::Arc;

#[derive(Clone)]
pub struct DatabaseAdminClient {
    inner: InternalDatabaseAdminClient<Channel>,
    token_source: Option<Arc<dyn TokenSource>>,
}
