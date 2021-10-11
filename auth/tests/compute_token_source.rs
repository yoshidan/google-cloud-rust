use gcpauth::*;

#[tokio::test]
async fn test_on_gce() -> Result<(), error::Error> {
    let on_gce = token_source::compute_token_source::on_gce().await;
    assert_eq!(false, on_gce);
    println!("executed first");
    let on_gce = token_source::compute_token_source::on_gce().await;
    assert_eq!(false, on_gce);
    println!("executed second");
    Ok(())
}

#[tokio::test]
//  available on GCE only
async fn test_new() -> Result<(), error::Error> {
    let scope = "https://www.googleapis.com/auth/cloud-platform,https://www.googleapis.com/auth/spanner.data";
    let ts = token_source::compute_token_source::ComputeTokenSource::new(scope);
    assert_eq!(true, ts.is_ok());
    Ok(())
}
