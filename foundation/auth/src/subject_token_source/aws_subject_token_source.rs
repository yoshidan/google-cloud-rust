use std::env::var;
use crate::credentials::CredentialSource;
use crate::error::Error;
use crate::subject_token_source::SubjectTokenSource;
use std::fmt::{Debug, Formatter};
use reqwest::RequestBuilder;
use serde::Deserialize;
use time::format_description::well_known::Rfc3339;
use time::macros::format_description;
use time::{Duration, OffsetDateTime};
use crate::misc::UnwrapOrEmpty;

#[derive(Deserialize)]
struct AWSSecurityCredentials {
    #[serde(rename="AccessKeyID")]
    access_key_id: String,
    secret_access_key: String,
    token: String,
}

pub struct AWSSubjectTokenSource {
    region_url: Option<String>,
    region_cred_verification_url: Option<String>,
    cred_verification_url: Option<String>,
    target_resource: Option<String>,
    imdsv2_session_token_url: Option<String>,
    request_signer: AWSRequestSigner,
}

impl AWSSubjectTokenSource {
    pub async fn new(audience: Option<String>, value: CredentialSource) -> Result<Self, Self::Error> {
        if !validate_metadata_server(&value.region_url) {
            return Err(Error::InvalidRegionURL(value.region_url));
        }
        if !validate_metadata_server(&value.cred_verification_url) {
            return Err(Error::InvalidCredVerificationURL(value.cred_verification_url));
        }
        if !validate_metadata_server(&value.imdsv2_session_token_url) {
            return Err(Error::InvalidIMDSv2SessionTokenURL(value.imdsv2_session_token_url));
        }

        let aws_session_token = if should_use_metadata_server() {
            get_aws_session_token(&value.imdsv2_session_token_url).await?
        }else {
            None
        };

        // not value.cred_verification_url
        let cred_verification_url = value.url;
        let credentials = get_security_credentials(&aws_session_token, &cred_verification_url).await?;

        let region = get_region(&aws_session_token, &value.region_url).await?;

        let request_signer = AWSRequestSigner {
            credentials,
            region
        };

        Ok(Self{
            region_url: value.region_url,
            region_cred_verification_url: value.regional_cred_verification_url,
            cred_verification_url,
            target_resource: audience,
            imdsv2_session_token_url: value.imdsv2_session_token_url,
            request_signer
        })
    }
}

impl SubjectTokenSource for AWSSubjectTokenSource {
    async fn subject_token(&self) -> Result<String, Error> {
        todo!("implements")
    }
}

const VALID_HOST_NAMES : [&str; 2] = ["169.254.169.254", "fd00:ec2::254"];

fn validate_metadata_server(metadata_url: &Option<String>) -> bool {
    let metadata_url = metadata_url.unwrap_or_empty();
    if metadata_url.is_empty() {
        return true
    }
    let host = match url::Url::parse(&metadata_url) {
        Err(_) => return false,
        Ok(v) => v.host_str().unwrap_or("")
    };

    VALID_HOST_NAMES.contains(&host)
}

struct AWSRequestSigner {
   credentials: AWSSecurityCredentials,
   region: String
}

impl AWSRequestSigner {
    fn sign_request(&self, req: RequestBuilder) -> Result<RequestBuilder, Error> {
        let format = format_description!("[year][month][day]T[hour][minute][second]Z");
        let now = OffsetDateTime::now_utc().format(&format)?;
        let mut req = req.header("x-amz-date", now);
        if let Some(security_token) = &self.credentials.security_token {
           req = req.header("x-amz-security-token", security_token);
        }
        self.with_auth(req)
    }

    fn with_auth(&self, req: RequestBuilder) -> Result<RequestBuilder, Error> {
        todo!("implements")
    }
}

fn should_use_metadata_server() -> bool {
    !can_retrieve_region_from_environment() || !can_retrieve_security_credential_from_environment()
}

fn can_retrieve_region_from_environment() -> bool {
    var("AWS_REGION").is_ok() || var("AWS_DEFAULT_REGION").is_ok()
}

fn can_retrieve_security_credential_from_environment() -> bool {
    var("AWS_ACCESS_KEY_ID").is_ok() && var("AWS_SECRET_ACCESS_KEY").is_ok()
}

async fn get_aws_session_token(imds_v2_session_token_url: &Option<String>) -> Result<Option<String>, Error> {
    let url = match imds_v2_session_token_url {
        Some(url) => url,
        None => return Ok(None)
    };

    let client = default_http_client();
    let response = client.put(&url).header("X-aws-ec2-metadata-token-ttl-seconds", "300").send().await?;
    Ok(response.text().await.map(Some)?)
}

async fn get_security_credentials(temporary_session_token: &Option<String>, url: &Option<String>) -> Result<AWSSecurityCredentials, Error> {
    if can_retrieve_security_credential_from_environment() {
        return Ok(AWSSecurityCredentials {
            access_key_id: var("AWS_ACCESS_KEY_ID").unwrap(),
            secret_access_key: var("AWS_SECRET_ACCESS_KEY").unwrap(),
            token: var("AWS_SESSION_TOKEN").unwrap(),
        });
    }

    // get metadata role name TODO error code
    let url = &url.ok_or(Error::NoCredentialsSource)?;
    let client = default_http_client();
    let builder=  client.get(url);
    if let(token) = temporary_session_token {
        builder.header("X-aws-ec2-metadata-token", token)
    }
    let role_name = builder.send().await?.text().await?;

    // get metadata security credentials
    let builder = client.get(format!("{}/{}", url, role_name))
        .header("Content-Type", "application/json");
    if let(token) = temporary_session_token {
        builder.header("X-aws-ec2-metadata-token", token)
    }
    let cred : AWSSecurityCredentials= builder.send().await?.json().await?;
    Ok(cred)
}

async fn get_region(temporary_session_token: &Option<String>, url: &Option<String>) -> Result<String, Error> {
    if can_retrieve_region_from_environment() {
        if let Ok(region) = var("AWS_REGION") {
            return Ok(region)
        }
        return Ok(var("AWS_DEFAULT_REGION").unwrap())
    }
    //TODO error code
    let url = &url.ok_or(Error::NoCredentialsSource)?;
    let client = default_http_client();
    let builder = client.get(url);
    if let(token) = temporary_session_token {
        builder.header("X-aws-ec2-metadata-token", token)
    }
    let body = builder.send().await?.bytes().await?;

    // This endpoint will return the region in format: us-east-2b.
    // Only the us-east-2 part should be used.
    let resp_body_end = if !body.is_empty() {
        body.len() - 1
    }else {
        0
    };
    Ok(String::from_utf8_lossy(&body[0..resp_body_end]).to_string())
}

fn default_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build().unwrap()
}