use crate::http::bigquery_job_client::BigqueryJobClient;
use crate::http::error::Error as HttpError;
use crate::http::job::get_query_results::GetQueryResultsRequest;
use crate::http::tabledata::list::Tuple;
use std::collections::VecDeque;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Http(#[from] HttpError),
    #[error(transparent)]
    Row(#[from] row::Error),
}

pub struct Iterator {
    pub(crate) client: BigqueryJobClient,
    pub(crate) project_id: String,
    pub(crate) job_id: String,
    pub(crate) request: GetQueryResultsRequest,
    pub(crate) chunk: VecDeque<Tuple>,
    pub total_size: i64,
}

impl Iterator {
    pub async fn next<T: TryFrom<Tuple, Error = row::Error>>(&mut self) -> Result<Option<T>, Error> {
        loop {
            if let Some(v) = self.chunk.pop_front() {
                return Ok(T::try_from(v).map(Some)?);
            }
            if self.request.page_token.is_none() {
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
    use crate::http::tabledata::list::{Cell, Tuple, Value};
    use base64::prelude::BASE64_STANDARD;
    use base64::Engine;
    use bigdecimal::BigDecimal;
    use std::str::FromStr;
    use time::error::ComponentRange;
    use time::macros::format_description;
    use time::{Date, OffsetDateTime, Time};

    #[derive(thiserror::Error, Debug)]
    pub enum Error {
        #[error("no data found")]
        NoDataFound,
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

    pub struct Row {
        inner: Vec<Cell>,
    }

    impl Row {
        pub fn column<'a, T: ValueDecodable<'a>>(&'a self, index: usize) -> Result<T, Error> {
            let cell: &Cell = self.inner.get(index).ok_or(Error::NoDataFound)?;
            T::decode(&cell.v)
        }
    }

    impl TryFrom<Tuple> for Row {
        type Error = Error;

        fn try_from(value: Tuple) -> Result<Self, Self::Error> {
            Ok(Self { inner: value.f })
        }
    }

    pub trait ValueDecodable<'a>: Sized {
        fn decode(value: &'a Value) -> Result<Self, Error>;
    }

    impl<'a> ValueDecodable<'a> for &'a str {
        fn decode(value: &'a Value) -> Result<Self, Error> {
            match value {
                Value::String(v) => Ok(v.as_str()),
                Value::Null => Err(Error::UnexpectedNullValue),
                _ => Err(Error::InvalidType),
            }
        }
    }

    impl<'a> ValueDecodable<'a> for String {
        fn decode(value: &'a Value) -> Result<Self, Error> {
            match value {
                Value::String(v) => Ok(v.to_string()),
                Value::Null => Err(Error::UnexpectedNullValue),
                _ => Err(Error::InvalidType),
            }
        }
    }

    impl<'a> ValueDecodable<'a> for Vec<u8> {
        fn decode(value: &'a Value) -> Result<Self, Error> {
            match value {
                Value::String(v) => Ok(BASE64_STANDARD.decode(v)?),
                Value::Null => Err(Error::UnexpectedNullValue),
                _ => Err(Error::InvalidType),
            }
        }
    }

    impl<'a> ValueDecodable<'a> for bool {
        fn decode(value: &'a Value) -> Result<Self, Error> {
            match value {
                Value::String(v) => Ok(v.parse::<bool>().map_err(|_| Error::FromString(v.clone()))?),
                Value::Null => Err(Error::UnexpectedNullValue),
                _ => Err(Error::InvalidType),
            }
        }
    }

    impl<'a> ValueDecodable<'a> for f64 {
        fn decode(value: &'a Value) -> Result<Self, Error> {
            match value {
                Value::String(v) => Ok(v.parse::<f64>().map_err(|_| Error::FromString(v.clone()))?),
                Value::Null => Err(Error::UnexpectedNullValue),
                _ => Err(Error::InvalidType),
            }
        }
    }

    impl<'a> ValueDecodable<'a> for i64 {
        fn decode(value: &'a Value) -> Result<Self, Error> {
            match value {
                Value::String(v) => Ok(v.parse::<i64>().map_err(|_| Error::FromString(v.clone()))?),
                Value::Null => Err(Error::UnexpectedNullValue),
                _ => Err(Error::InvalidType),
            }
        }
    }

    impl<'a> ValueDecodable<'a> for BigDecimal {
        fn decode(value: &'a Value) -> Result<Self, Error> {
            match value {
                Value::String(v) => Ok(BigDecimal::from_str(v)?),
                Value::Null => Err(Error::UnexpectedNullValue),
                _ => Err(Error::InvalidType),
            }
        }
    }

    impl<'a> ValueDecodable<'a> for OffsetDateTime {
        fn decode(value: &'a Value) -> Result<Self, Error> {
            let f = f64::decode(value)?;
            let sec = f.trunc();
            // Timestamps in BigQuery have microsecond precision, so we must
            // return a round number of microseconds.
            let micro = ((f - sec.clone()) * 1000000.0 + 0.5).trunc();
            Ok(OffsetDateTime::from_unix_timestamp_nanos(
                sec as i128 * 1_000_000_000 + micro as i128 * 1000,
            )?)
        }
    }

    impl<'a> ValueDecodable<'a> for Date {
        fn decode(value: &'a Value) -> Result<Self, Error> {
            match value {
                Value::String(v) => Ok(Date::parse(v, format_description!("[year]-[month]-[day]"))?),
                Value::Null => Err(Error::UnexpectedNullValue),
                _ => Err(Error::InvalidType),
            }
        }
    }

    impl<'a> ValueDecodable<'a> for Time {
        fn decode(value: &'a Value) -> Result<Self, Error> {
            match value {
                Value::String(v) => Ok(Time::parse(v, format_description!("[hour]:[minute]:[second]"))?),
                Value::Null => Err(Error::UnexpectedNullValue),
                _ => Err(Error::InvalidType),
            }
        }
    }

    impl<'a, T> ValueDecodable<'a> for Vec<T>
    where
        T: ValueDecodable<'a>,
    {
        fn decode(value: &'a Value) -> Result<Self, Error> {
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

    impl<'a, T> ValueDecodable<'a> for Option<T>
    where
        T: ValueDecodable<'a>,
    {
        fn decode(value: &'a Value) -> Result<Self, Error> {
            match value {
                Value::Null => Ok(None),
                _ => Ok(Some(T::decode(value)?)),
            }
        }
    }
}
