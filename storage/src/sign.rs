use std::collections::{BTreeMap, HashMap};
use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use std::time::{Duration, SystemTime};

use base64::prelude::*;
use once_cell::sync::Lazy;
use pkcs8::der::pem::PemLabel;
use pkcs8::SecretDocument;
use regex::Regex;
use sha2::{Digest, Sha256};
use time::format_description::well_known::iso8601::{EncodedConfig, TimePrecision};
use time::format_description::well_known::{self, Iso8601};
use time::macros::format_description;
use time::OffsetDateTime;
use url;
use url::{ParseError, Url};

use crate::http;
use crate::sign::SignedURLError::InvalidOption;

static SPACE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r" +").unwrap());
static TAB_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"[\t]+").unwrap());
static ONE_WEEK_IN_SECONDS: u64 = 604801;

pub enum SignedURLMethod {
    DELETE,
    GET,
    HEAD,
    POST,
    PUT,
}

impl SignedURLMethod {
    pub fn as_str(&self) -> &str {
        match self {
            SignedURLMethod::DELETE => "DELETE",
            SignedURLMethod::GET => "GET",
            SignedURLMethod::HEAD => "HEAD",
            SignedURLMethod::POST => "POST",
            SignedURLMethod::PUT => "PUT",
        }
    }
}

pub trait URLStyle {
    fn host(&self, bucket: &str) -> String;
    fn path(&self, bucket: &str, object: &str) -> String;
}

pub struct PathStyle {}

const HOST: &str = "storage.googleapis.com";

impl URLStyle for PathStyle {
    fn host(&self, _bucket: &str) -> String {
        //TODO emulator support
        HOST.to_string()
    }

    fn path(&self, bucket: &str, object: &str) -> String {
        if object.is_empty() {
            return bucket.to_string();
        }
        format!("{bucket}/{object}")
    }
}

#[derive(Clone)]
pub enum SignBy {
    PrivateKey(Vec<u8>),
    SignBytes,
}

impl Debug for SignBy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SignBy::PrivateKey(_) => f.write_str("private_key"),
            SignBy::SignBytes => f.write_str("sign_bytes"),
        }
    }
}

/// SignedURLOptions allows you to restrict the access to the signed URL.
pub struct SignedURLOptions {
    /// Method is the HTTP method to be used with the signed URL.
    /// Signed URLs can be used with GET, HEAD, PUT, and DELETE requests.
    /// Required.
    pub method: SignedURLMethod,

    /// StartTime is the time at which the signed URL starts being valid.
    /// Defaults to the current time.
    /// Optional.
    pub start_time: Option<std::time::SystemTime>,

    /// Expires is the duration of time, beginning at StartTime, within which
    /// the signed URL is valid. For SigningSchemeV4, the duration may be no
    /// more than 604800 seconds (7 days).
    /// Required.
    pub expires: std::time::Duration,

    /// ContentType is the content type header the client must provide
    /// to use the generated signed URL.
    /// Optional.
    pub content_type: Option<String>,

    /// Headers is a list of extension headers the client must provide
    /// in order to use the generated signed URL. Each must be a string of the
    /// form "key:values", with multiple values separated by a semicolon.
    /// Optional.
    pub headers: Vec<String>,

    /// QueryParameters is a map of additional query parameters. When
    /// SigningScheme is V4, this is used in computing the signature, and the
    /// client must use the same query parameters when using the generated signed
    /// URL.
    /// Optional.
    pub query_parameters: HashMap<String, Vec<String>>,

    /// MD5 is the base64 encoded MD5 checksum of the file.
    /// If provided, the client should provide the exact value on the request
    /// header in order to use the signed URL.
    /// Optional.
    pub md5: Option<String>,

    /// Style provides options for the type of URL to use. Options are
    /// PathStyle (default), BucketBoundHostname, and VirtualHostedStyle. See
    /// https://cloud.google.com/storage/docs/request-endpoints for details.
    /// Only supported for V4 signing.
    /// Optional.
    pub style: Box<dyn URLStyle + Send + Sync>,

    /// Insecure determines whether the signed URL should use HTTPS (default) or
    /// HTTP.
    /// Only supported for V4 signing.
    /// Optional.
    pub insecure: bool,
}

