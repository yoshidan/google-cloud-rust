//pub mod entity;
pub mod bucket_access_controls;
pub mod buckets;
pub mod channels;
pub mod default_object_access_controls;
pub mod hmac_keys;
pub mod notifications;
pub mod object_access_controls;
pub mod objects;
pub mod storage_client;

use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};
use serde::{de, Deserialize};
use std::fmt::Display;
use std::str::FromStr;
pub use tokio_util::sync::CancellationToken;

pub(crate) const BASE_URL: &str = "https://storage.googleapis.com/storage/v1";
pub(crate) const UPLOAD_BASE_URL: &str = "https://storage.googleapis.com/upload/storage/v1";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("http error status={0} message={1}")]
    Response(u16, String),
    #[error(transparent)]
    HttpClient(#[from] reqwest::Error),
    #[error(transparent)]
    AuthError(#[from] google_cloud_auth::error::Error),
    #[error("operation cancelled")]
    Cancelled,
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

pub fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: de::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}
