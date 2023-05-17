use crate::http::bigquery_job_client::BigqueryJobClient;
use crate::http::error::Error as HttpError;
use crate::http::job::get_query_results::GetQueryResultsRequest;
use crate::http::tabledata::list::Tuple;
use async_trait::async_trait;
use std::collections::VecDeque;

#[async_trait]
pub trait AsyncIterator {
    async fn next<T: TryFrom<Tuple, Error = String>>(&mut self) -> Result<Option<T>, Error>;
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Http(#[from] HttpError),
    #[error("invalid type {0}")]
    Decode(String),
}

pub struct QueryIterator {
    pub(crate) client: BigqueryJobClient,
    pub(crate) project_id: String,
    pub(crate) job_id: String,
    pub(crate) request: GetQueryResultsRequest,
    pub(crate) chunk: VecDeque<Tuple>,
    pub total_size: i64,
}

#[async_trait]
impl AsyncIterator for QueryIterator {
    async fn next<T: TryFrom<Tuple, Error = String>>(&mut self) -> Result<Option<T>, Error> {
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
        return self.next().await;
    }
}
