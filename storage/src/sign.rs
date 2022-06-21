use crate::sign::SignedURLError::InvalidOption;
use chrono::{DateTime, SecondsFormat, Utc};
use once_cell::sync::Lazy;
use regex::Regex;
use ring::{rand, signature};
use rsa::pkcs8::{DecodePrivateKey, EncodePrivateKey};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::Duration;
use url;
use url::ParseError;

static SPACE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r" +").unwrap());
static TAB_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"[\t]+").unwrap());

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

#[derive(PartialEq)]
pub enum SigningScheme {
    /// V2 is deprecated. https://cloud.google.com/storage/docs/access-control/signed-urls?types#types
    /// SigningSchemeV2

    /// SigningSchemeV4 uses the V4 scheme to sign URLs.
    SigningSchemeV4,
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
        return format!("{}/{}", bucket, object);
    }
}

type SignBytes = Box<dyn Fn(&[u8]) -> Result<Vec<u8>, SignedURLError>>;

pub enum SignBy {
    PrivateKey(Vec<u8>),
    SignBytes(SignBytes),
}

/// SignedURLOptions allows you to restrict the access to the signed URL.
pub struct SignedURLOptions {
    /// GoogleAccessID represents the authorizer of the signed URL generation.
    /// It is typically the Google service account client email address from
    /// the Google Developers Console in the form of "xxx@developer.gserviceaccount.com".
    /// Required.
    pub google_access_id: String,

    /// PrivateKey is the Google service account private key. It is obtainable
    /// from the Google Developers Console.
    /// At https://console.developers.google.com/project/<your-project-id>/apiui/credential,
    /// create a service account client ID or reuse one of your existing service account
    /// credentials. Click on the "Generate new P12 key" to generate and download
    /// a new private key. Once you download the P12 file, use the following command
    /// to convert it into a PEM file.
    ///
    ///    $ openssl pkcs12 -in key.p12 -passin pass:notasecret -out key.pem -nodes
    ///
    /// Provide the contents of the PEM file as a byte slice.
    /// Exactly one of PrivateKey or SignBytes must be non-nil.
    ///
    /// SignBytes is a function for implementing custom signing.
    pub sign_by: SignBy,

    /// Method is the HTTP method to be used with the signed URL.
    /// Signed URLs can be used with GET, HEAD, PUT, and DELETE requests.
    /// Required.
    pub method: SignedURLMethod,

    /// Expires is the expiration time on the signed URL. It must be
    /// a datetime in the future. For SigningSchemeV4, the expiration may be no
    /// more than seven days in the future.
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
    pub style: Box<dyn URLStyle>,

    /// Insecure determines whether the signed URL should use HTTPS (default) or
    /// HTTP.
    /// Only supported for V4 signing.
    /// Optional.
    pub insecure: bool,

    /// Scheme determines the version of URL signing to use. Default is SigningSchemeV4.
    pub scheme: SigningScheme,
}

