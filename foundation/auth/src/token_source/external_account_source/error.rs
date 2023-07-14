use url::ParseError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid Region URL: {0}")]
    InvalidRegionURL(String),

    #[error("Invalid Cred Verification Region URL: {0}")]
    InvalidCredVerificationURL(String),

    #[error("Invalid imds v2 session token URL: {0}")]
    InvalidIMDSv2SessionTokenURL(String),

    #[error("No Credentials Source ")]
    NoCredentialsSource,

    #[error("aws version {0} is not supported in the current build")]
    UnsupportedAWSVersion(String),

    #[error("Unsupported Subject Token Source")]
    UnsupportedSubjectTokenSource,

    #[error(transparent)]
    HttpError(#[from] reqwest::Error),

    #[error(transparent)]
    JsonError(#[from] serde_json::error::Error),

    #[error(transparent)]
    URLError(#[from] ParseError),

    #[error(transparent)]
    TimeFormatError(#[from] time::error::Format),

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

    #[error(transparent)]
    InvalidHashLength(#[from] sha2::digest::InvalidLength),

    #[error("failed to get role name : status={0}")]
    UnexpectedStatusOnGetRoleName(u16),

    #[error("failed to get session token : status={0}")]
    UnexpectedStatusOnGetSessionToken(u16),

    #[error("failed to get credentials : status={0}")]
    UnexpectedStatusOnGetCredentials(u16),

    #[error("failed to get region  : status={0}")]
    UnexpectedStatusOnGetRegion(u16),

    #[error("failed to token : status={0}, detail={1}")]
    UnexpectedStatusOnToken(u16, String),
}
