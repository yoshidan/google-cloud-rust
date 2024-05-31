use url::ParseError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid Region URL: {0}")]
    InvalidRegionURL(String),

    #[error("Invalid Security Credentials URL: {0}")]
    InvalidSecurityCredentialsURL(String),

    #[error("Invalid imds v2 session token URL: {0}")]
    InvalidIMDSv2SessionTokenURL(String),

    #[error("No Credentials Source ")]
    NoCredentialsSource,

    #[error("AWS version {0} is not supported in the current build")]
    UnsupportedAWSVersion(String),

    #[error("Unsupported Subject Token Source")]
    UnsupportedSubjectTokenSource,

    #[error("Unsupported Format Type")]
    UnsupportedFormatType,

    #[error(transparent)]
    HttpError(#[from] reqwest::Error),

    #[error(transparent)]
    JsonError(#[from] serde_json::error::Error),

    #[error(transparent)]
    URLError(#[from] ParseError),

    #[error(transparent)]
    TimeFormatError(#[from] time::error::Format),

    #[error(transparent)]
    IoError(#[from] tokio::io::Error),

    #[error("Missing Region URL")]
    MissingRegionURL,

    #[error("Missing Security Credentials URL")]
    MissingSecurityCredentialsURL,

    #[error("Missing Regional Cred Verification URL")]
    MissingRegionalCredVerificationURL,

    #[error("Missing Token URL")]
    MissingTokenURL,

    #[error("Missing Subject Token Type")]
    MissingSubjectTokenType,

    #[error("Missing Headers")]
    MissingHeaders,

    #[error("Missing Format")]
    MissingFormat,

    #[error("Missing Subject Token Field Name")]
    MissingSubjectTokenFieldName,

    #[error(transparent)]
    InvalidHashLength(#[from] sha2::digest::InvalidLength),

    #[error("Failed to get role name. No IAM role may be attached to instance : status={0}")]
    UnexpectedStatusOnGetRoleName(u16),

    #[error("Failed to get session token : status={0}")]
    UnexpectedStatusOnGetSessionToken(u16),

    #[error("Failed to get credentials : status={0}")]
    UnexpectedStatusOnGetCredentials(u16),

    #[error("Failed to get region : status={0}")]
    UnexpectedStatusOnGetRegion(u16),

    #[error("Failed to get subject token: status={0}, detail={1}")]
    UnexpectedStatusOnGetSubjectToken(u16, String),
}
