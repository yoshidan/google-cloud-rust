use std::collections::VecDeque;
use std::marker::PhantomData;

use crate::http::bigquery_job_client::BigqueryJobClient;
use crate::http::error::Error as HttpError;
use crate::http::job::get_query_results::GetQueryResultsRequest;
use crate::http::query::value::StructDecodable;
use crate::http::tabledata::list::Tuple;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Http(#[from] HttpError),
    #[error(transparent)]
    Value(#[from] value::Error),
}

pub struct Iterator<T: StructDecodable> {
    pub(crate) client: BigqueryJobClient,
    pub(crate) project_id: String,
    pub(crate) job_id: String,
    pub(crate) request: GetQueryResultsRequest,
    pub(crate) chunk: VecDeque<Tuple>,
    pub(crate) force_first_fetch: bool,
    pub total_size: i64,
    pub(crate) _marker: PhantomData<T>,
}

impl<T: StructDecodable> Iterator<T> {
    pub async fn next(&mut self) -> Result<Option<T>, Error> {
        loop {
            if let Some(v) = self.chunk.pop_front() {
                return Ok(T::decode(v).map(Some)?);
            }
            if self.force_first_fetch {
                self.force_first_fetch = false
            } else if self.request.page_token.is_none() {
                return Ok(None);
            }
            let response = self
                .client
                .get_query_results(self.project_id.as_str(), self.job_id.as_str(), &self.request)
                .await?;
            if response.rows.is_none() {
                return Ok(None);
            }
            let v = response.rows.unwrap();
            self.chunk = VecDeque::from(v);
            self.request.page_token = response.page_token;
        }
    }
}

pub mod row {
    use crate::http::query::value::StructDecodable;
    use crate::http::tabledata::list::{Cell, Tuple};

    #[derive(thiserror::Error, Debug)]
    pub enum Error {
        #[error("no data found: {0}")]
        UnexpectedColumnIndex(usize),
        #[error(transparent)]
        Value(#[from] super::value::Error),
    }

    pub struct Row {
        inner: Vec<Cell>,
    }

    impl Row {
        pub fn column<T: super::value::Decodable>(&self, index: usize) -> Result<T, Error> {
            let cell: &Cell = self.inner.get(index).ok_or(Error::UnexpectedColumnIndex(index))?;
            Ok(T::decode(&cell.v)?)
        }
    }

    impl StructDecodable for Row {
        fn decode(value: Tuple) -> Result<Self, crate::http::query::value::Error> {
            Ok(Self { inner: value.f })
        }
    }
}

pub mod value {
    use std::str::FromStr;

    use base64::prelude::BASE64_STANDARD;
    use base64::Engine;
    use bigdecimal::BigDecimal;
    use time::error::ComponentRange;
    use time::macros::format_description;
    use time::{Date, OffsetDateTime, Time};

    use crate::http::tabledata::list::{Tuple, Value};

    #[derive(thiserror::Error, Debug)]
    pub enum Error {
        #[error("invalid type")]
        InvalidType,
        #[error("unexpected null value")]
        UnexpectedNullValue,
        #[error(transparent)]
        Timestamp(#[from] ComponentRange),
        #[error("invalid number {0}")]
        FromString(String),
        #[error(transparent)]
        Base64(#[from] base64::DecodeError),
        #[error(transparent)]
        ParseDateTime(#[from] time::error::Parse),
        #[error(transparent)]
        ParseBigDecimal(#[from] bigdecimal::ParseBigDecimalError),
    }

    pub trait Decodable: Sized {
        fn decode(value: &Value) -> Result<Self, Error>;
    }

    pub trait StructDecodable: Sized {
        fn decode(value: Tuple) -> Result<Self, Error>;
    }

    impl<T: StructDecodable> Decodable for T {
        fn decode(value: &Value) -> Result<Self, Error> {
            match value {
                Value::Struct(v) => T::decode(v.clone()),
                Value::Null => Err(Error::UnexpectedNullValue),
                _ => Err(Error::InvalidType),
            }
        }
    }

    impl Decodable for String {
        fn decode(value: &Value) -> Result<Self, Error> {
            match value {
                Value::String(v) => Ok(v.to_string()),
                Value::Null => Err(Error::UnexpectedNullValue),
                _ => Err(Error::InvalidType),
            }
        }
    }

    impl Decodable for Vec<u8> {
        fn decode(value: &Value) -> Result<Self, Error> {
            match value {
                Value::String(v) => Ok(BASE64_STANDARD.decode(v)?),
                Value::Null => Err(Error::UnexpectedNullValue),
                _ => Err(Error::InvalidType),
            }
        }
    }

    impl Decodable for bool {
        fn decode(value: &Value) -> Result<Self, Error> {
            match value {
                Value::String(v) => Ok(v.parse::<bool>().map_err(|_| Error::FromString(v.clone()))?),
                Value::Null => Err(Error::UnexpectedNullValue),
                _ => Err(Error::InvalidType),
            }
        }
    }

    impl Decodable for f64 {
        fn decode(value: &Value) -> Result<Self, Error> {
            match value {
                Value::String(v) => Ok(v.parse::<f64>().map_err(|_| Error::FromString(v.clone()))?),
                Value::Null => Err(Error::UnexpectedNullValue),
                _ => Err(Error::InvalidType),
            }
        }
    }

    impl Decodable for i64 {
        fn decode(value: &Value) -> Result<Self, Error> {
            match value {
                Value::String(v) => Ok(v.parse::<i64>().map_err(|_| Error::FromString(v.clone()))?),
                Value::Null => Err(Error::UnexpectedNullValue),
                _ => Err(Error::InvalidType),
            }
        }
    }

    impl Decodable for BigDecimal {
        fn decode(value: &Value) -> Result<Self, Error> {
            match value {
                Value::String(v) => Ok(BigDecimal::from_str(v)?),
                Value::Null => Err(Error::UnexpectedNullValue),
                _ => Err(Error::InvalidType),
            }
        }
    }

    impl Decodable for OffsetDateTime {
        fn decode(value: &Value) -> Result<Self, Error> {
            let f = f64::decode(value)?;
            let sec = f.trunc();
            // Timestamps in BigQuery have microsecond precision, so we must
            // return a round number of microseconds.
            let micro = ((f - sec) * 1000000.0 + 0.5).trunc();
            Ok(OffsetDateTime::from_unix_timestamp_nanos(
                sec as i128 * 1_000_000_000 + micro as i128 * 1000,
            )?)
        }
    }

    impl Decodable for Date {
        fn decode(value: &Value) -> Result<Self, Error> {
            match value {
                Value::String(v) => Ok(Date::parse(v, format_description!("[year]-[month]-[day]"))?),
                Value::Null => Err(Error::UnexpectedNullValue),
                _ => Err(Error::InvalidType),
            }
        }
    }

    impl Decodable for Time {
        fn decode(value: &Value) -> Result<Self, Error> {
            match value {
                Value::String(v) => Ok(Time::parse(v, format_description!("[hour]:[minute]:[second]"))?),
                Value::Null => Err(Error::UnexpectedNullValue),
                _ => Err(Error::InvalidType),
            }
        }
    }

    impl<T> Decodable for Vec<T>
    where
        T: Decodable,
    {
        fn decode(value: &Value) -> Result<Self, Error> {
            match value {
                Value::Array(v) => {
                    let mut result = Vec::with_capacity(v.len());
                    for element in v {
                        result.push(T::decode(&element.v)?);
                    }
                    Ok(result)
                }
                Value::Null => Err(Error::UnexpectedNullValue),
                _ => Err(Error::InvalidType),
            }
        }
    }

    impl<T> Decodable for Option<T>
    where
        T: Decodable,
    {
        fn decode(value: &Value) -> Result<Self, Error> {
            match value {
                Value::Null => Ok(None),
                _ => Ok(Some(T::decode(value)?)),
            }
        }
    }
}
