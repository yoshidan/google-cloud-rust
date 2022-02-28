pub mod database_admin_client;

#[cfg(test)]
mod tests {
    use crate::admin::database::database_admin_client::DatabaseAdminClient;
    use chrono::Utc;

    use google_cloud_googleapis::spanner::admin::database::v1::database::State;
    use google_cloud_googleapis::spanner::admin::database::v1::{
        CreateDatabaseRequest, Database, DropDatabaseRequest, GetDatabaseDdlRequest, GetDatabaseRequest,
        ListDatabasesRequest, UpdateDatabaseDdlRequest,
    };

    use serial_test::serial;

    async fn create_database() -> Database {
        std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
        let client = DatabaseAdminClient::default().await.unwrap();
        let database_id = format!("test{}ut", Utc::now().timestamp_nanos());
        let request = CreateDatabaseRequest {
            parent: "projects/local-project/instances/test-instance".to_string(),
            create_statement: format!("CREATE DATABASE {}", database_id),
            extra_statements: vec!["CREATE TABLE Tbl (ID STRING(MAX)) PRIMARY KEY(ID)".to_string()],
            encryption_config: None,
        };

        let creation_result = match client.create_database(request, None, None).await {
            Ok(mut res) => res.wait(None, None).await,
            Err(err) => panic!("err: {:?}", err),
        };
        match creation_result {
            Ok(res) => return res.unwrap(),
            Err(err) => panic!("err: {:?}", err),
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
        std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
        let client = DatabaseAdminClient::default().await.unwrap();
        let name = format!("projects/local-project/instances/test-instance/databases/local-database");
        let request = GetDatabaseRequest { name: name.clone() };

        match client.get_database(request, None, None).await {
            Ok(res) => {
                let db = res.into_inner();
                assert_eq!(db.name, name);
            }
            Err(err) => panic!("err: {:?}", err),
        };
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_database() {
        let database = create_database().await;
        let client = DatabaseAdminClient::default().await.unwrap();
        let request = DropDatabaseRequest {
            database: database.name.to_string(),
        };
        match client.drop_database(request, None, None).await {
            Ok(_res) => assert!(true),
            Err(err) => panic!("err: {:?}", err),
        };
    }

    #[tokio::test]
    #[serial]
    async fn test_list_databases() {
        std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
        let client = DatabaseAdminClient::default().await.unwrap();
        let request = ListDatabasesRequest {
            parent: "projects/local-project/instances/test-instance".to_string(),
            page_size: 1,
            page_token: "".to_string(),
        };

        match client.list_databases(request, None, None).await {
            Ok(res) => {
                println!("size = {}", res.len());
                assert!(res.len() > 0);
            }
            Err(err) => panic!("err: {:?}", err),
        };
    }

    #[tokio::test]
    #[serial]
    async fn test_get_database_ddl() {
        let database = create_database().await;
        std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
        let client = DatabaseAdminClient::default().await.unwrap();
        let request = GetDatabaseDdlRequest {
            database: database.name.to_string(),
        };

        match client.get_database_ddl(request, None, None).await {
            Ok(res) => {
                assert_eq!(res.into_inner().statements.len(), 1);
            }
            Err(err) => panic!("err: {:?}", err),
        };
    }

    #[tokio::test]
    #[serial]
    async fn test_update_database_ddl() {
        let database = create_database().await;
        std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
        let client = DatabaseAdminClient::default().await.unwrap();
        let request = UpdateDatabaseDdlRequest {
            database: database.name.to_string(),
            statements: vec!["CREATE TABLE Tbl1 (ID INT64) PRIMARY KEY(ID)".to_string()],
            operation_id: "".to_string(),
        };

        let update_result = match client.update_database_ddl(request, None, None).await {
            Ok(mut res) => res.wait(None, None).await,
            Err(err) => panic!("err: {:?}", err),
        };
        match update_result {
            Ok(_) => assert!(true),
            Err(err) => panic!("err: {:?}", err),
        }
    }
}
