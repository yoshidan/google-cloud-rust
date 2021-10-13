#[path = "google.rpc.rs"]
pub mod rpc;

#[path = ""]
pub mod spanner {
    #[path = "google.spanner.v1.rs"]
    pub mod v1;
}
