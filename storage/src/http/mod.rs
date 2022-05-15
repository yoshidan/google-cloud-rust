//pub mod entity;
pub mod buckets;
pub mod objects;
pub mod channels;
pub mod bucket_access_controls;
pub mod object_access_controls;
pub mod default_object_access_controls;
pub mod storage_client;

use std::fmt::Display;
use std::str::FromStr;
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use serde::{de, Deserialize};
pub use tokio_util::sync::CancellationToken;

pub(crate) const BASE_URL: &str = "https://storage.googleapis.com/storage/v1";

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

/// https://github.com/ThouCheese/cloud-storage-rs/blob/0b09eccf5f6795becb50c4b2f444daeae7995141/src/resources/object.rs
const ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'*')
    .remove(b'-')
    .remove(b'.')
    .remove(b'_');

const NOSLASH_ENCODE_SET: &AsciiSet = &ENCODE_SET.remove(b'/').remove(b'~');

pub(crate) fn percent_encode_noslash(input: &str) -> String {
    utf8_percent_encode(input, NOSLASH_ENCODE_SET).to_string()
}

pub(crate) fn percent_encode(input: &str) -> String {
    utf8_percent_encode(input, ENCODE_SET).to_string()
}


#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_empty(v: &str) -> bool {
    v.is_empty()
}

fn from_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: FromStr,
        T::Err: Display,
        D: de::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(de::Error::custom)
}

fn from_str_opt<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Display,
        D: serde::Deserializer<'de>,
{
    let s: Result<serde_json::Value, _> = serde::Deserialize::deserialize(deserializer);
    match s {
        Ok(serde_json::Value::String(s)) => T::from_str(&s).map_err(serde::de::Error::custom).map(Option::from),
        Ok(serde_json::Value::Number(num)) => T::from_str(&num.to_string())
            .map_err(serde::de::Error::custom)
            .map(Option::from),
        Ok(_value) => Err(serde::de::Error::custom("Incorrect type")),
        Err(_) => Ok(None),
    }
}
