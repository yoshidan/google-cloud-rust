use base64::prelude::*;
use serde::Deserialize;
use tokio::fs;

use crate::error::Error;

const CREDENTIALS_FILE: &str = "application_default_credentials.json";

#[allow(dead_code)]
#[derive(Deserialize, Clone, PartialEq)]
#[cfg_attr(test, derive(Debug))]
pub struct ServiceAccountImpersonationInfo {
    pub(crate) token_lifetime_seconds: i32,
}

#[allow(dead_code)]
#[derive(Deserialize, Clone, PartialEq)]
#[cfg_attr(test, derive(Debug))]
pub struct ExecutableConfig {
    pub(crate) command: String,
    pub(crate) timeout_millis: Option<i32>,
    pub(crate) output_file: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Clone, PartialEq)]
#[cfg_attr(test, derive(Debug))]
pub struct Format {
    #[serde(rename(deserialize = "type"))]
    pub(crate) tp: String,
    pub(crate) subject_token_field_name: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Clone, PartialEq)]
#[cfg_attr(test, derive(Debug))]
pub struct CredentialSource {
    pub(crate) file: Option<String>,

    pub(crate) url: Option<String>,
    pub(crate) headers: Option<std::collections::HashMap<String, String>>,

    pub(crate) executable: Option<ExecutableConfig>,

    pub(crate) environment_id: Option<String>,
    pub(crate) region_url: Option<String>,
    pub(crate) regional_cred_verification_url: Option<String>,
    pub(crate) cred_verification_url: Option<String>,
    pub(crate) imdsv2_session_token_url: Option<String>,
    pub(crate) format: Option<Format>,
}