impl Default for SignedURLOptions {
    fn default() -> Self {
        Self {
            method: SignedURLMethod::GET,
            start_time: None,
            expires: std::time::Duration::from_secs(600),
            content_type: None,
            headers: vec![],
            query_parameters: Default::default(),
            md5: None,
            style: Box::new(PathStyle {}),
            insecure: false,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SignedURLError {
    #[error("invalid option {0}")]
    InvalidOption(&'static str),
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error("cert error by: {0}")]
    CertError(String),
    #[error(transparent)]
    SignBlob(#[from] http::Error),
}

pub(crate) fn create_signed_buffer(
    bucket: &str,
    name: &str,
    google_access_id: &str,
    opts: &SignedURLOptions,
) -> Result<(Vec<u8>, Url), SignedURLError> {
    validate_options(opts)?;
    let start_time: OffsetDateTime = opts.start_time.unwrap_or_else(SystemTime::now).into();

    let headers = v4_sanitize_headers(&opts.headers);
    // create base url
    let host = opts.style.host(bucket);
    let mut builder = {
        let url = if opts.insecure {
            format!("http://{}", &host)
        } else {
            format!("https://{}", &host)
        };
        url::Url::parse(&url)
    }?;

    // create signed headers
    let signed_headers = {
        let mut header_names = extract_header_names(&headers);
        header_names.push("host");
        if opts.content_type.is_some() {
            header_names.push("content-type");
        }
        if opts.md5.is_some() {
            header_names.push("content-md5");
        }
        header_names.sort_unstable();
        header_names.join(";")
    };

    const CONFIG: EncodedConfig = well_known::iso8601::Config::DEFAULT
        .set_use_separators(false)
        .set_time_precision(TimePrecision::Second { decimal_digits: None })
        .encode();

    let timestamp = start_time.format(&Iso8601::<CONFIG>).unwrap();
    let credential_scope = format!(
        "{}/auto/storage/goog4_request",
        start_time.format(format_description!("[year][month][day]")).unwrap()
    );

    // append query parameters
    {
        let mut query_parameters = [
            ("X-Goog-Algorithm", "GOOG4-RSA-SHA256"),
            ("X-Goog-Credential", &format!("{}/{}", google_access_id, credential_scope)),
            ("X-Goog-Date", &timestamp),
            ("X-Goog-Expires", opts.expires.as_secs().to_string().as_str()),
            ("X-Goog-SignedHeaders", &signed_headers),
        ]
        .into_iter()
        .map(|(key, value)| (key.to_owned(), vec![value.to_owned()]))
        .collect::<BTreeMap<_, _>>();
        query_parameters.extend(opts.query_parameters.clone());

        let mut query = builder.query_pairs_mut();
        for (k, values) in &query_parameters {
            for value in values {
                query.append_pair(k.as_str(), value.as_str());
            }
        }
    }
    let escaped_query = builder.query().unwrap().replace('+', "%20");
    tracing::trace!("escaped_query={}", escaped_query);

    // create header with value
    let header_with_value = {
        let mut header_with_value = vec![format!("host:{host}")];
        header_with_value.extend_from_slice(&headers);
        if let Some(content_type) = &opts.content_type {
            header_with_value.push(format!("content-type:{content_type}"))
        }
        if let Some(md5) = &opts.md5 {
            header_with_value.push(format!("content-md5:{md5}"))
        }
        header_with_value.sort();
        header_with_value
    };
    let path = opts.style.path(bucket, name);
    builder.set_path(&path);

    // create raw buffer
    let buffer = {
        let mut buffer = format!(
            "{}\n{}\n{}\n{}\n\n{}\n",
            opts.method.as_str(),
            builder.path().replace('+', "%20"),
            escaped_query,
            header_with_value.join("\n"),
            signed_headers
        )
        .into_bytes();

        // If the user provides a value for X-Goog-Content-SHA256, we must use
        // that value in the request string. If not, we use UNSIGNED-PAYLOAD.
        let sha256_header = header_with_value.iter().any(|h| {
            let ret = h.to_lowercase().starts_with("x-goog-content-sha256") && h.contains(':');
            if ret {
                let v: Vec<&str> = h.splitn(2, ':').collect();
                buffer.extend_from_slice(v[1].as_bytes());
            }
            ret
        });
        if !sha256_header {
            buffer.extend_from_slice("UNSIGNED-PAYLOAD".as_bytes());
        }
        buffer
    };
    tracing::trace!("raw_buffer={:?}", String::from_utf8_lossy(&buffer));

    // create signed buffer
    let hex_digest = hex::encode(Sha256::digest(buffer));
    let mut signed_buffer: Vec<u8> = vec![];
    signed_buffer.extend_from_slice("GOOG4-RSA-SHA256\n".as_bytes());
    signed_buffer.extend_from_slice(format!("{timestamp}\n").as_bytes());
    signed_buffer.extend_from_slice(format!("{credential_scope}\n").as_bytes());
    signed_buffer.extend_from_slice(hex_digest.as_bytes());
    Ok((signed_buffer, builder))
}

fn v4_sanitize_headers(hdrs: &[String]) -> Vec<String> {
    let mut sanitized = HashMap::<String, Vec<String>>::new();
    for hdr in hdrs {
        let trimmed = hdr.trim().to_string();
        let split: Vec<&str> = trimmed.split(':').collect();
        if split.len() < 2 {
            continue;
        }
        let key = split[0].trim().to_lowercase();
        let space_removed = SPACE_REGEX.replace_all(split[1].trim(), " ");
        let value = TAB_REGEX.replace_all(space_removed.as_ref(), "\t");
        if !value.is_empty() {
            sanitized.entry(key).or_default().push(value.to_string());
        }
    }
    let mut sanitized_headers = Vec::with_capacity(sanitized.len());
    for (key, value) in sanitized {
        sanitized_headers.push(format!("{}:{}", key, value.join(",")));
    }
    sanitized_headers
}

fn extract_header_names(kvs: &[String]) -> Vec<&str> {
    kvs.iter()
        .map(|header| {
            let name_value: Vec<&str> = header.split(':').collect();
            name_value[0]
        })
        .collect()
}

fn validate_options(opts: &SignedURLOptions) -> Result<(), SignedURLError> {
    if opts.expires.is_zero() {
        return Err(InvalidOption("storage: expires cannot be zero"));
    }
    if let Some(md5) = &opts.md5 {
        match BASE64_STANDARD.decode(md5) {
            Ok(v) => {
                if v.len() != 16 {
                    return Err(InvalidOption("storage: invalid MD5 checksum length"));
                }
            }
            Err(_e) => return Err(InvalidOption("storage: invalid MD5 checksum")),
        }
    }
    if opts.expires > Duration::from_secs(ONE_WEEK_IN_SECONDS) {
        return Err(InvalidOption("storage: expires must be within seven days from now"));
    }
    Ok(())
}

pub struct RsaKeyPair {
    inner: ring::signature::RsaKeyPair,
}

impl PemLabel for RsaKeyPair {
    const PEM_LABEL: &'static str = "PRIVATE KEY";
}

impl TryFrom<&Vec<u8>> for RsaKeyPair {
    type Error = SignedURLError;

    fn try_from(pem: &Vec<u8>) -> Result<Self, Self::Error> {
        let str = String::from_utf8_lossy(pem);
        let (label, doc) = SecretDocument::from_pem(&str).map_err(|v| SignedURLError::CertError(v.to_string()))?;
        Self::validate_pem_label(label).map_err(|_| SignedURLError::CertError(label.to_string()))?;
        let key_pair = ring::signature::RsaKeyPair::from_pkcs8(doc.as_bytes())
            .map_err(|e| SignedURLError::CertError(e.to_string()))?;
        Ok(Self { inner: key_pair })
    }
}

impl Deref for RsaKeyPair {
    type Target = ring::signature::RsaKeyPair;

    fn deref(&self) -> &ring::signature::RsaKeyPair {
        &self.inner
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::time::Duration;

    use serial_test::serial;

    use crate::http::storage_client::test::bucket_name;
    use google_cloud_auth::credentials::CredentialsFile;

    use crate::sign::{create_signed_buffer, SignedURLOptions};

    #[tokio::test]
    #[serial]
    async fn create_signed_buffer_test() {
        let file = CredentialsFile::new().await.unwrap();
        let param = {
            let mut param = HashMap::new();
            param.insert("tes t+".to_string(), vec!["++ +".to_string()]);
            param
        };
        let google_access_id = file.client_email.unwrap();
        let opts = SignedURLOptions {
            expires: Duration::from_secs(3600),
            query_parameters: param,
            ..Default::default()
        };
        let (signed_buffer, _builder) = create_signed_buffer(
            &bucket_name(&file.project_id.unwrap(), "object"),
            "test1",
            &google_access_id,
            &opts,
        )
        .unwrap();
        assert_eq!(signed_buffer.len(), 134)
    }
}
