use crate::bucket::SignedURLError::InvalidOption;
use crate::util;
use chrono::{DateTime, SecondsFormat, Timelike, Utc};
use once_cell::sync::Lazy;
use regex::Regex;
use ring::{rand, signature};
use rsa::pkcs1::der::Document;
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::pkcs8::{DecodePrivateKey, EncodePrivateKey};
use rsa::{PaddingScheme, PublicKeyParts};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt::format;
use std::iter::Map;
use std::ops::{Add, Index, Sub};
use std::time::Duration;
use url;

static SPACE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r" +").unwrap());
static TAB_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"[\t]+").unwrap());
const SIGNED_URL_METHODS: [&str; 5] = ["DELETE", "GET", "HEAD", "POST", "PUT"];

pub struct BucketHandle {
    name: String,
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

const HOST: &str = "sorage.googleapis.com";

impl URLStyle for PathStyle {
    fn host(&self, _bucket: &str) -> String {
        match std::env::var("STORAGE_EMULATOR_HOST") {
            Ok(host) => {
                if host.contains("://") {
                    let v: Vec<&str> = host.splitn(2, "://").collect();
                    v[1].to_string()
                } else {
                    host.to_string()
                }
            }
            Err(_e) => HOST.to_string(),
        }
    }

    fn path(&self, bucket: &str, object: &str) -> String {
        if object.is_empty() {
            return bucket.to_string();
        }
        return format!("{}/{}", bucket, object);
    }
}

/// SignedURLOptions allows you to restrict the access to the signed URL.
pub struct SignedURLOptions {
    /// GoogleAccessID represents the authorizer of the signed URL generation.
    /// It is typically the Google service account client email address from
    /// the Google Developers Console in the form of "xxx@developer.gserviceaccount.com".
    /// Required.
    google_access_id: String,

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
    private_key: Vec<u8>,

    /// SignBytes is a function for implementing custom signing. For example, if
    /// your application is running on Google App Engine, you can use
    /// appengine's internal signing function:
    ///     ctx := appengine.NewContext(request)
    ///     acc, _ := appengine.ServiceAccount(ctx)
    ///     url, err := SignedURL("bucket", "object", &SignedURLOptions{
    ///     	GoogleAccessID: acc,
    ///     	SignBytes: func(b []byte) ([]byte, error) {
    ///     		_, signedBytes, err := appengine.SignBytes(ctx, b)
    ///     		return signedBytes, err
    ///     	},
    ///     	// etc.
    ///     })
    ///
    /// Exactly one of PrivateKey or SignBytes must be non-nil.
    sign_bytes: Option<Box<dyn Fn(&[u8]) -> Result<Vec<u8>, SignedURLError>>>,

    /// Method is the HTTP method to be used with the signed URL.
    /// Signed URLs can be used with GET, HEAD, PUT, and DELETE requests.
    /// Required.
    method: String,

    /// Expires is the expiration time on the signed URL. It must be
    /// a datetime in the future. For SigningSchemeV4, the expiration may be no
    /// more than seven days in the future.
    /// Required.
    expires: std::time::Duration,

    /// ContentType is the content type header the client must provide
    /// to use the generated signed URL.
    /// Optional.
    content_type: String,

    /// Headers is a list of extension headers the client must provide
    /// in order to use the generated signed URL. Each must be a string of the
    /// form "key:values", with multiple values separated by a semicolon.
    /// Optional.
    headers: Vec<String>,

    /// QueryParameters is a map of additional query parameters. When
    /// SigningScheme is V4, this is used in computing the signature, and the
    /// client must use the same query parameters when using the generated signed
    /// URL.
    /// Optional.
    query_parameters: HashMap<String, Vec<String>>,

    /// MD5 is the base64 encoded MD5 checksum of the file.
    /// If provided, the client should provide the exact value on the request
    /// header in order to use the signed URL.
    /// Optional.
    md5: String,

    /// Style provides options for the type of URL to use. Options are
    /// PathStyle (default), BucketBoundHostname, and VirtualHostedStyle. See
    /// https://cloud.google.com/storage/docs/request-endpoints for details.
    /// Only supported for V4 signing.
    /// Optional.
    style: Box<dyn URLStyle>,

    /// Insecure determines whether the signed URL should use HTTPS (default) or
    /// HTTP.
    /// Only supported for V4 signing.
    /// Optional.
    insecure: bool,

