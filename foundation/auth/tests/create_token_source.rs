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
    assert_eq!(true, token.expiry.unwrap().timestamp() > 0);
    Ok(())
}

#[tokio::test]
async fn test_get_credentials() -> Result<(), error::Error> {
    let cred = get_credentials().await?;
    assert!(cred.project_id.is_some());
    assert!(cred.file.is_some());

    let audience = "https://spanner.googleapis.com/";
    let scopes = [
        "https://www.googleapis.com/auth/cloud-platform",
        "https://www.googleapis.com/auth/spanner.data",
    ];
    let config = Config {
        audience: Some(audience),
        scopes: Some(&scopes),
    };
    let ts = create_token_source_from_credentials(&cred, config).await?;
    let token = ts.token().await?;
    assert_eq!("Bearer", token.token_type);
    assert_eq!(true, token.expiry.unwrap().timestamp() > 0);
    Ok(())
}
