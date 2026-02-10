pub mod instance_admin_client;

#[cfg(test)]
mod tests {
    use serial_test::serial;
    use time::OffsetDateTime;

    use google_cloud_gax::conn::{ConnectionManager, ConnectionOptions, Environment};
    use google_cloud_googleapis::spanner::admin::instance::v1::instance::State;

    use google_cloud_googleapis::spanner::admin::instance::v1::{
        CreateInstanceRequest, DeleteInstanceRequest, GetInstanceConfigRequest, GetInstanceRequest, Instance,
        ListInstanceConfigsRequest, ListInstancesRequest,
    };
    use google_cloud_longrunning::autogen::operations_client::OperationsClient;

    use crate::admin::instance::instance_admin_client::InstanceAdminClient;
    use crate::apiv1::conn_pool::{AUDIENCE, SPANNER};

    async fn new_client() -> InstanceAdminClient {
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
        InstanceAdminClient::new(conn_pool.conn(), lro_client)
    }

    async fn create_instance() -> Instance {
        let client = new_client().await;
        let instance_id = format!("test{}ut", OffsetDateTime::now_utc().unix_timestamp_nanos());
        let name = format!("projects/local-project/instances/{instance_id}");
        let request = CreateInstanceRequest {
            parent: "projects/local-project".to_string(),
            instance_id: instance_id.to_string(),
            instance: Some(Instance {
                name: name.to_string(),
                config: "".to_string(),
                display_name: "test-instance-ut".to_string(),
                node_count: 0,
                processing_units: 0,
                replica_compute_capacity: vec![],
                autoscaling_config: None,
                state: 0,
                labels: Default::default(),
                instance_type: 0,
                endpoint_uris: vec![],
                create_time: None,
                update_time: None,
                free_instance_metadata: None,
                edition: 0,
                default_backup_schedule_type: 0,
            }),
        };

        let creation_result = match client.create_instance(request, None).await {
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
    async fn test_create_instance() {
        let instance = create_instance().await;
        assert_eq!(instance.state, State::Ready as i32);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_instance() {
        let client = new_client().await;
        let name = "projects/local-project/instances/test-instance".to_string();
        let request = GetInstanceRequest {
            name: name.clone(),
            field_mask: None,
        };

        match client.get_instance(request, None).await {
            Ok(res) => {
                let instance = res.into_inner();
                assert_eq!(instance.name, name);
            }
            Err(err) => panic!("err: {err:?}"),
        };
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_instance() {
        let instance = create_instance().await;
        let client = new_client().await;
        let request = DeleteInstanceRequest {
            name: instance.name.to_string(),
        };
        let _ = client.delete_instance(request, None).await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_list_instances() {
        let client = new_client().await;
        let request = ListInstancesRequest {
            parent: "projects/local-project".to_string(),
            page_size: 1,
            page_token: "".to_string(),
            filter: "".to_string(),
            instance_deadline: None,
        };

        match client.list_instances(request, None).await {
            Ok(res) => {
                println!("size = {}", res.len());
                assert!(!res.is_empty());
            }
            Err(err) => panic!("err: {err:?}"),
        };
    }

    #[tokio::test]
    #[serial]
    async fn test_list_instance_configs() {
        let client = new_client().await;
        let request = ListInstanceConfigsRequest {
            parent: "projects/local-project".to_string(),
            page_size: 1,
            page_token: "".to_string(),
        };

        match client.list_instance_configs(request, None).await {
            Ok(res) => {
                println!("size = {}", res.len());
                assert!(!res.is_empty());
            }
            Err(err) => panic!("err: {err:?}"),
        };
    }

    #[tokio::test]
    #[serial]
    async fn test_get_instance_config() {
        let client = new_client().await;
        let name = "projects/local-project/instanceConfigs/emulator-config".to_string();
        let request = GetInstanceConfigRequest { name: name.clone() };

        match client.get_instance_config(request, None).await {
            Ok(res) => {
                let instance = res;
                assert_eq!(instance.name, name);
            }
            Err(err) => panic!("err: {err:?}"),
        };
    }
}