    // Scheme determines the version of URL signing to use. Default is
    // SigningSchemeV2.
    scheme: SigningScheme,
}

#[derive(thiserror::Error, Debug)]
pub enum SignedURLError {
    #[error("invalid option {0}")]
    InvalidOption(&'static str),
}

impl BucketHandle {
    pub fn signed_url(object: String, opts: &SignedURLOptions) -> Result<String, SignedURLError> {
        //TODO
        Ok("".to_string())
    }
}

pub fn signed_url<F, U>(name: String, object: String, opts: &SignedURLOptions) -> Result<String, SignedURLError>
where
    U: URLStyle,
{
    let now = Utc::now();
    let _ = validate_options(opts, &now)?;

    //TODO
    Ok("".to_string())
}

fn v4_sanitize_headers(hdrs: &[String]) -> Vec<String> {
    let mut sanitized = HashMap::<String, Vec<String>>::new();
    for hdr in hdrs {
        let trimmed = hdr.trim().to_string();
        let split: Vec<&str> = trimmed.split(":").into_iter().collect();
        if split.len() < 2 {
            continue;
        }
        let key = split[0].trim().to_lowercase();
        let space_removed = SPACE_REGEX.replace_all(split[1].trim(), " ");
        let value = TAB_REGEX.replace_all(space_removed.as_ref(), "\t");
        if !value.is_empty() {
            if sanitized.contains_key(&key) {
                sanitized.get_mut(&key).unwrap().push(value.to_string());
            } else {
                sanitized.insert(key, vec![value.to_string()]);
            }
        }
    }
    let mut sanitized_headers = Vec::with_capacity(sanitized.len());
    let mut index = 0;
    for (key, value) in sanitized {
        sanitized_headers[index] = format!("{}:{}", key, value.join(",").to_string());
        index += 1;
    }
    sanitized_headers
}

fn signed_url_v4(
    bucket: &str,
    name: &str,
    opts: &SignedURLOptions,
    now: DateTime<Utc>,
) -> Result<String, SignedURLError> {
    let mut buffer: Vec<u8> = vec![];
    buffer.extend_from_slice(format!("{}\n", opts.method).as_bytes());

    let path = opts.style.path(bucket, name);
    let mut ui = url::Url::parse("https://storage.google.com").unwrap();
    let raw_path = path_encode_v4(&path);
    buffer.extend_from_slice(format!("/{}\n", raw_path).as_bytes());

    let mut header_names = extract_header_names(&opts.headers);
    header_names.push("host");
    if !opts.content_type.is_empty() {
        header_names.push("content-type");
    }
    if !opts.md5.is_empty() {
        header_names.push("content-md5");
    }
    header_names.sort();

    let signed_headers = header_names.join(";");
    let timestamp = now.to_rfc3339_opts(SecondsFormat::Secs, true);
    let credential_scope = format!("{}/auto/storage/goog4_request", now.format("%Y%m%d"));
    let mut canonical_query_string = util::QueryParam::new();
    canonical_query_string.adds("X-Goog-Algorithm".to_string(), vec!["GOOG4-RSA-SHA256".to_string()]);
    canonical_query_string.adds(
        "X-Goog-Credential".to_string(),
        vec![format!("{}/{}", opts.google_access_id, credential_scope)],
    );
    canonical_query_string.adds("X-Goog-Date".to_string(), vec![timestamp.clone()]);
    canonical_query_string.adds("X-Goog-Expires".to_string(), vec![opts.expires.as_secs().to_string()]);
    canonical_query_string.adds("X-Goog-SignedHeaders".to_string(), vec![signed_headers.clone()]);
    for (k, v) in &opts.query_parameters {
        canonical_query_string.adds(k.clone(), v.clone())
    }
    let escaped_query = canonical_query_string.encode().replace("+", "%20");
    println!("escap={}", escaped_query);
    buffer.extend_from_slice(format!("/{}\n", escaped_query).as_bytes());

    let host = opts.style.host(bucket).to_string();
    if opts.insecure {
        ui.set_scheme("http");
    }
    ui.set_path(&path);
    ui.set_host(Some(host.as_str()));

    let mut header_with_value = vec![format!("host:{}", host)];
    header_with_value.extend_from_slice(&opts.headers);
    if !opts.content_type.is_empty() {
        header_with_value.push(format!("content-type:{}", opts.content_type))
    }
    if !opts.md5.is_empty() {
        header_with_value.push(format!("content-md5:{}", opts.md5))
    }
    header_with_value.sort();
    let canonical_headers = header_with_value.join(" ");
    buffer.extend_from_slice(format!("{}\n\n", canonical_headers).as_bytes());
    buffer.extend_from_slice(format!("{}\n", signed_headers).as_bytes());

    /// If the user provides a value for X-Goog-Content-SHA256, we must use
    /// that value in the request string. If not, we use UNSIGNED-PAYLOAD.
    let sha256_header = header_with_value
        .iter()
        .find(|h| {
            let ret = h.to_lowercase().starts_with("x-goog-content-sha256") && h.contains(":");
            if ret {
                let v: Vec<&str> = h.splitn(2, ":").collect();
                buffer.extend_from_slice(v[1].as_bytes());
            }
            ret
        })
        .is_some();
    if !sha256_header {
        buffer.extend_from_slice("UNSIGNED-PAYLOAD".as_bytes());
    }
    let hex_digest = Sha256::digest(buffer);
    let mut signed_buffer: Vec<u8> = vec![];
    signed_buffer.extend_from_slice("GOOG4-RSA-SHA256\n".as_bytes());
    signed_buffer.extend_from_slice(format!("{}\n", timestamp).as_bytes());
    signed_buffer.extend_from_slice(format!("{}\n", credential_scope).as_bytes());
    signed_buffer.extend_from_slice(hex_digest.as_slice());

    if !opts.private_key.is_empty() {
        let str = String::from_utf8(opts.private_key.clone()).unwrap();
        let pkcs = rsa::RsaPrivateKey::from_pkcs8_pem(&str).unwrap();
        let der = pkcs.to_pkcs8_der().unwrap();
        let key_pair = ring::signature::RsaKeyPair::from_pkcs8(der.as_ref()).unwrap();
        let mut signed = vec![0; key_pair.public_modulus_len()];
        key_pair
            .sign(
                &signature::RSA_PKCS1_SHA256,
                &rand::SystemRandom::new(),
                signed_buffer.as_slice(),
                &mut signed,
            )
            .unwrap();
        canonical_query_string.adds("X-Goog-Signature".to_string(), vec![hex::encode(signed)]);
    } else {
        let f = opts.sign_bytes.as_ref().unwrap();
        let signed = f(signed_buffer.as_slice()).unwrap();
        canonical_query_string.adds("X-Goog-Signature".to_string(), vec![hex::encode(signed)]);
    }
    for (k,v) in &canonical_query_string.inner {
        for v1 in v {
            ui.query_pairs_mut().append_pair(k, v1);
        }
    }
    println!("query={}",ui.query().unwrap().replace("+","%20"));
    Ok(ui.to_string())
}

fn path_encode_v4(path: &str) -> String {
    let segments: Vec<&str> = path.split("/").collect();
    let mut encoded_segments = Vec::with_capacity(segments.len());
    for segment in segments {
        encoded_segments.push(urlencoding::encode(segment).to_string());
    }
    let encoded_str = encoded_segments.join("/");
    return encoded_str.replace("+", "%20");
}

fn extract_header_names(kvs: &[String]) -> Vec<&str> {
    let mut res = vec![];
    for header in kvs {
        let name_value: Vec<&str> = header.split(":").collect();
        res.push(name_value[0])
    }
    res
}

fn validate_options(opts: &SignedURLOptions, now: &DateTime<Utc>) -> Result<(), SignedURLError> {
    if opts.google_access_id.is_empty() {
        return Err(InvalidOption("storage: missing required GoogleAccessID"));
    }
    if opts.private_key.is_empty() && opts.sign_bytes.is_none() {
        return Err(InvalidOption("storage: exactly one of PrivateKey or SignedBytes must be set"));
    }
    if !SIGNED_URL_METHODS.contains(&opts.method.to_uppercase().as_str()) {
        return Err(InvalidOption("storage: invalid HTTP method"));
    }
    if opts.expires.is_zero() {
        return Err(InvalidOption("missing required expires option"));
    }
    if !opts.md5.is_empty() {
        match base64::decode(&opts.md5) {
            Ok(v) => {
                if v.len() != 16 {
                    return Err(InvalidOption("storage: invalid MD5 checksum length"));
                }
            }
            Err(_e) => return Err(InvalidOption("storage: invalid MD5 checksum")),
        }
    }
    if opts.scheme == SigningScheme::SigningSchemeV4 {
        if opts.expires > Duration::from_secs(604801) {
            return Err(InvalidOption("storage: expires must be within seven days from now"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::bucket::{PathStyle, SignedURLOptions, SigningScheme};
    use chrono::{DateTime, Utc};
    use serial_test::serial;
    use std::collections::HashMap;
    use std::time::Duration;

    #[tokio::test]
    #[serial]
    async fn signed_url() {
        let cred = google_cloud_auth::get_credentials().await.unwrap();
        let mut param = HashMap::new();
        param.insert("tes t+".to_string(), vec!["++ +".to_string()]);
        let file =cred.file.unwrap();
        let opts = SignedURLOptions {
            google_access_id: file.client_email.unwrap().to_string(),
            private_key: file.private_key.unwrap().into(),
            sign_bytes: None,
            method: "".to_string(),
            expires: Duration::from_secs(86400),
            content_type: "".to_string(),
            headers: vec![],
            query_parameters: param,
            md5: "".to_string(),
            style: Box::new(PathStyle {}),
            insecure: false,
            scheme: SigningScheme::SigningSchemeV4,
        };
        let url = crate::bucket::signed_url_v4("bucket", "test.txt?日本語=TXT あいうえ+a", &opts, chrono::Utc::now()).unwrap();
        println!("url={}", url);
    }
}
