use gcpauth::token::TokenSource;
use gcpauth::*;
use std::fs::File;
use std::io::Write;

#[tokio::test]
async fn test() -> Result<(), error::Error> {
    let authorized_user_credentials = std::env::var("TEST_USER_CREDENTIALS")
        .map_err(|_e| error::Error::StringError("env required".to_string()))?;

    let json = base64::decode(authorized_user_credentials)
        .map_err(|_e| error::Error::StringError("invalid cred".to_string()))?;
    let mut file = File::create(".cred.json").map_err(error::Error::IOError)?;
    file.write_all(json.as_slice())
        .map_err(error::Error::IOError)?;

    std::env::set_var("GOOGLE_APPLICATION_CREDENTIALS", ".cred.json");
    let credentials = credentials::CredentialsFile::new().await?;
    let ts = token_source::authorized_user_token_source::UserAccountTokenSource::new(&credentials)?;
    let token = ts.token().await?;
    assert_eq!("Bearer", token.token_type);
    assert_eq!(true, token.expiry.unwrap().timestamp() > 0);
    Ok(())
}
