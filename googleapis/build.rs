fn main() {
    #[cfg(feature = "generate")]
    {
        let config = prost_build::Config::new();
        generate(config, "src");
    }

    #[cfg(all(feature = "generate", feature = "bytes"))]
    {
        let mut bytes_config = prost_build::Config::new();
        bytes_config.bytes(&["."]);
        generate(bytes_config, "src/bytes");
    }
}

#[cfg(feature = "generate")]
fn generate(config: prost_build::Config, out_dir: impl AsRef<std::path::Path>) {
    tonic_build::configure()
        .build_server(false)
        .out_dir(out_dir) // you can change the generated code's location
        .compile_with_config(
            config,
            &[
                "googleapis/google/cloud/bigquery/storage/v1/storage.proto",
                "googleapis/google/storage/v2/storage.proto",
                "googleapis/google/pubsub/v1/pubsub.proto",
                "googleapis/google/spanner/v1/spanner.proto",
                "googleapis/google/spanner/admin/database/v1/spanner_database_admin.proto",
                "googleapis/google/spanner/admin/instance/v1/spanner_instance_admin.proto",
            ],
            &["googleapis"], // specify the root location to search proto dependencies
        )
        .unwrap();
}
