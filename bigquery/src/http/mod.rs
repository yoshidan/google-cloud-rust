use std::fmt::Display;
use std::str::FromStr;

use serde::{de, Deserialize, Deserializer};
use serde_json::Value;

pub mod bigquery_client;
pub mod bigquery_dataset_client;
pub mod bigquery_job_client;
pub mod bigquery_routine_client;
pub mod bigquery_row_access_policy_client;
pub mod bigquery_table_client;
pub mod bigquery_tabledata_client;
pub mod bigquery_model_client;
pub mod dataset;
pub mod error;
pub mod job;
pub mod model;
pub mod routine;
pub mod row_access_policy;
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

fn from_str_vec_option<'de, T, D>(deserializer: D) -> Result<Option<Vec<T>>, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let s: Result<Value, _> = Deserialize::deserialize(deserializer);
    match s {
        Ok(Value::Array(vec)) => {
            let mut result = Vec::with_capacity(vec.len());
            for v in vec {
                let v = match v {
                    Value::String(s) => T::from_str(&s).map_err(de::Error::custom)?,
                    _ => return Err(de::Error::custom("Incorrect type")),
                };
                result.push(v);
            }
            Ok(Some(result))
        }
        Ok(_) => Err(de::Error::custom("Incorrect type")),
        Err(_) => Ok(None),
    }
}

fn from_str_vec<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let vec: Vec<String> = Vec::deserialize(deserializer)?;
    let mut result = Vec::with_capacity(vec.len());
    for v in vec {
        result.push(T::from_str(&v).map_err(de::Error::custom)?);
    }
    Ok(result)
}

#[cfg(test)]
mod test {

    #[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
    #[serde(rename_all = "camelCase")]
    struct Test {
        #[serde(default, deserialize_with = "crate::http::from_str_vec_option")]
        pub field: Option<Vec<i64>>,
    }

    #[test]
    fn test_from_str_vec_option() {
        let value: Test = serde_json::from_str(r#"{"field": ["100", "200"]}"#).unwrap();
        let record = value.field.unwrap();
        assert_eq!(vec![100, 200], record);

        let value: Test = serde_json::from_str(r#"{}"#).unwrap();
        assert!(value.field.is_none());
    }

    #[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
    #[serde(rename_all = "camelCase")]
    struct Test2 {
        #[serde(deserialize_with = "crate::http::from_str_vec")]
        pub field: Vec<i64>,
    }

    #[test]
    fn test_from_str_vec() {
        let value: Test2 = serde_json::from_str(r#"{"field": ["100", "200"]}"#).unwrap();
        assert_eq!(vec![100, 200], value.field);
        let result = serde_json::from_str::<Test2>(r#"{}"#);
        assert!(result.is_err())
    }
}
