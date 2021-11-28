use google_cloud_googleapis::Status;
use hyper::Response;

pub mod database;
pub mod instance;

const SCOPES: [&str; 2] = [
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/spanner.admin",
];
