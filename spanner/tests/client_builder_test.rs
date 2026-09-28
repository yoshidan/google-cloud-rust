use gcloud_spanner::client::{Client, ClientConfig, Compression, Error};

// Construct and destructure every original public field without `..` so adding
// fields to ClientConfig cannot silently break downstream users again.
fn legacy_config() -> ClientConfig {
    let ClientConfig {
        session_config,
        channel_config,
        endpoint,
        environment,
        disable_route_to_leader,
        metrics,
    } = ClientConfig::default();
    ClientConfig {
        session_config,
        channel_config,
        endpoint,
        environment,
        disable_route_to_leader,
        metrics,
    }
}

#[tokio::test]
async fn legacy_config_and_builder_share_validation() {
    let mut config = legacy_config();
    config.session_config.max_opened = config.channel_config.num_channels * 100 + 1;
    let legacy_error = match Client::new("projects/test/instances/test/databases/test", config).await {
        Err(Error::InvalidConfig(message)) => message,
        _ => panic!("Expected session capacity validation before connecting"),
    };
    let mut config = legacy_config();
    config.session_config.max_opened = config.channel_config.num_channels * 100 + 1;
    let builder_error = match Client::builder("projects/test/instances/test/databases/test")
        .config(config)
        .compression(Compression::Gzip)
        .build()
        .await
    {
        Err(Error::InvalidConfig(message)) => message,
        _ => panic!("Expected session capacity validation before connecting"),
    };
    assert_eq!(legacy_error, builder_error);
}
