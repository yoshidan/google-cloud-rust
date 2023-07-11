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

    #[error("Missing External Token URL")]
    MissingExternalTokenURL,

    #[error("Missing Subject Token Type")]
    MissingSubjectTokenType,

    #[error(transparent)]
    InvalidHashLength(#[from] sha2::digest::InvalidLength),
}
