use google_cloud_gax::conn::{Channel, ConnectionManager, Error};
use google_cloud_googleapis::spanner::admin::database::v1::database_admin_client::DatabaseAdminClient as InternalDatabaseAdminClient;
use google_cloud_googleapis::spanner::admin::instance::v1::instance_admin_client::InstanceAdminClient as InternalInstanceAdminClient;
use google_cloud_longrunning::autogen::operations_client::OperationsClient;

use crate::admin::database::database_admin_client::DatabaseAdminClient;
use crate::admin::instance::instance_admin_client::InstanceAdminClient;
use crate::admin::AdminClientConfig;
use crate::apiv1::conn_pool::{AUDIENCE, SPANNER};

#[derive(Clone)]
pub struct Client {
    database: DatabaseAdminClient,
    instance: InstanceAdminClient,
}

impl Client {
    pub async fn new(config: AdminClientConfig) -> Result<Self, Error> {
        let (conn, lro_client) = internal_client(&config).await?;
        let database = DatabaseAdminClient::new(InternalDatabaseAdminClient::new(conn), lro_client);

        let (conn, lro_client) = internal_client(&config).await?;
        let instance = InstanceAdminClient::new(InternalInstanceAdminClient::new(conn), lro_client);
        Ok(Self { database, instance })
    }

    pub fn database(&self) -> &DatabaseAdminClient {
        &self.database
    }

    pub fn instance(&self) -> &InstanceAdminClient {
        &self.instance
    }
}

async fn internal_client(config: &AdminClientConfig) -> Result<(Channel, OperationsClient), Error> {
    let conn_pool = ConnectionManager::new(1, SPANNER, AUDIENCE, &config.environment).await?;
    let conn = conn_pool.conn();
    let lro_client = OperationsClient::new(conn).await?;
    Ok((conn_pool.conn(), lro_client))
}
