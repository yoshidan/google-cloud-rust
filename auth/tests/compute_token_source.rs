use gcpauth::*;

#[tokio::test]
//  available on GCE only
async fn test_new() -> Result<(), error::Error> {
    let scope = "https://www.googleapis.com/auth/cloud-platform,https://www.googleapis.com/auth/spanner.data";
    let ts = token_source::compute_token_source::ComputeTokenSource::new(scope);
    assert_eq!(true, ts.is_ok());
    Ok(())
}
