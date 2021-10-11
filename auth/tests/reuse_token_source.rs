use gcpauth::token::TokenSource;
use gcpauth::*;

#[tokio::test]
async fn test_reuse_token_source() -> Result<(), error::Error> {
    let credentials = credentials::CredentialsFile::new().await?;
    let audience = "https://spanner.googleapis.com/";
    let ts = token_source::service_account_token_source::ServiceAccountTokenSource::new(
        &credentials,
        audience,
    )?;
    let token = ts.token().await?;
    assert_eq!(true, token.expiry.unwrap().timestamp() > 0);
    let old_token_value = token.access_token.clone();
    let rts = token_source::reuse_token_source::ReuseTokenSource::new(Box::new(ts), token);
    let new_token = rts.token().await?;
    assert_eq!(old_token_value, new_token.access_token);
    Ok(())
}
