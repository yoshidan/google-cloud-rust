#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("scopes is required if the audience is none")]
    ScopeOrAudienceRequired,

    #[error("unsupported account {0}")]
    UnsupportedAccountType(String),

    #[error("refresh token is required for user account credentials")]
    RefreshTokenIsRequired,

    #[error("json error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("jwt error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),

    #[error("http error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("GOOGLE_APPLICATION_CREDENTIALS or default credentials is required: {0}")]
    CredentialsIOError(#[from] std::io::Error),

    #[error("os env error: {0}")]
    VarError(#[from] std::env::VarError),

    #[error("user home directory not found")]
    NoHomeDirectoryFound,

    #[error("Server responded with error status is {0}")]
    DeserializeError(String),

    #[error("Private Key is required")]
    NoPrivateKeyFound,

    #[error("No Credentials File Found")]
    NoCredentialsFileFound,

    #[error("invalid authentication token")]
    InvalidToken,

    #[error(transparent)]
    TimeParse(#[from] time::error::Parse),

    #[cfg(feature = "external-account")]
    #[error(transparent)]
    ExternalAccountSource(#[from] crate::token_source::external_account_source::error::Error),
}
