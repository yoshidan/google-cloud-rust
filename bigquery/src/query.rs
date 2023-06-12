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
    use std::str::FromStr;
    use time::error::ComponentRange;
    use time::OffsetDateTime;

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
        InvalidNumber(String),
    }

    pub struct Row {
        inner: Vec<Cell>,
    }

    impl Row {
        pub fn column<'a, T: TryFrom<&'a Value, Error = Error>>(&'a self, index: usize) -> Result<T, Error> {
            let cell: &Cell = self.inner.get(index).ok_or(Error::NoDataFound)?;
            T::try_from(&cell.v)
        }
    }

    impl TryFrom<Tuple> for Row {
        type Error = Error;

        fn try_from(value: Tuple) -> Result<Self, Self::Error> {
            Ok(Self { inner: value.f })
        }
    }

    impl<'a> TryFrom<&'a Value> for &'a str {
        type Error = Error;

        fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
            match value {
                Value::String(v) => Ok(v.as_str()),
                Value::Null => Err(Error::UnexpectedNullValue),
                _ => Err(Error::InvalidType),
            }
        }
    }

    impl<'a> TryFrom<&'a Value> for OffsetDateTime {
        type Error = Error;

        fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
            match value {
                Value::String(v) => {
                    let f: f64 = v.parse().map_err(|_| Error::InvalidNumber(v.clone()))?;
                    let sec = f.trunc();
                    // Timestamps in BigQuery have microsecond precision, so we must
                    // return a round number of microseconds.
                    let micro = ((f - sec.clone()) * 1000000.0 + 0.5).trunc();
                    Ok(OffsetDateTime::from_unix_timestamp_nanos(
                        sec as i128 * 1_000_000_000 + micro as i128 * 1000,
                    )?)
                }
                Value::Null => Err(Error::UnexpectedNullValue),
                _ => Err(Error::InvalidType),
            }
        }
    }

    impl<'a> TryFrom<&'a Value> for i64 {
        type Error = Error;

        fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
            match value {
                Value::String(v) => Ok(v.parse::<i64>().map_err(|_| Error::InvalidNumber(v.clone()))?),
                Value::Null => Err(Error::UnexpectedNullValue),
                _ => Err(Error::InvalidType),
            }
        }
    }
}
