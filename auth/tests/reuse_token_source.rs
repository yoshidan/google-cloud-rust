use google_cloud_auth::token_source::reuse_token_source::ReuseTokenSource;
use google_cloud_auth::token_source::service_account_token_source::ServiceAccountTokenSource;
use google_cloud_auth::token_source::token_source::TokenSource;
use google_cloud_auth::token_source::*;
use google_cloud_auth::*;

#[tokio::test]
async fn test_reuse_token_source() -> Result<(), error::Error> {
    let credentials = credentia::CredentialsFile::new().await?;
    let audience = "https://spanner.googleapis.com/";
    let ts = ServiceAccountTokenSource::new(&credentials, audience)?;
    let token = ts.token().await?;
    assert_eq!(true, token.expiry.unwrap().timestamp() > 0);
    let old_token_value = token.access_token.clone();
    let rts = ReuseTokenSource::new(Box::new(ts), token);
    let new_token = rts.token().await?;
    assert_eq!(old_token_value, new_token.access_token);
    Ok(())
}
