use std::collections::VecDeque;
use std::convert::Infallible;
use crate::http::bigquery_job_client::BigqueryJobClient;
use crate::http::error::Error as HttpError;
use crate::http::job::get_query_results::GetQueryResultsRequest;
use crate::http::job::query::{QueryRequest, QueryResponse};
use crate::http::tabledata::list::Tuple;

#[derive(thiserror::Error)]
enum Error {
    #[error(transparent)]
    Http(#[from] HttpError),
    #[error(transparent)]
    Decode(#[from] Infallible)
}

pub struct RowIterator {
    pub(crate) client: BigqueryJobClient,
    pub(crate) project_id: String,
    pub(crate) job_id: String,
    pub(crate) request: GetQueryResultsRequest,
    pub(crate) chunk: VecDeque<Tuple>,
    pub total_size: i64
}

impl <T: TryFrom<Tuple>> RowIterator {
    pub async fn next<T>(&mut self) -> Result<Option<T>, Error> {
        if let Some(v) = self.chunk.pop_front() {
            return T::try_from(v).map(Some).map_err(Error::Decode);
        }
        if self.request.page_token.is_none() {
            return Ok(None)
        }
        let response = self.client.get_query_results(self.project_id.as_str(), self.job_id.as_str(), &self.request).await?;
        match response.rows {
            None => return Ok(None),
            Some(v) => {
                self.chunk = VecDeque::from(v);
                self.request.page_token = response.page_token;
                self.next()
            }
        }
    }
}