use crate::http::bigquery_client::BigqueryClient;
use crate::http::error::Error;
use crate::http::types::Policy;
use crate::http::{job, table};

use crate::http::job::Job;
use crate::http::table::list::{ListTablesRequest, ListTablesResponse, TableOverview};
use std::sync::Arc;

#[derive(Clone)]
pub struct BigqueryJobClient {
    inner: Arc<BigqueryClient>,
}

impl BigqueryJobClient {
    pub fn new(inner: Arc<BigqueryClient>) -> Self {
        Self { inner }
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn create(&self, metadata: &Job) -> Result<Job, Error> {
        let builder = job::insert::build(self.inner.endpoint(), self.inner.http(), metadata);
        self.inner.send(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn delete(&self, project_id: &str, job_id: &str) -> Result<(), Error> {
        let builder = job::delete::build(self.inner.endpoint(), self.inner.http(), project_id, job_id);
        self.inner.send_get_empty(builder).await
    }
}

#[cfg(test)]
mod test {
    use crate::http::bigquery_client::test::create_client;
    use std::ops::Add;

    use crate::http::bigquery_job_client::BigqueryJobClient;
    use crate::http::job::{Job, JobConfiguration, JobConfigurationQuery};
    use serial_test::serial;
    use std::sync::Arc;
    use time::OffsetDateTime;

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt::try_init();
    }

    #[tokio::test]
    #[serial]
    pub async fn crud_job() {
        let (client, project) = create_client().await;
        let client = BigqueryJobClient::new(Arc::new(client));

        // empty
        let mut job1 = Job::default();
        job1.job_reference.job_id = format!("rust_test_{}", OffsetDateTime::now_utc().unix_timestamp());
        job1.job_reference.project_id = project.to_string();
        job1.job_reference.location = Some("asia-northeast1".to_string());
        job1.configuration = JobConfiguration {
            query: Some(JobConfigurationQuery {
                query: "SELECT 1 FROM rust_test_external_table.test_job1".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let job1 = client.create(&job1).await;
        let job1 = job1.unwrap();

        // cleanup
        let jref = job1.job_reference;
        client
            .delete(jref.project_id.as_str(), jref.job_id.as_str())
            .await
            .unwrap();
    }
}
