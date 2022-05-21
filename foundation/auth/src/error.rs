#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("scopes is required if the audience is none")]
    ScopeOrAudienceRequired,

    #[error("unsupported account {0}")]
    UnsupportedAccountType(String),

    #[error("refresh token is required for user account credentials")]
    RefreshTokenIsRequired,

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

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
