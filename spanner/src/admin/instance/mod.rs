pub mod instance_admin_client;

#[cfg(test)]
mod tests {
    use crate::admin::instance::instance_admin_client::InstanceAdminClient;
    use google_cloud_googleapis::Code;
    use serial_test::serial;
    use google_cloud_googleapis::spanner::admin::instance::v1::{Instance, CreateInstanceRequest};
    use google_cloud_gax::call_option::{BackoffRetrySettings, BackoffRetryer};

    #[tokio::test]
    #[serial]
    async fn test_create_instance() {
        std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
        let mut client = InstanceAdminClient::default().await.unwrap();
        let request = CreateInstanceRequest {
            parent: "projects/local-project".to_string(),
            instance_id: "test-instance2".to_string(),
            instance: Some(Instance {
                name: "projects/local-project/instances/test-instance2".to_string(),
                config: "".to_string(),
                display_name: "test-instance-ut".to_string(),
                node_count: 0,
                processing_units: 0,
                state: 0,
                labels: Default::default(),
                endpoint_uris: vec![]
            })
        };

        let creation_result= match client.create_instance(request, None).await {
            Ok(mut res) => {
                let config = BackoffRetrySettings {
                    retryer: BackoffRetryer { backoff: Default::default(), codes: vec![Code::DeadlineExceeded] }
                };
                res.wait::<Instance>(config).await
            }
            Err(err) => panic!("err: {:?}", err),
        };
        match creation_result {
            Ok(res) => {
                println!("{:?}", res.unwrap().name);
            }
            Err(err) => panic!("err: {:?}", err),
        }
    }
}
