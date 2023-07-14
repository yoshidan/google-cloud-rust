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

    #[error("get role name error : status={0}")]
    UnexpectedStatusOnGetRoleName(u16),

    #[error("get session token error : status={0}")]
    UnexpectedStatusOnGetSessionToken(u16),

    #[error("get credentials error : status={0}")]
    UnexpectedStatusOnGetCredentials(u16),

    #[error("get region error : status={0}")]
    UnexpectedStatusOnGetRegion(u16),

    #[error("token : status={0}")]
    UnexpectedStatusOnToken(u16),
}
