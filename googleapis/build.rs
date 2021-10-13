fn main() {
    tonic_build::configure()
        .build_server(false)
        .out_dir("src") // you can change the generated code's location
        .compile(
            &["googleapis/google/spanner/v1/spanner.proto"],
            &["googleapis"], // specify the root location to search proto dependencies
        )
        .unwrap();
}
