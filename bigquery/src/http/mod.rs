use serde::{de, Deserialize, Deserializer};
use serde_json::Value;
use std::fmt::Display;
use std::str::FromStr;

pub mod bigquery_client;
mod bigquery_dataset_client;
pub mod bigquery_table_client;
pub mod bigquery_tabledata_client;
pub mod dataset;
pub mod error;
pub mod routine;
pub mod table;
pub mod tabledata;
pub mod types;

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
