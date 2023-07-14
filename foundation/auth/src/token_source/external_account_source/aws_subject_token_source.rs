use std::env::var;
use std::fmt::{Debug, Formatter};
use std::path::PathBuf;

use async_trait::async_trait;
use hmac::Mac;
use path_clean::PathClean;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;

use time::macros::format_description;
use url::Url;

use crate::credentials::CredentialSource;
use crate::misc::UnwrapOrEmpty;
use crate::token_source::default_http_client;
use crate::token_source::external_account_source::error::Error;
use crate::token_source::external_account_source::subject_token_source::SubjectTokenSource;

const AWS_ALGORITHM: &str = "AWS4-HMAC-SHA256";
const AWS_REQUEST_TYPE: &str = "aws4_request";
const AWS_ACCESS_KEY_ID: &str = "AWS_ACCESS_KEY_ID";
const AWS_DEFAULT_REGION: &str = "AWS_DEFAULT_REGION";
const AWS_REGION: &str = "AWS_REGION";
const AWS_SECRET_ACCESS_KEY: &str = "AWS_SECRET_ACCESS_KEY";
const AWS_SESSION_TOKEN: &str = "AWS_SESSION_TOKEN";
const AWS_IMDS_V2_SESSION_TOKEN_HEADER: &str = "X-aws-ec2-metadata-token";

pub struct AWSSubjectTokenSource {
    subject_token_url: Url,
    target_resource: Option<String>,
    credentials: AWSSecurityCredentials,
    region: String,
}

impl Debug for AWSSubjectTokenSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AWSSubjectTokenSource")
            .field("target_resource", &self.target_resource)
            .field("region", &self.region)
            .finish_non_exhaustive()
    }
}

impl AWSSubjectTokenSource {
    pub async fn new(audience: Option<String>, value: CredentialSource) -> Result<Self, Error> {
        if !validate_metadata_server(&value.region_url) {
            return Err(Error::InvalidRegionURL(value.region_url.unwrap_or_empty()));
        }
        // Not value.cred_verification_url but value.url
        if !validate_metadata_server(&value.url) {
            return Err(Error::InvalidCredVerificationURL(value.url.unwrap_or_empty()));
        }
        if !validate_metadata_server(&value.imdsv2_session_token_url) {
            return Err(Error::InvalidIMDSv2SessionTokenURL(
                value.imdsv2_session_token_url.unwrap_or_empty(),
            ));
        }

        let aws_session_token = if should_use_metadata_server() {
            get_aws_session_token(&value.imdsv2_session_token_url).await?
        } else {
            None
        };

        let credentials = get_security_credentials(&aws_session_token, &value.url).await?;
        let region = get_region(&aws_session_token, &value.region_url).await?;

        let url = value
            .regional_cred_verification_url
            .as_ref()
            .ok_or(Error::MissingRegionalCredVerificationURL)?;
        let subject_token_url = Url::parse(&url.replace("{region}", &region))?;

        Ok(Self {
            subject_token_url,
            target_resource: audience,
            credentials,
            region,
        })
    }

    fn generate_authentication(
        &self,
        method: &str,
        now: &OffsetDateTime,
        headers: &[(&str, &str)],
    ) -> Result<String, Error> {
        let date_stamp = now.format(&format_description!("[year][month][day]"))?;
        let service_name: Vec<String> = self
            .subject_token_url
            .host_str()
            .unwrap_or_default()
            .split('.')
            .map(|v| v.to_string())
            .collect();
        let service_name = service_name[0].as_str();
        let credential_scope = format!("{}/{}/{}/{}", date_stamp, &self.region, service_name, AWS_REQUEST_TYPE);

        // canonicalize headers
        let (header_keys, header_values) = canonical_headers(headers);

        // canonicalize query
        let query = self.subject_token_url.query().unwrap_or_default();

        // canonicalize path
        let path = self.subject_token_url.path();
        let path = if path.is_empty() {
            "/".to_string()
        } else {
            PathBuf::from(path).clean().to_string_lossy().to_string()
        };

        // canonicalize request
        let data_hash = hex::encode(Sha256::digest(vec![])); // hash for empty body
        let request_string = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            method, path, query, header_keys, header_values, data_hash
        );
        let request_hash = hex::encode(Sha256::digest(request_string.as_bytes()));
        let date_stamp = now.format(&format_description!("[year][month][day]T[hour][minute][second]Z"))?;
        let string_to_sign = format!("{}\n{}\n{}\n{}", AWS_ALGORITHM, date_stamp, credential_scope, request_hash);

