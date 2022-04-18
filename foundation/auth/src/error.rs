use std::env::VarError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("scopes is required if the audience is none")]
    ScopeOrAudienceRequired,

    #[error("unsupported account {0}")]
    UnsupportedAccountType(String),

    #[error("refresh token is required for user account credentials")]
    RefreshTokenIsRequired,

    #[error(transparent)]
    JsonError(#[from] json::Error),

    #[error(transparent)]
    JwtError(#[from] jwt::errors::Error),

    #[error(transparent)]
    HyperError(#[from] hyper::Error),

    #[error(transparent)]
    HttpError(#[from] hyper::http::Error),

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    VarError(#[from] VarError),

    #[error("user home directory not found")]
    NoHomeDirectoryFound,

    #[error("Server responded with error status is {0}")]
    DeserializeError(String),

    #[error("Private Key is required")]
    NoPrivateKeyFound,

    #[error("No Credentials File Found")]
    NoCredentialsFileFound,
}
