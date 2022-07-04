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
use serde::{de, Deserialize, Deserializer};
use serde_json::Value;
use std::fmt::Display;
use std::str::FromStr;
pub use tokio_util::sync::CancellationToken;

//TODO emulator support
pub(crate) const BASE_URL: &str = "https://storage.googleapis.com";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("http error status={0} message={1}")]
    Response(u16, String),
    #[error(transparent)]
    HttpClient(#[from] reqwest::Error),
    #[error(transparent)]
    ResponseJson(#[from] serde_json::Error),
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
