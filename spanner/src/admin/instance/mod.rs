pub mod instance_admin_client;

#[cfg(test)]
mod tests {
    use crate::admin::instance::instance_admin_client::InstanceAdminClient;
    use chrono::{Utc};
    
    use google_cloud_googleapis::spanner::admin::instance::v1::instance::State;
    use google_cloud_googleapis::spanner::admin::instance::v1::{
        CreateInstanceRequest, DeleteInstanceRequest, GetInstanceRequest, Instance,
    };
    
    use serial_test::serial;

    async fn create_instance() -> Instance {
        std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
        let mut client = InstanceAdminClient::default().await.unwrap();
        let instance_id = format!("test{}ut", Utc::now().timestamp_nanos());
        let name = format!("projects/local-project/instances/{}", instance_id);
        let request = CreateInstanceRequest {
            parent: "projects/local-project".to_string(),
            instance_id: instance_id.to_string(),
            instance: Some(Instance {
                name: name.to_string(),
                config: "".to_string(),
                display_name: "test-instance-ut".to_string(),
                node_count: 0,
                processing_units: 0,
                state: 0,
                labels: Default::default(),
                endpoint_uris: vec![],
            }),
        };

        let creation_result = match client.create_instance(request, None).await {
            Ok(mut res) => res.wait(None).await,
            Err(err) => panic!("err: {:?}", err),
        };
        match creation_result {
            Ok(res) => return res.unwrap(),
            Err(err) => panic!("err: {:?}", err),
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
        std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
        let mut client = InstanceAdminClient::default().await.unwrap();
        let name = format!("projects/local-project/instances/test-instance");
        let request = GetInstanceRequest {
            name: name.clone(),
            field_mask: None,
        };

        match client.get_instance(request, None).await {
            Ok(res) => {
                let instance = res.into_inner();
                assert_eq!(instance.name, name);
            }
            Err(err) => panic!("err: {:?}", err),
        };
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_instance() {
        let instance = create_instance().await;
        let mut client = InstanceAdminClient::default().await.unwrap();
        let request = DeleteInstanceRequest {
            name: instance.name.to_string(),
        };
        match client.delete_instance(request, None).await {
            Ok(_res) => assert!(true),
            Err(err) => panic!("err: {:?}", err),
        };
    }
}
