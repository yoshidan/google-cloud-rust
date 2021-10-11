use crate::error::Error;
use serde::Deserialize;
use tokio::fs;

const CREDENTIALS_FILE: &str = "application_default_credentials.json";

#[derive(Deserialize)]
pub struct Format {
    pub tp: String,
    pub subject_token_field_name: String,
}

#[derive(Deserialize)]
pub struct CredentialSource {
    pub file: String,
    pub url: String,
    pub headers: std::collections::HashMap<String, String>,
    pub environment_id: String,
    pub region_url: String,
    pub regional_cred_verification_url: String,
    pub cred_verification_url: String,
    pub format: Format,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct CredentialsFile {
    #[serde(rename(deserialize = "type"))]
    pub tp: String,

    // Service Account fields
    pub client_email: Option<String>,
    pub private_key_id: Option<String>,
    pub private_key: Option<String>,
    pub auth_uri: Option<String>,
    pub token_uri: Option<String>,
    pub project_id: Option<String>,

    // User Credential fields
    // (These typically come from gcloud auth.)
    pub client_secret: Option<String>,
    pub client_id: Option<String>,
    pub refresh_token: Option<String>,

    // External Account fields
    pub audience: Option<String>,
    pub subject_token_type: Option<String>,
    pub token_url_external: Option<String>,
    pub token_info_url: Option<String>,
    pub service_account_impersonation_url: Option<String>,
    pub credential_source: Option<CredentialSource>,
    pub quota_project_id: Option<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct Credentials {
    client_id: String,
    client_secret: String,
    redirect_urls: Vec<String>,
    auth_uri: String,
    token_uri: String,
}

impl CredentialsFile {
    pub async fn new() -> Result<Self, Error> {
        let path = match std::env::var("GOOGLE_APPLICATION_CREDENTIALS") {
            Ok(s) => Ok(std::path::Path::new(s.as_str()).to_path_buf()),
            Err(_e) => {
                // get well known file name
                if cfg!(target_os = "windows") {
                    let app_data = std::env::var("APPDATA").unwrap();
                    Ok(std::path::Path::new(app_data.as_str())
                        .join("gcloud")
                        .join(CREDENTIALS_FILE))
                } else {
                    match home::home_dir() {
                        Some(s) => Ok(s.join(".config").join("gcloud").join(CREDENTIALS_FILE)),
                        None => Err(Error::StringError(
                            "user home directory not found".to_string(),
                        )),
                    }
                }
            }
        };

        let credentials_json = match path {
            Ok(s) => fs::read(s).await.map_err(Error::IOError)?,
            Err(e) => return Err(e),
        };

        return Ok(json::from_slice(credentials_json.as_slice()).map_err(Error::JsonError)?);
    }

    pub fn unwrap_private_key(&self) -> Result<jwt::EncodingKey, Error> {
        return jwt::EncodingKey::from_rsa_pem(self.private_key.as_ref().unwrap().as_bytes())
            .map_err(Error::JwtError);
    }
}