impl Default for SignedURLOptions {
    fn default() -> Self {
        Self {
            google_access_id: "".to_string(),
            sign_by: SignBy::PrivateKey(vec![]),
            method: SignedURLMethod::GET,
            expires: std::time::Duration::from_secs(600),
            content_type: None,
            headers: vec![],
            query_parameters: Default::default(),
            md5: None,
            style: Box::new(PathStyle {}),
            insecure: false,
            scheme: SigningScheme::SigningSchemeV4,
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
    CredentialError(#[from] google_cloud_auth::error::Error),
    #[error(transparent)]
    MetadataError(#[from] google_cloud_metadata::Error),
}

pub(crate) fn signed_url(bucket: &str, object: &str, opts: SignedURLOptions) -> Result<String, SignedURLError> {
    let now = Utc::now();
    let _ = validate_options(&opts, &now)?;

    match &opts.scheme {
        SigningScheme::SigningSchemeV4 => {
            let mut opts = opts;
            opts.headers = v4_sanitize_headers(&opts.headers);
            signed_url_v4(bucket, object, &opts, now)
        }
    }
}

fn v4_sanitize_headers(hdrs: &[String]) -> Vec<String> {
    let mut sanitized = HashMap::<String, Vec<String>>::new();
    for hdr in hdrs {
        let trimmed = hdr.trim().to_string();
        let split: Vec<&str> = trimmed.split(':').into_iter().collect();
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

fn signed_url_v4(
    bucket: &str,
    name: &str,
    opts: &SignedURLOptions,
    now: DateTime<Utc>,
) -> Result<String, SignedURLError> {
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
        let mut header_names = extract_header_names(&opts.headers);
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

    let timestamp = now
        .to_rfc3339_opts(SecondsFormat::Secs, true)
        .replace("-", "")
        .replace(":", "");
    let credential_scope = format!("{}/auto/storage/goog4_request", now.format("%Y%m%d"));

    // append query parameters
    {
        let mut query = builder.query_pairs_mut();
        query.append_pair("X-Goog-Algorithm", "GOOG4-RSA-SHA256");
        query.append_pair("X-Goog-Credential", &format!("{}/{}", opts.google_access_id, credential_scope));
        query.append_pair("X-Goog-Date", &timestamp);
        query.append_pair("X-Goog-Expires", opts.expires.as_secs().to_string().as_str());
        query.append_pair("X-Goog-SignedHeaders", &signed_headers);
        for (k, values) in &opts.query_parameters {
            for value in values {
                query.append_pair(k.as_str(), value.as_str());
            }
        }
    }
    let escaped_query = builder.query().unwrap().replace("+", "%20");
    tracing::trace!("escaped_query={}", escaped_query);

    // create header with value
    let header_with_value = {
        let mut header_with_value = vec![format!("host:{}", host)];
        header_with_value.extend_from_slice(&opts.headers);
        if let Some(content_type) = &opts.content_type {
            header_with_value.push(format!("content-type:{}", content_type))
        }
        if let Some(md5) = &opts.md5 {
            header_with_value.push(format!("content-md5:{}", md5))
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
            builder.path().replace("+", "%20"),
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
    let signed_buffer = {
        let hex_digest = hex::encode(Sha256::digest(buffer));
        let mut signed_buffer: Vec<u8> = vec![];
        signed_buffer.extend_from_slice("GOOG4-RSA-SHA256\n".as_bytes());
        signed_buffer.extend_from_slice(format!("{}\n", timestamp).as_bytes());
        signed_buffer.extend_from_slice(format!("{}\n", credential_scope).as_bytes());
        signed_buffer.extend_from_slice(hex_digest.as_bytes());
        signed_buffer
    };
    tracing::trace!("signed_buffer={:?}", String::from_utf8_lossy(&signed_buffer));

    // create signature
    let signature = match &opts.sign_by {
        SignBy::PrivateKey(private_key) => {
            let str = String::from_utf8_lossy(private_key);
            let pkcs = rsa::RsaPrivateKey::from_pkcs8_pem(str.as_ref())
                .map_err(|e| SignedURLError::CertError(e.to_string()))?;
            let der = pkcs
                .to_pkcs8_der()
                .map_err(|e| SignedURLError::CertError(e.to_string()))?;
            let key_pair = ring::signature::RsaKeyPair::from_pkcs8(der.as_ref())
                .map_err(|e| SignedURLError::CertError(e.to_string()))?;
            let mut signed = vec![0; key_pair.public_modulus_len()];
            key_pair
                .sign(
                    &signature::RSA_PKCS1_SHA256,
                    &rand::SystemRandom::new(),
                    signed_buffer.as_slice(),
                    &mut signed,
                )
                .map_err(|e| SignedURLError::CertError(e.to_string()))?;
            signed
        }
        SignBy::SignBytes(f) => f(signed_buffer.as_slice())?,
    };
    builder
        .query_pairs_mut()
        .append_pair("X-Goog-Signature", &hex::encode(signature));
    Ok(builder.to_string())
}

fn extract_header_names(kvs: &[String]) -> Vec<&str> {
    return kvs
        .iter()
        .map(|header| {
            let name_value: Vec<&str> = header.split(':').collect();
            name_value[0]
        })
        .collect();
}

fn validate_options(opts: &SignedURLOptions, _now: &DateTime<Utc>) -> Result<(), SignedURLError> {
    if opts.google_access_id.is_empty() {
        return Err(InvalidOption("storage: missing required GoogleAccessID"));
    }
    if opts.expires.is_zero() {
        return Err(InvalidOption("missing required expires option"));
    }
    if let Some(md5) = &opts.md5 {
        match base64::decode(md5) {
            Ok(v) => {
                if v.len() != 16 {
                    return Err(InvalidOption("storage: invalid MD5 checksum length"));
                }
            }
            Err(_e) => return Err(InvalidOption("storage: invalid MD5 checksum")),
        }
    }
    if opts.scheme == SigningScheme::SigningSchemeV4 && opts.expires > Duration::from_secs(604801) {
        return Err(InvalidOption("storage: expires must be within seven days from now"));
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::sign::{signed_url, SignBy, SignedURLOptions};

    use google_cloud_auth::credentials::CredentialsFile;
    use serial_test::serial;
    use std::collections::HashMap;
    use std::time::Duration;

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt::try_init();
    }

    #[tokio::test]
    #[serial]
    async fn signed_url_internal() {
        let file = CredentialsFile::new().await.unwrap();
        let param = {
            let mut param = HashMap::new();
            param.insert("tes t+".to_string(), vec!["++ +".to_string()]);
            param
        };
        let opts = SignedURLOptions {
            sign_by: SignBy::PrivateKey(file.private_key.unwrap().into()),
            google_access_id: file.client_email.unwrap(),
            expires: Duration::from_secs(3600),
            query_parameters: param,
            ..Default::default()
        };
        let url = signed_url("rust-object-test", "test1", opts).unwrap();
        println!("downloading={:?}", url);
        let result = reqwest::Client::default().get(url).send().await.unwrap();
        assert!(result.status().is_success());
    }
}
