use std::fmt::Display;
use std::str::FromStr;
use serde::{de, Deserialize};

pub mod acl;
pub mod hmac_key;
pub mod bucket;
pub mod channel;
pub mod common;
pub mod iam;
pub mod object;
pub mod notification;

pub struct Prefix(String);

impl Prefix {
    pub fn as_param(&self) -> (&'static str, &str)  {
        ("prefix", v.0.as_str())
    }
}

pub struct PageToken(String);

impl PageToken {
    pub fn as_param(&self) -> (&'static str, &str)  {
        ("page_token", v.0.as_str())
    }
}

pub struct StringParam(&'static str, String);

impl StringParam {
    pub fn as_param(&self) -> (&'static str, &str) {
        (self.0, self.1.as_str())
    }
}

pub struct Project(String);

impl Project {
    pub fn as_param(&self ) -> (&'static str, &str)  {
        ("project", v.0.as_str())
    }
}

pub struct MaxResults(i32);

impl MaxResults {
    pub fn to_param(&self) -> StringParam  {
        StringParam("max_results", v.0.to_string())
    }
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