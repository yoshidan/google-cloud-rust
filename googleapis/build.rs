#[cfg(feature = "generate")]
fn main() {
    tonic_build::configure()
        .build_server(false)
        .out_dir("src") // you can change the generated code's location
        .compile(
            &[
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

#[cfg(not(feature = "generate"))]
fn main() {}
