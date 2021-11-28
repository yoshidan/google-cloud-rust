pub mod instance_admin_client;

#[cfg(test)]
mod tests {
    use crate::admin::instance::instance_admin_client::InstanceAdminClient;
    use google_cloud_googleapis::Code;
    use serial_test::serial;
    use google_cloud_googleapis::spanner::admin::instance::v1::{Instance, CreateInstanceRequest};
    use google_cloud_gax::call_option::{BackoffRetrySettings, BackoffRetryer};
    use chrono::{Utc, Timelike};
    use google_cloud_googleapis::spanner::admin::instance::v1::instance::State;

    #[tokio::test]
    #[serial]
    async fn test_create_instance() {
        std::env::set_var("SPANNER_EMULATOR_HOST", "localhost:9010");
        let mut client = InstanceAdminClient::default().await.unwrap();
        let instance_id = format!("test{}ut",Utc::now().second());
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
                endpoint_uris: vec![]
            })
        };

        let creation_result= match client.create_instance(request, None).await {
            Ok(mut res) => {
                let config = BackoffRetrySettings {
                    retryer: BackoffRetryer { backoff: Default::default(), codes: vec![Code::DeadlineExceeded] }
                };
                res.wait(config).await
            }
            Err(err) => panic!("err: {:?}", err),
        };
        match creation_result {
            Ok(res) => {
                let instance = res.unwrap();
                assert_eq!(instance.name, name);
                assert_eq!(instance.state, State::Ready as i32)
            }
            Err(err) => panic!("err: {:?}", err),
        }
    }
}
