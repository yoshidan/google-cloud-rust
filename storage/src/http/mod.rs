use std::fmt::Display;
use std::str::FromStr;
use std::string::FromUtf8Error;

use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};
use reqwest::Response;
use serde::{de, Deserialize, Deserializer};
use serde_json::Value;

//pub mod entity;
pub mod bucket_access_controls;
pub mod buckets;
pub mod channels;
pub mod default_object_access_controls;
pub mod hmac_keys;
pub mod notifications;
pub mod object_access_controls;
pub mod objects;
pub mod resumable_upload_client;
pub mod service_account_client;
pub mod storage_client;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("http error status={0} message={1}")]
    Response(u16, String),
    #[error(transparent)]
    HttpClient(#[from] reqwest::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("operation cancelled")]
    Cancelled,
    #[error(transparent)]
    Base64DecodeError(#[from] base64::DecodeError),
    #[error(transparent)]
    Std(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error(transparent)]
    FromUtf8Error(#[from] FromUtf8Error),
}

impl Error {
    pub async fn from_response(r: Response) -> Error {
        let status = r.status().as_u16();
        let text = match r.text().await {
            Ok(text) => text,
            Err(e) => format!("{e}"),
        };
        Error::Response(status, text)
    }
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
