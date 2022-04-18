use crate::bucket::SignedURLError::InvalidOption;
use chrono::{DateTime, Utc};
use std::iter::Map;
use std::ops::Add;
use std::time::Duration;

pub struct BucketHandle {
    name: String,
}

const signed_url_methods: [&str; 5] = ["DELETE", "GET", "HEAD", "POST", "PUT"];

pub enum SigningScheme {
    /// V2 is deprecated. https://cloud.google.com/storage/docs/access-control/signed-urls?types#types
    /// SigningSchemeV2

    /// SigningSchemeV4 uses the V4 scheme to sign URLs.
    SigningSchemeV4,
}

/// SignedURLOptions allows you to restrict the access to the signed URL.
pub struct SignedURLOptions<F>
where
    F: Fn(&[u8]) -> Result<Vec<u8>, SignedURLError>,
{
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
    sign_bytes: Option<F>,

    /// Method is the HTTP method to be used with the signed URL.
    /// Signed URLs can be used with GET, HEAD, PUT, and DELETE requests.
    /// Required.
    method: String,

    /// Expires is the expiration time on the signed URL. It must be
    /// a datetime in the future. For SigningSchemeV4, the expiration may be no
    /// more than seven days in the future.
    /// Required.
    expires: DateTime<Utc>,

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
    query_parameters: Map<String, Vec<String>>,

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
    ///Style URLStyle

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
    pub fn signed_url<F>(object: String, opts: &SignedURLOptions<F>) -> Result<String, SignedURLError> {
        //TODO
        Ok("".to_string())
    }
}

pub fn signed_url<F>(name: String, object: String, opts: &SignedURLOptions<F>) -> Result<String, SignedURLError> {
    let now = Utc::now();
    let _ = validate_options(opts, &now)?;

    //TODO
    Ok("".to_string())
}

fn validate_options<F>(opts: &SignedURLOptions<F>, now: &DateTime<Utc>) -> Result<(), SignedURLError> {
    if opts.google_access_id.is_empty() {
        return Err(InvalidOption("storage: missing required GoogleAccessID"));
    }
    if opts.private_key.is_empty() && opts.sign_bytes.is_none() {
        return Err(InvalidOption("storage: exactly one of PrivateKey or SignedBytes must be set"));
    }
    if !signed_url_methods.contains(&opts.method.as_str()) {
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
        let cutoff = now.add(Duration::from_secs(604801));
        if !opts.expires.lt(cutoff) {
            return Err(InvalidOption("storage: expires must be within seven days from now"));
        }
    }
    Ok(())
}
