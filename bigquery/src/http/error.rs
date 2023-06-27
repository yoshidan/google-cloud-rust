use std::fmt;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// An error returned from the Google Cloud Storage service.
    #[error(transparent)]
    Response(#[from] ErrorResponse),

    /// An error from the HTTP client.
    #[error(transparent)]
    HttpClient(#[from] reqwest::Error),

    /// An error from a token source.
    #[error("token source failed: {0}")]
    TokenSource(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    /// An HTTP status value, without the textual description.
    ///
    /// Example values include: `400` (Bad Request), `401` (Unauthorized), and `404` (Not Found).
    pub code: u16,

    /// Description of the error. Same as `errors.message`.
    pub message: String,
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for ErrorResponse {}

#[derive(serde::Deserialize)]
pub(crate) struct ErrorWrapper {
    pub(crate) error: ErrorResponse,
}
