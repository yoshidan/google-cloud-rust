use google_cloud_auth::token_source::service_account_token_source::{
    OAuth2ServiceAccountTokenSource, ServiceAccountTokenSource,
};
use google_cloud_auth::token_source::token_source::TokenSource;
use google_cloud_auth::token_source::*;
use google_cloud_auth::*;

#[tokio::test]
async fn test_jwt_token_source() -> Result<(), error::Error> {
    let credentials = credentials::CredentialsFile::new().await?;
    let audience = "https://spanner.googleapis.com/";
    let ts = ServiceAccountTokenSource::new(&credentials, audience)?;
    let token = ts.token().await?;
    assert_eq!("Bearer", token.token_type);
    assert_eq!(true, token.expiry.unwrap().timestamp() > 0);
    Ok(())
}

#[tokio::test]
async fn test_oauth2_token_source() -> Result<(), error::Error> {
    let credentials = credentials::CredentialsFile::new().await?;
    let scope = "https://www.googleapis.com/auth/cloud-platform https://www.googleapis.com/auth/spanner.data";
    let ts = OAuth2ServiceAccountTokenSource::new(&credentials, scope)?;
    let token = ts.token().await?;
    assert_eq!("Bearer", token.token_type);
    assert_eq!(true, token.expiry.unwrap().timestamp() > 0);
    Ok(())
}
