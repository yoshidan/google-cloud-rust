use google_cloud_auth::token_source::compute_token_source::ComputeTokenSource;
use google_cloud_auth::token_source::*;
use google_cloud_auth::*;

#[tokio::test]
//  available on GCE only
async fn test_new() -> Result<(), error::Error> {
    let scope = "https://www.googleapis.com/auth/cloud-platform,https://www.googleapis.com/auth/spanner.data";
    let ts = ComputeTokenSource::new(scope);
    assert_eq!(true, ts.is_ok());
    Ok(())
}
