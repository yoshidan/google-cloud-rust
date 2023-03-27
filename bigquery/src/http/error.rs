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

/// An error response returned from Google Cloud Storage.
///
/// See the [`HTTP status and error codes for JSON`][1] documentation for more details.
///
/// [1]: https://cloud.google.com/storage/docs/json_api/v1/status-codes
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    /// An HTTP status value, without the textual description.
    ///
    /// Example values include: `400` (Bad Request), `401` (Unauthorized), and `404` (Not Found).
    pub code: u16,

    /// A container for the error details.
    pub errors: Vec<ErrorResponseItem>,

    /// Description of the error. Same as `errors.message`.
    pub message: String,
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for ErrorResponse {}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponseItem {
    /// The scope of the error. Example values include: `global` and `push`.
    pub domain: String,

    /// The specific item within the `locationType` that caused the error. For example, if you
    /// specify an invalid value for a parameter, the `location` will be the name of the parameter.
    ///
    /// Example values include: `Authorization`, `project`, and `projection`.
    pub location: Option<String>,

    /// The location or part of the request that caused the error. Use with `location` to pinpoint
    /// the error. For example, if you specify an invalid value for a parameter, the `locationType`
    /// will be `parameter` and the `location` will be the name of the parameter.
    ///
    /// Example values include `header` and `parameter`.
    pub location_type: Option<String>,

    /// Description of the error.
    ///
    /// Example values include `Invalid argument`, `Login required`, and
    /// `Required parameter: project`.
    pub message: String,

    /// Example values include `invalid`, `invalidParameter`, and `required`.
    pub reason: String,
}

impl fmt::Display for ErrorResponseItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.message.fmt(f)
    }
}

/// The GCS error response JSON format contains an extra object level that is inconvenient to include in our
/// error.
#[derive(serde::Deserialize)]
pub(crate) struct ErrorWrapper {
    pub(crate) error: ErrorResponse,
}