        // sign
        let mut signing_key = format!("AWS4{}", self.credentials.secret_access_key).into_bytes();
        for input in [self.region.as_str(), service_name, AWS_REQUEST_TYPE, string_to_sign.as_str()] {
            let mut mac = hmac::Hmac::<Sha256>::new_from_slice(&signing_key)?;
            mac.update(input.as_bytes());
            let result = mac.finalize();
            signing_key = result.into_bytes().to_vec();
        }

        Ok(format!(
            "{} Credential={}/{}, SignedHeaders={}, Signature={}",
            AWS_ALGORITHM,
            self.credentials.access_key_id,
            credential_scope,
            header_keys,
            hex::encode(signing_key)
        ))
    }
}

#[async_trait]
impl SubjectTokenSource for AWSSubjectTokenSource {
    async fn subject_token(&self) -> Result<String, Error> {
        let now = OffsetDateTime::now_utc();
        let format_date = now.format(&format_description!("[year][month][day]T[hour][minute][second]Z"))?;
        let mut sorted_headers: Vec<(&str, &str)> = vec![
            ("host", self.subject_token_url.host_str().unwrap_or("")),
            ("x-amz-date", &format_date),
        ];
        if let Some(security_token) = &self.credentials.token {
            sorted_headers.push(("x-amz-security-token", security_token));
        }
        // The full, canonical resource name of the workload identity pool
        // provider, with or without the HTTPS prefix.
        // Including this header as part of the signature is recommended to
        // ensure data integrity.
        if let Some(target_resource) = &self.target_resource {
            sorted_headers.push(("x-goog-cloud-target-resource", target_resource));
        }
        let method = "POST";
        let authorization = self.generate_authentication(method, &now, &sorted_headers)?;

        let mut aws_headers = Vec::with_capacity(sorted_headers.len() + 1);
        aws_headers.push(AWSRequestHeader {
            key: "Authorization".to_string(),
            value: authorization,
        });
        for header in sorted_headers {
            aws_headers.push(AWSRequestHeader {
                key: header.0.to_string(),
                value: header.1.to_string(),
            })
        }
        let aws_request = AWSRequest {
            url: self.subject_token_url.to_string(),
            method,
            headers: aws_headers,
        };
        let result = serde_json::to_string(&aws_request)?;
        Ok(utf8_percent_encode(&result, NON_ALPHANUMERIC).to_string())
    }
}

#[derive(Deserialize)]
struct AWSSecurityCredentials {
    #[serde(rename = "AccessKeyID")]
    access_key_id: String,
    secret_access_key: String,
    token: Option<String>,
}

#[derive(Serialize)]
struct AWSRequestHeader {
    key: String,
    value: String,
}

#[derive(Serialize)]
struct AWSRequest {
    url: String,
    method: &'static str,
    headers: Vec<AWSRequestHeader>,
}

const VALID_HOST_NAMES: [&str; 2] = ["169.254.169.254", "fd00:ec2::254"];

fn validate_metadata_server(metadata_url: &Option<String>) -> bool {
    let metadata_url = metadata_url.unwrap_or_empty();
    if metadata_url.is_empty() {
        return true;
    }
    let host = match Url::parse(&metadata_url) {
        Err(_) => return false,
        Ok(v) => v,
    };

    VALID_HOST_NAMES.contains(&host.host_str().unwrap_or(""))
}

fn should_use_metadata_server() -> bool {
    !can_retrieve_region_from_environment() || !can_retrieve_security_credential_from_environment()
}

fn can_retrieve_region_from_environment() -> bool {
    var(AWS_REGION).is_ok() || var(AWS_DEFAULT_REGION).is_ok()
}

