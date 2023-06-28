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

    /// Detailed error code & message from the Google API frontend.
    pub errors: Option<Vec<ErrorResponseItem>>,

    /// Description of the error. Same as `errors.message`.
    pub message: String,
}

const RETRYABLE_CODES: [u16; 4] = [500, 502, 503, 504];

impl ErrorResponse {
    pub fn is_retryable(&self, retryable_reasons: &[&str]) -> bool {
        if RETRYABLE_CODES.contains(&self.code) {
            return true;
        }
        match &self.errors {
            None => false,
            Some(details) => {
                for detail in details {
                    for reason in retryable_reasons {
                        if &detail.reason == reason {
                            return true;
                        }
                    }
                }
                return false;
            }
        }
    }
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for ErrorResponse {}

/// ErrorItem is a detailed error code & message from the Google API frontend.
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponseItem {
    /// Message is the human-readable description of the error.
    pub message: String,

    /// Reason is the typed error code. For example: "some_example".
    pub reason: String,
}

impl fmt::Display for ErrorResponseItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.message.fmt(f)
    }
}

#[derive(serde::Deserialize)]
pub(crate) struct ErrorWrapper {
    pub(crate) error: ErrorResponse,
}
