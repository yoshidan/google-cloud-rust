use crate::error::Error;
use serde::Deserialize;
use tokio::fs;

const CREDENTIALS_FILE: &str = "application_default_credentials.json";

#[allow(dead_code)]
#[derive(Deserialize)]
pub(crate) struct Format {
    tp: String,
    subject_token_field_name: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct CredentialSource {
    file: String,
    url: String,
    headers: std::collections::HashMap<String, String>,
    environment_id: String,
    region_url: String,
    regional_cred_verification_url: String,
    cred_verification_url: String,
    format: Format,
}

#[allow(dead_code)]
#[derive(Deserialize)]
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

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct Credentials {
    client_id: String,
    client_secret: String,
    redirect_urls: Vec<String>,
    auth_uri: String,
    token_uri: String,
}

impl CredentialsFile {
    pub async fn new() -> Result<Self, Error> {
        let credentials_json = {
            if let Ok(credentials) = Self::json_from_env().await {
                credentials
            } else {
                Self::json_from_file().await?
            }
        };

        Ok(json::from_slice(credentials_json.as_slice())?)
    }

    async fn json_from_env() -> Result<Vec<u8>, ()> {
        let credentials = std::env::var("GOOGLE_APPLICATION_CREDENTIALS_JSON")
            .map_err(|_| ())
            .map(Vec::<u8>::from)?;

        if let Ok(decoded) = base64::decode(credentials.clone()) {
            Ok(decoded)
        } else {
            Ok(credentials)
        }
    }

    async fn json_from_file() -> Result<Vec<u8>, Error> {
        let path = match std::env::var("GOOGLE_APPLICATION_CREDENTIALS") {
            Ok(s) => Ok(std::path::Path::new(s.as_str()).to_path_buf()),
            Err(_e) => {
                // get well known file name
                if cfg!(target_os = "windows") {
                    let app_data = std::env::var("APPDATA")?;
                    Ok(std::path::Path::new(app_data.as_str())
                        .join("gcloud")
                        .join(CREDENTIALS_FILE))
                } else {
                    match home::home_dir() {
                        Some(s) => Ok(s.join(".config").join("gcloud").join(CREDENTIALS_FILE)),
                        None => Err(Error::NoHomeDirectoryFound),
                    }
                }
            }
        }?;

        let credentials_json = fs::read(path).await?;

        Ok(credentials_json)
    }

    pub(crate) fn try_to_private_key(&self) -> Result<jwt::EncodingKey, Error> {
        match self.private_key.as_ref() {
            Some(key) => Ok(jwt::EncodingKey::from_rsa_pem(key.as_bytes())?),
            None => Err(Error::NoPrivateKeyFound),
        }
    }
}
