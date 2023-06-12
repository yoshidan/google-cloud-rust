use std::collections::VecDeque;
use crate::http::bigquery_job_client::BigqueryJobClient;
use crate::http::job::get_query_results::GetQueryResultsRequest;
use crate::http::tabledata::list::Tuple;
use crate::http::error::Error as HttpError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Http(#[from] HttpError),
    #[error("invalid type {0}")]
    Decode(String),
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
    pub async fn next<T: TryFrom<Tuple, Error = String>>(&mut self) -> Result<Option<T>, Error> {
        loop {
            if let Some(v) = self.chunk.pop_front() {
                return T::try_from(v).map(Some).map_err(Error::Decode);
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

    #[derive(thiserror::Error, Debug)]
    pub enum Error {
        #[error("not data found")]
        NoDataFound,
        #[error("invalid type")]
        Decode(String),
    }

    pub struct Row {
        inner: Vec<Cell>,
    }

    impl Row {
        pub fn column<'a, T: TryFrom<&'a Value, Error = String>>(&'a self, index: usize) -> Result<T, Error> {
            let cell: &Cell = self.inner.get(index).ok_or(Error::NoDataFound)?;
            T::try_from(&cell.v).map_err(Error::Decode)
        }
    }

    impl TryFrom<Tuple> for Row {
        type Error = String;

        fn try_from(value: Tuple) -> Result<Self, Self::Error> {
            Ok(Self { inner: value.f })
        }
    }

    impl<'a> TryFrom<&'a Value> for &'a str {
        type Error = String;

        fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
            Ok(match value {
                Value::String(v) => v.as_str(),
                Value::Null => "",
                _ => "invalid value for &str",
            })
        }
    }
}