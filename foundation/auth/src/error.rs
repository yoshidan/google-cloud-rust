use serde::Deserialize;

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

    #[error("http error on {0}: {1}")]
    HttpError(String, reqwest::Error),

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
    #[error("external account error : {0}")]
    ExternalAccountSource(#[from] crate::token_source::external_account_source::error::Error),

    #[error("unexpected impersonation token response : status={0}, detail={1}")]
    UnexpectedImpersonateTokenResponse(u16, String),

    #[error("No target_audience Found in the private claims")]
    NoTargetAudienceFound,

    #[error("Unexpected token response: status={status}, error={error}, description={error_description}")]
    TokenErrorResponse {
        status: u16,
        error: String,
        error_description: String,
    },
}

#[derive(Debug, Deserialize)]
pub(crate) struct TokenErrorResponse {
    pub(crate) error: String,
    pub(crate) error_description: String,
}