fn can_retrieve_security_credential_from_environment() -> bool {
    var(AWS_ACCESS_KEY_ID).is_ok() && var(AWS_SECRET_ACCESS_KEY).is_ok()
}

async fn get_aws_session_token(imds_v2_session_token_url: &Option<String>) -> Result<Option<String>, Error> {
    let url = match imds_v2_session_token_url {
        Some(url) => url,
        None => return Ok(None),
    };

    let client = default_http_client();
    let response = client
        .put(url)
        .header("X-aws-ec2-metadata-token-ttl-seconds", "300")
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(Error::UnexpectedStatusOnGetSessionToken(response.status().as_u16()))
    }
    Ok(response.text().await.map(Some)?)
}

async fn get_security_credentials(
    temporary_session_token: &Option<String>,
    url: &Option<String>,
) -> Result<AWSSecurityCredentials, Error> {
    if can_retrieve_security_credential_from_environment() {
        return Ok(AWSSecurityCredentials {
            access_key_id: var(AWS_ACCESS_KEY_ID).unwrap(),
            secret_access_key: var(AWS_SECRET_ACCESS_KEY).unwrap(),
            token: var(AWS_SESSION_TOKEN).ok(),
        });
    }
    tracing::debug!("start get_security_credentials url = {:?}", url);

    // get metadata role name
    let url = url.as_ref().ok_or(Error::MissingSecurityCredentialsURL)?;
    let client = default_http_client();
    let mut builder = client.get(url);
    if let Some(token) = temporary_session_token {
        builder = builder.header(AWS_IMDS_V2_SESSION_TOKEN_HEADER, token);
    }
    let response = builder.send().await?;
    if !response.status().is_success() {
        return Err(Error::UnexpectedStatusOnGetRoleName(response.status().as_u16()))
    }
    let role_name = response.text().await?;

    let url = format!("{}/{}", url, role_name);
    tracing::debug!("start get_security_credentials by role url = {:?}", url);

    // get metadata security credentials
    let mut builder = client
        .get(url)
        .header("Content-Type", "application/json");
    if let Some(token) = temporary_session_token {
        builder = builder.header(AWS_IMDS_V2_SESSION_TOKEN_HEADER, token);
    }
    let response = builder.send().await?;
    if !response.status().is_success() {
        return Err(Error::UnexpectedStatusOnGetCredentials(response.status().as_u16()))
    }
    let cred: AWSSecurityCredentials = response.json().await?;
    Ok(cred)
}

async fn get_region(temporary_session_token: &Option<String>, url: &Option<String>) -> Result<String, Error> {
    if can_retrieve_region_from_environment() {
        if let Ok(region) = var(AWS_REGION) {
            return Ok(region);
        }
        return Ok(var(AWS_DEFAULT_REGION).unwrap());
    }
    let url = url.as_ref().ok_or(Error::MissingRegionURL)?;
    let client = default_http_client();
    let mut builder = client.get(url);
    if let Some(token) = temporary_session_token {
        builder = builder.header(AWS_IMDS_V2_SESSION_TOKEN_HEADER, token);
    }
    let response = builder.send().await?;
    if !response.status().is_success() {
        return Err(Error::UnexpectedStatusOnGetRegion(response.status().as_u16()))
    }
    let body = response.bytes().await?;

    // This endpoint will return the region in format: us-east-2b.
    // Only the us-east-2 part should be used.
    let resp_body_end = if !body.is_empty() { body.len() - 1 } else { 0 };
    Ok(String::from_utf8_lossy(&body[0..resp_body_end]).to_string())
}

fn canonical_headers<'a>(sorted_headers: &[(&'a str, &'a str)]) -> (String, String) {
    let mut full_headers: Vec<String> = Vec::with_capacity(sorted_headers.len());
    let mut keys = Vec::with_capacity(sorted_headers.len());
    for header in sorted_headers {
        keys.push(header.0);
        full_headers.push(format!("{}:{}", header.0, header.1));
    }
    (keys.join(";"), full_headers.join("\n"))
}
