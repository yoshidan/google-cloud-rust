use std::env::var;
use std::fmt::{Debug, Formatter};
use std::path::PathBuf;

use async_trait::async_trait;
use hmac::Mac;
use path_clean::PathClean;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::macros::format_description;
use time::OffsetDateTime;
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

impl AWSSubjectTokenSource {
    pub async fn new(audience: Option<String>, value: CredentialSource) -> Result<Self, Error> {
        if !validate_metadata_server(&value.region_url) {
            return Err(Error::InvalidRegionURL(value.region_url.unwrap_or_empty()));
        }
        // Not value.cred_verification_url but value.url
        if !validate_metadata_server(&value.url) {
            return Err(Error::InvalidSecurityCredentialsURL(value.url.unwrap_or_empty()));
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

    fn create_auth_header(
        &self,
        method: &str,
        now: &OffsetDateTime,
        headers: &[(&str, &str)],
    ) -> Result<String, Error> {
        let date_stamp_short = now.format(&format_description!("[year][month][day]"))?;
        let service_name: Vec<String> = self
            .subject_token_url
            .host_str()
            .unwrap_or_default()
            .split('.')
            .map(|v| v.to_string())
            .collect();
        let service_name = service_name[0].as_str();
        let credential_scope = format!("{}/{}/{}/{}", date_stamp_short, &self.region, service_name, AWS_REQUEST_TYPE);

        let (header_keys, header_values) = canonical_headers(headers);
        let query = self.subject_token_url.query().unwrap_or_default();
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
            method, path, query, header_values, header_keys, data_hash
        );
        let request_hash = hex::encode(Sha256::digest(request_string.as_bytes()));
        let date_stamp_long = now.format(&format_description!("[year][month][day]T[hour][minute][second]Z"))?;
        let string_to_sign = format!("{}\n{}\n{}\n{}", AWS_ALGORITHM, date_stamp_long, credential_scope, request_hash);

        // sign
        let mut signing_key = format!("AWS4{}", self.credentials.secret_access_key).into_bytes();
        for input in [
            date_stamp_short.as_str(),
            self.region.as_str(),
            service_name,
            AWS_REQUEST_TYPE,
            string_to_sign.as_str(),
        ] {
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

    fn create_subject_token(&self, now: OffsetDateTime) -> Result<String, Error> {
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
        let authorization = self.create_auth_header(method, &now, &sorted_headers)?;

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

#[async_trait]
impl SubjectTokenSource for AWSSubjectTokenSource {
    async fn subject_token(&self) -> Result<String, Error> {
        self.create_subject_token(OffsetDateTime::now_utc())
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct AWSSecurityCredentials {
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
        return Err(Error::UnexpectedStatusOnGetSessionToken(response.status().as_u16()));
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

    // get metadata role name
    let url = url.as_ref().ok_or(Error::MissingSecurityCredentialsURL)?;
    let client = default_http_client();
    let mut builder = client.get(url);
    if let Some(token) = temporary_session_token {
        builder = builder.header(AWS_IMDS_V2_SESSION_TOKEN_HEADER, token);
    }
    let response = builder.send().await?;
    if !response.status().is_success() {
        return Err(Error::UnexpectedStatusOnGetRoleName(response.status().as_u16()));
    }
    let role_name = response.text().await?;

    // get metadata security credentials
    let url = format!("{}/{}", url, role_name);
    let mut builder = client.get(url);
    if let Some(token) = temporary_session_token {
        builder = builder.header(AWS_IMDS_V2_SESSION_TOKEN_HEADER, token);
    }
    let response = builder.send().await?;
    if !response.status().is_success() {
        return Err(Error::UnexpectedStatusOnGetCredentials(response.status().as_u16()));
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
        return Err(Error::UnexpectedStatusOnGetRegion(response.status().as_u16()));
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
        full_headers.push(format!("{}:{}\n", header.0, header.1));
    }
    (keys.join(";"), full_headers.join(""))
}

#[cfg(test)]
mod tests {
    use time::macros::{datetime, format_description};
    use url::Url;

    use crate::credentials::CredentialsFile;
    use crate::token_source::external_account_source::aws_subject_token_source::{
        AWSSecurityCredentials, AWSSubjectTokenSource,
    };

    fn create_token_source() -> AWSSubjectTokenSource {
        let cred = r#"{
            "type": "external_account",
            "audience": "//iam.googleapis.com/projects/myprojectnumber/locations/global/workloadIdentityPools/aws-test/providers/aws-test",
            "subject_token_type": "urn:ietf:params:aws:token-type:aws4_request",
            "service_account_impersonation_url": "https://iamcredentials.googleapis.com/test",
            "token_url": "https://sts.googleapis.com/v1/token",
            "credential_source": {
                "environment_id": "aws1",
                "region_url": "http://169.254.169.254/latest/meta-data/placement/availability-zone",
                "url": "http://169.254.169.254/latest/meta-data/iam/security-credentials",
                "regional_cred_verification_url": "https://sts.{region}.amazonaws.com?Action=GetCallerIdentity&Version=2011-06-15"
            }
        }"#;
        let region = "ap-northeast-1b".to_string();
        let cred: CredentialsFile = serde_json::from_str(cred).unwrap();
        let url = cred.credential_source.unwrap().regional_cred_verification_url.unwrap();
        let subject_token_url = Url::parse(&url.replace("{region}", &region)).unwrap();

        AWSSubjectTokenSource {
            subject_token_url,
            target_resource: cred.audience,
            credentials: AWSSecurityCredentials {
                access_key_id: "AccessKeyId".to_string(),
                secret_access_key: "SecretAccessKey".to_string(),
                token: Some("SecurityToken".to_string()),
            },
            region,
        }
    }
    #[test]
    fn test_create_auth_header() {
        let source = create_token_source();
        let now = datetime!(2022-12-31 00:00:00).assume_utc();
        let format_date = now
            .format(&format_description!("[year][month][day]T[hour][minute][second]Z"))
            .unwrap();
        let sorted_headers: Vec<(&str, &str)> = vec![
            ("host", source.subject_token_url.host_str().unwrap_or("")),
            ("x-amz-date", &format_date),
            ("x-amz-security-token", source.credentials.token.as_ref().unwrap()),
            ("x-goog-cloud-target-resource", source.target_resource.as_ref().unwrap()),
        ];
        let actual = source.create_auth_header("POST", &now, &sorted_headers).unwrap();
        let expected = "AWS4-HMAC-SHA256 Credential=AccessKeyId/20221231/ap-northeast-1b/sts/aws4_request, SignedHeaders=host;x-amz-date;x-amz-security-token;x-goog-cloud-target-resource, Signature=168a40df8b7c11fb0588a13cada1443e31e4736de702232f9a2177b26edda21c";
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_create_subject_token() {
        let source = create_token_source();
        let now = datetime!(2022-12-31 00:00:00).assume_utc();
        match source.create_subject_token(now) {
            Ok(token) => {
                let expected = "%7B%22url%22%3A%22https%3A%2F%2Fsts%2Eap%2Dnortheast%2D1b%2Eamazonaws%2Ecom%2F%3FAction%3DGetCallerIdentity%26Version%3D2011%2D06%2D15%22%2C%22method%22%3A%22POST%22%2C%22headers%22%3A%5B%7B%22key%22%3A%22Authorization%22%2C%22value%22%3A%22AWS4%2DHMAC%2DSHA256%20Credential%3DAccessKeyId%2F20221231%2Fap%2Dnortheast%2D1b%2Fsts%2Faws4%5Frequest%2C%20SignedHeaders%3Dhost%3Bx%2Damz%2Ddate%3Bx%2Damz%2Dsecurity%2Dtoken%3Bx%2Dgoog%2Dcloud%2Dtarget%2Dresource%2C%20Signature%3D168a40df8b7c11fb0588a13cada1443e31e4736de702232f9a2177b26edda21c%22%7D%2C%7B%22key%22%3A%22host%22%2C%22value%22%3A%22sts%2Eap%2Dnortheast%2D1b%2Eamazonaws%2Ecom%22%7D%2C%7B%22key%22%3A%22x%2Damz%2Ddate%22%2C%22value%22%3A%2220221231T000000Z%22%7D%2C%7B%22key%22%3A%22x%2Damz%2Dsecurity%2Dtoken%22%2C%22value%22%3A%22SecurityToken%22%7D%2C%7B%22key%22%3A%22x%2Dgoog%2Dcloud%2Dtarget%2Dresource%22%2C%22value%22%3A%22%2F%2Fiam%2Egoogleapis%2Ecom%2Fprojects%2Fmyprojectnumber%2Flocations%2Fglobal%2FworkloadIdentityPools%2Faws%2Dtest%2Fproviders%2Faws%2Dtest%22%7D%5D%7D";
                assert_eq!(token, expected);
            }
            Err(err) => {
                tracing::error!("error={},{:?}", err, err);
                unreachable!();
            }
        }
    }
}
