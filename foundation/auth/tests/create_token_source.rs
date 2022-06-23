use google_cloud_auth::*;

#[tokio::test]
async fn test_create_token_source() -> Result<(), error::Error> {
    let audience = "https://spanner.googleapis.com/";
    let scopes = [
        "https://www.googleapis.com/auth/cloud-platform",
        "https://www.googleapis.com/auth/spanner.data",
    ];
    let config = Config {
        audience: Some(audience),
        scopes: Some(&scopes),
    };
    let ts = create_token_source(config).await?;
    let token = ts.token().await?;
    assert_eq!("Bearer", token.token_type);
    assert!(token.expiry.unwrap().timestamp() > 0);
    Ok(())
}

#[tokio::test]
async fn test_create_token_source_without_aud() -> Result<(), error::Error> {
    let scopes = [
        "https://www.googleapis.com/auth/cloud-platform",
        "https://www.googleapis.com/auth/devstorage.full_control",
    ];
    let config = Config {
        audience: None,
        scopes: Some(&scopes),
    };
    let ts = create_token_source(config).await?;
    let token = ts.token().await?;
    assert_eq!("Bearer", token.token_type);
    assert!(token.expiry.unwrap().timestamp() > 0);
    Ok(())
}
