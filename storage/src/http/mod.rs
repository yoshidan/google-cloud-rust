use std::fmt::Display;
use std::str::FromStr;

use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};
use reqwest::Response;
use serde::{de, Deserialize, Deserializer};
use serde_json::Value;

//pub mod entity;
pub mod bucket_access_controls;
pub mod buckets;
pub mod channels;
pub mod default_object_access_controls;
pub mod error;
pub mod hmac_keys;
pub mod notifications;
pub mod object_access_controls;
pub mod objects;
pub mod resumable_upload_client;
pub mod service_account_client;
pub mod storage_client;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// An error returned from the Google Cloud Storage service.
    #[error(transparent)]
    Response(#[from] error::ErrorResponse),

    /// An error from the underlying HTTP client.
    #[error(transparent)]
    HttpClient(#[from] reqwest::Error),

    /// An error from one of the middleware used.
    #[error(transparent)]
    HttpMiddleware(anyhow::Error),

    /// An error from a token source.
    #[error("token source failed: {0}")]
    TokenSource(Box<dyn std::error::Error + Send + Sync>),

    /// Invalid Range error
    #[error("invalid range header, received: {0}")]
    InvalidRangeHeader(String),

    #[error("Request failed: {0} detail={1}")]
    RawResponse(reqwest::Error, String),
}

impl From<reqwest_middleware::Error> for Error {
    fn from(error: reqwest_middleware::Error) -> Self {
        match error {
            reqwest_middleware::Error::Middleware(err) => Error::HttpMiddleware(err),
            reqwest_middleware::Error::Reqwest(err) => Error::HttpClient(err),
        }
    }
}

/// Checks whether an HTTP response is successful and returns it, or returns an error.
pub(crate) async fn check_response_status(response: Response) -> Result<Response, Error> {
    // Check the status code, returning the response if it is not an error.
    let error = match response.error_for_status_ref() {
        Ok(_) => return Ok(response),
        Err(error) => error,
    };

    // try to extract a response error, falling back to the status error if it can not be parsed.
    Err(response
        .json::<error::ErrorWrapper>()
        .await
        .map(|wrapper| Error::Response(wrapper.error))
        .unwrap_or(Error::HttpClient(error)))
}

pub(crate) trait Escape {
    fn escape(&self) -> String;
}

impl Escape for String {
    fn escape(&self) -> String {
        utf8_percent_encode(self, ENCODE_SET).to_string()
    }
}

const ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC.remove(b'*').remove(b'-').remove(b'.').remove(b'_');

fn from_str_option<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s: Result<Value, _> = Deserialize::deserialize(deserializer);
    match s {
        Ok(Value::String(s)) => T::from_str(&s).map_err(de::Error::custom).map(Some),
        Ok(Value::Number(num)) => T::from_str(&num.to_string()).map_err(de::Error::custom).map(Some),
        Ok(_) => Err(de::Error::custom("Incorrect type")),
        Err(_) => Ok(None),
    }
}

pub fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: de::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}

pub fn is_i64_zero(num: &i64) -> bool {
    *num == 0
}

/// Provides serialization and deserialization for base64 encoded fields.
mod base64 {
    use base64::prelude::*;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: AsRef<[u8]>,
        S: Serializer,
    {
        BASE64_STANDARD.encode(value.as_ref()).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        BASE64_STANDARD
            .decode(String::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
    }
}