#[allow(dead_code)]
#[derive(Deserialize, Clone, PartialEq)]
#[cfg_attr(test, derive(Debug))]
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
    #[serde(rename = "token_url")]
    pub token_url_external: Option<String>,
    pub token_info_url: Option<String>,
    pub service_account_impersonation_url: Option<String>,
    pub service_account_impersonation: Option<ServiceAccountImpersonationInfo>,
    pub delegates: Option<Vec<String>>,
    pub credential_source: Option<CredentialSource>,
    pub quota_project_id: Option<String>,
    pub workforce_pool_user_project: Option<String>,
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

        Ok(serde_json::from_slice(credentials_json.as_slice())?)
    }

    pub async fn new_from_file(filepath: String) -> Result<Self, Error> {
        let credentials_json = fs::read(filepath).await?;
        Ok(serde_json::from_slice(credentials_json.as_slice())?)
    }

    pub async fn new_from_str(str: &str) -> Result<Self, Error> {
        Ok(serde_json::from_str(str)?)
    }

    async fn json_from_env() -> Result<Vec<u8>, ()> {
        let credentials = std::env::var("GOOGLE_APPLICATION_CREDENTIALS_JSON")
            .map_err(|_| ())
            .map(Vec::<u8>::from)?;

        if let Ok(decoded) = BASE64_STANDARD.decode(credentials.clone()) {
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

        let credentials_json = fs::read(path).await.map_err(Error::CredentialsIOError)?;

        Ok(credentials_json)
    }

    pub(crate) fn try_to_private_key(&self) -> Result<jsonwebtoken::EncodingKey, Error> {
        match self.private_key.as_ref() {
            Some(key) => Ok(jsonwebtoken::EncodingKey::from_rsa_pem(key.as_bytes())?),
            None => Err(Error::NoPrivateKeyFound),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    const CREDENTIALS_FILE_CONTENT: &str = r#"{
  "type": "service_account",
  "project_id": "fake_project_id",
  "private_key_id": "fake_private_key_id",
  "private_key": "-----BEGIN PRIVATE KEY-----\nfake_private_key\n-----END PRIVATE KEY-----\n",
  "client_email": "fake@fake_project_id.iam.gserviceaccount.com",
  "client_id": "123456789010111213141516171819",
  "auth_uri": "https://accounts.google.com/o/oauth2/auth",
  "token_uri": "https://oauth2.googleapis.com/token",
  "auth_provider_x509_cert_url": "https://www.googleapis.com/oauth2/v1/certs",
  "client_x509_cert_url": "https://www.googleapis.com/robot/v1/metadata/x509/fake%40fake_project_id.iam.gserviceaccount.com",
  "universe_domain": "googleapis.com"
}"#;

    #[tokio::test]
    async fn test_credentials_file_new_from_file() {
        // setup:
        let temp_credentials_dir = tempdir().expect("Cannot create temporary directory");
        let temp_credentials_path = temp_credentials_dir.path().join(CREDENTIALS_FILE);
        let mut credentials_file = File::create(&temp_credentials_path).expect("Cannot create temporary file");
        credentials_file
            .write_all(CREDENTIALS_FILE_CONTENT.as_bytes())
            .expect("Cannot write content to file");

        // execute:
        let credentials_file_result =
            CredentialsFile::new_from_file(temp_credentials_path.to_string_lossy().to_string()).await;

        // verify:
        let expected_credentials_file: CredentialsFile =
            serde_json::from_str(CREDENTIALS_FILE_CONTENT).expect("Credentials file JSON deserialization not working");
        match credentials_file_result {
            Err(_) => panic!(),
            Ok(cf) => assert_eq!(expected_credentials_file, cf),
        }
    }

    #[tokio::test]
    async fn test_credentials_file_new_from_str() {
        // execute:
        let credentials_file_result = CredentialsFile::new_from_str(CREDENTIALS_FILE_CONTENT).await;

        // verify:
        let expected_credentials_file: CredentialsFile =
            serde_json::from_str(CREDENTIALS_FILE_CONTENT).expect("Credentials file JSON deserialization not working");
        match credentials_file_result {
            Err(_) => panic!(),
            Ok(cf) => assert_eq!(expected_credentials_file, cf),
        }
    }

    #[tokio::test]
    async fn test_credentials_file_new_from_env_var_json() {
        // setup:
        temp_env::async_with_vars(
            [
                ("GOOGLE_APPLICATION_CREDENTIALS_JSON", Some(CREDENTIALS_FILE_CONTENT)),
                ("GOOGLE_APPLICATION_CREDENTIALS", None), // make sure file env is not interfering
            ],
            async {
                // execute:
                let credentials_file_result = CredentialsFile::new().await;

                // verify:
                let expected_credentials_file: CredentialsFile = serde_json::from_str(CREDENTIALS_FILE_CONTENT)
                    .expect("Credentials file JSON deserialization not working");
                match credentials_file_result {
                    Err(_) => panic!(),
                    Ok(cf) => assert_eq!(expected_credentials_file, cf),
                }
            },
        )
        .await;
    }

    #[tokio::test]
    async fn test_credentials_file_new_from_env_var_json_base_64_encoded() {
        // setup:
        temp_env::async_with_vars(
            [
                (
                    "GOOGLE_APPLICATION_CREDENTIALS_JSON",
                    Some(base64::engine::general_purpose::STANDARD.encode(CREDENTIALS_FILE_CONTENT)),
                ),
                ("GOOGLE_APPLICATION_CREDENTIALS", None), // make sure file env is not interfering
            ],
            async {
                // execute:
                let credentials_file_result = CredentialsFile::new().await;

                // verify:
                let expected_credentials_file: CredentialsFile = serde_json::from_str(CREDENTIALS_FILE_CONTENT)
                    .expect("Credentials file JSON deserialization not working");
                match credentials_file_result {
                    Err(_) => panic!(),
                    Ok(cf) => assert_eq!(expected_credentials_file, cf),
                }
            },
        )
        .await
    }

    #[tokio::test]
    async fn test_credentials_file_new_env_var_file() {
        // setup:
        let temp_credentials_dir = tempdir().expect("Cannot create temporary directory");
        let temp_credentials_path = temp_credentials_dir.path().join(CREDENTIALS_FILE);
        let mut credentials_file = File::create(&temp_credentials_path).expect("Cannot create temporary file");

        temp_env::async_with_vars(
            [
                (
                    "GOOGLE_APPLICATION_CREDENTIALS",
                    Some(temp_credentials_path.to_string_lossy().to_string()),
                ),
                ("GOOGLE_APPLICATION_CREDENTIALS_JSON", None), // make sure file env is not interfering
            ],
            async {
                credentials_file
                    .write_all(CREDENTIALS_FILE_CONTENT.as_bytes())
                    .expect("Cannot write content to file");

                // execute:
                let credentials_file_result = CredentialsFile::new().await;

                // verify:
                let expected_credentials_file: CredentialsFile = serde_json::from_str(CREDENTIALS_FILE_CONTENT)
                    .expect("Credentials file JSON deserialization not working");
                match credentials_file_result {
                    Err(_) => panic!(),
                    Ok(cf) => assert_eq!(expected_credentials_file, cf),
                }
            },
        )
        .await
    }
}
