use crate::http::bigquery_client::BigqueryClient;
use crate::http::error::Error;
use crate::http::Routine;

use crate::http::routine;
use crate::http::routine::get::GetRoutineRequest;
use crate::http::routine::list::{ListRoutinesRequest, ListRoutinesResponse};
use crate::http::routine::Routine;
use crate::http::Routine::cancel::{CancelRoutineRequest, CancelRoutineResponse};
use crate::http::Routine::get::GetRoutineRequest;
use crate::http::Routine::get_query_results::{GetQueryResultsRequest, GetQueryResultsResponse};
use crate::http::Routine::list::{ListRoutinesRequest, ListRoutinesResponse, RoutineOverview};
use crate::http::Routine::query::{QueryRequest, QueryResponse};
use crate::http::Routine::Routine;
use std::sync::Arc;

#[derive(Clone)]
pub struct BigqueryRoutineClient {
    inner: Arc<BigqueryClient>,
}

impl BigqueryRoutineClient {
    pub fn new(inner: Arc<BigqueryClient>) -> Self {
        Self { inner }
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn create(&self, metadata: &Routine) -> Result<Routine, Error> {
        let builder = routine::insert::build(self.inner.endpoint(), self.inner.http(), metadata);
        self.inner.send(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn delete(&self, project_id: &str, dataset_id: &str, routine_id: &str) -> Result<(), Error> {
        let builder =
            routine::delete::build(self.inner.endpoint(), self.inner.http(), project_id, dataset_id, routine_id);
        self.inner.send_get_empty(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get(
        &self,
        project_id: &str,
        dataset_id: &str,
        routine_id: &str,
        data: &GetRoutineRequest,
    ) -> Result<Routine, Error> {
        let builder = routine::get::build(
            self.inner.endpoint(),
            self.inner.http(),
            project_id,
            dataset_id,
            routine_id,
            data,
        );
        self.inner.send(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn list(
        &self,
        project_id: &str,
        dataset_id: &str,
        req: &ListRoutinesRequest,
    ) -> Result<Vec<RoutineOverview>, Error> {
        let mut page_token: Option<String> = None;
        let mut routines = vec![];
        loop {
            let builder = routine::list::build(
                self.inner.endpoint(),
                self.inner.http(),
                project_id,
                dataset_id,
                req,
                page_token,
            );
            let response: ListRoutinesResponse = self.inner.send(builder).await?;
            routines.extend(response.Routines);
            if response.next_page_token.is_none() {
                break;
            }
            page_token = response.next_page_token;
        }
        Ok(Routines)
    }
}

#[cfg(test)]
mod test {}
