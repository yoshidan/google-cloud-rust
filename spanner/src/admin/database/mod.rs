pub mod database_admin_client;

#[cfg(test)]
mod tests {
    use serial_test::serial;
    use time::OffsetDateTime;

    use google_cloud_gax::conn::{ConnectionManager, ConnectionOptions, Environment};
    use google_cloud_googleapis::spanner::admin::database::v1::database::State;

    use google_cloud_googleapis::spanner::admin::database::v1::{
        CreateDatabaseRequest, Database, DatabaseDialect, DropDatabaseRequest, GetDatabaseDdlRequest,
        GetDatabaseRequest, ListDatabasesRequest, UpdateDatabaseDdlRequest,
    };
    use google_cloud_longrunning::autogen::operations_client::OperationsClient;

    use crate::admin::database::database_admin_client::DatabaseAdminClient;
    use crate::apiv1::conn_pool::{AUDIENCE, SPANNER};

    async fn new_client() -> DatabaseAdminClient {
        let conn_pool = ConnectionManager::new(
            1,
            SPANNER,
            AUDIENCE,
            &Environment::Emulator("localhost:9010".to_string()),
            &ConnectionOptions::default(),
        )
        .await
        .unwrap();
        let lro_client = OperationsClient::new(conn_pool.conn()).await.unwrap();
        DatabaseAdminClient::new(conn_pool.conn(), lro_client)
    }

    async fn create_database() -> Database {
        let client = new_client().await;
        let database_id = format!("test{}ut", OffsetDateTime::now_utc().unix_timestamp_nanos());
        let request = CreateDatabaseRequest {
            parent: "projects/local-project/instances/test-instance".to_string(),
            create_statement: format!("CREATE DATABASE {database_id}"),
            extra_statements: vec!["CREATE TABLE Tbl (ID STRING(MAX)) PRIMARY KEY(ID)".to_string()],
            encryption_config: None,
            database_dialect: DatabaseDialect::GoogleStandardSql.into(),
            proto_descriptors: vec![],
        };

        let creation_result = match client.create_database(request, None).await {
            Ok(mut res) => res.wait(None).await,
            Err(err) => panic!("err: {err:?}"),
        };
        match creation_result {
            Ok(res) => res.unwrap(),
            Err(err) => panic!("err: {err:?}"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_create_database() {
        let instance = create_database().await;
        assert_eq!(instance.state, State::Ready as i32);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_database() {
        let client = new_client().await;
        let name = "projects/local-project/instances/test-instance/databases/local-database".to_string();
        let request = GetDatabaseRequest { name: name.clone() };

        match client.get_database(request, None).await {
            Ok(res) => {
                let db = res.into_inner();
                assert_eq!(db.name, name);
            }
            Err(err) => panic!("err: {err:?}"),
        };
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_database() {
        let database = create_database().await;
        let client = new_client().await;
        let request = DropDatabaseRequest {
            database: database.name.to_string(),
        };
        let _ = client.drop_database(request, None).await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_list_databases() {
        let client = new_client().await;
        let request = ListDatabasesRequest {
            parent: "projects/local-project/instances/test-instance".to_string(),
            page_size: 1,
            page_token: "".to_string(),
        };

        match client.list_databases(request, None).await {
            Ok(res) => {
                println!("size = {}", res.len());
                assert!(!res.is_empty());
            }
            Err(err) => panic!("err: {err:?}"),
        };
    }

    #[tokio::test]
    #[serial]
    async fn test_get_database_ddl() {
        let database = create_database().await;
        let client = new_client().await;
        let request = GetDatabaseDdlRequest {
            database: database.name.to_string(),
        };

        match client.get_database_ddl(request, None).await {
            Ok(res) => {
                assert_eq!(res.into_inner().statements.len(), 1);
            }
            Err(err) => panic!("err: {err:?}"),
        };
    }

    #[tokio::test]
    #[serial]
    async fn test_update_database_ddl() {
        let database = create_database().await;
        let client = new_client().await;
        let request = UpdateDatabaseDdlRequest {
            database: database.name.to_string(),
            statements: vec!["CREATE TABLE Tbl1 (ID INT64) PRIMARY KEY(ID)".to_string()],
            operation_id: "".to_string(),
            proto_descriptors: vec![],
            throughput_mode: false,
        };

        let update_result = match client.update_database_ddl(request, None).await {
            Ok(mut res) => res.wait(None).await,
            Err(err) => panic!("err: {err:?}"),
        };
        let _ = update_result.unwrap();
    }
}
