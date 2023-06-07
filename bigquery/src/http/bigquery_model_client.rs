use crate::http::bigquery_client::BigqueryClient;
use crate::http::error::Error;
use crate::http::model;
use crate::http::model::Model;

use crate::http::model::list::{ListModelsRequest, ListModelsResponse, ModelOverview};
use std::sync::Arc;

#[derive(Clone)]
pub struct BigqueryModelClient {
    inner: Arc<BigqueryClient>,
}

impl BigqueryModelClient {
    pub fn new(inner: Arc<BigqueryClient>) -> Self {
        Self { inner }
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn delete(&self, project_id: &str, dataset_id: &str, table_id: &str) -> Result<(), Error> {
        let builder = model::delete::build(self.inner.endpoint(), self.inner.http(), project_id, dataset_id, table_id);
        self.inner.send_get_empty(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn patch(&self, metadata: &Model) -> Result<Model, Error> {
        let builder = model::patch::build(self.inner.endpoint(), self.inner.http(), metadata);
        self.inner.send(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get(&self, project_id: &str, dataset_id: &str, model_id: &str) -> Result<Model, Error> {
        let builder = model::get::build(self.inner.endpoint(), self.inner.http(), project_id, dataset_id, model_id);
        self.inner.send(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn list(
        &self,
        project_id: &str,
        dataset_id: &str,
        req: &ListModelsRequest,
    ) -> Result<Vec<ModelOverview>, Error> {
        let mut page_token: Option<String> = None;
        let mut models = vec![];
        loop {
            let builder = model::list::build(
                self.inner.endpoint(),
                self.inner.http(),
                project_id,
                dataset_id,
                req,
                page_token,
            );
            let response: ListModelsResponse = self.inner.send(builder).await?;
            models.extend(response.models);
            if response.next_page_token.is_none() {
                break;
            }
            page_token = response.next_page_token;
        }
        Ok(models)
    }
}

#[cfg(test)]
mod test {
    use crate::http::bigquery_client::test::create_client;

    use crate::http::bigquery_job_client::BigqueryJobClient;
    use crate::http::bigquery_model_client::BigqueryModelClient;
    use crate::http::job::get::GetJobRequest;
    use crate::http::job::query::QueryRequest;
    use crate::http::job::{Job, JobConfiguration, JobConfigurationQuery, JobState, JobType};
    use crate::http::model::list::ListModelsRequest;
    use serial_test::serial;
    use std::sync::Arc;
    use time::OffsetDateTime;

    use crate::http::model::ModelType;

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt::try_init();
    }

    #[tokio::test]
    #[serial]
    pub async fn crud_model() {
        let (client, project) = create_client().await;
        let job_client = BigqueryJobClient::new(Arc::new(client.clone()));
        let client = BigqueryModelClient::new(Arc::new(client));

        // create model
        let model_id = format!("penguins_model_{}", OffsetDateTime::now_utc().unix_timestamp());
        let mut job1 = Job::default();
        job1.job_reference.job_id = format!("rust_test_model_job_{}", OffsetDateTime::now_utc().unix_timestamp());
        job1.job_reference.project_id = project.to_string();
        job1.job_reference.location = Some("US".to_string());
        job1.configuration = JobConfiguration {
            job: JobType::Query(JobConfigurationQuery {
                use_legacy_sql: Some(false),
                query: format!(
                    "
                    CREATE OR REPLACE MODEL `rust_test_model_us.{}`
                    OPTIONS (model_type='linear_reg', input_label_cols=['body_mass_g']) AS
                        SELECT
                            *
                        FROM
                            `bigquery-public-data.ml_datasets.penguins`
                        WHERE
                            body_mass_g IS NOT NULL
                        LIMIT 100
                    ",
                    model_id
                ),
                ..Default::default()
            }),
            ..Default::default()
        };
        let mut job = job_client.create(&job1).await.unwrap();

        // wait for training complete
        let elapsed = 0;
        loop {
            if job.status.state == JobState::Done {
                break;
            }
            let jr = &job.job_reference;
            job = job_client
                .get(&jr.project_id, &jr.job_id, &GetJobRequest { location: None })
                .await
                .unwrap();
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            tracing::info!("current job status.state = {:?}", job.status.state);
            assert!(elapsed < 20, "model creation timedout");
        }

        // predict
        let result = job_client
            .query(
                &project,
                &QueryRequest {
                    max_results: None,
                    query: format!(
                        "
                    SELECT * FROM  ML.PREDICT(MODEL `rust_test_model_us.{}`, (
                        SELECT
                            *
                        FROM
                            `bigquery-public-data.ml_datasets.penguins`
                        WHERE
                            body_mass_g IS NOT NULL
                        AND island = 'Biscoe' LIMIT 10))
                    ",
                        model_id
                    ),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert_eq!(result.total_rows.unwrap(), 10);

        // list / get / patch / delete
        let models = client
            .list(&project, "rust_test_model_us", &ListModelsRequest::default())
            .await
            .unwrap();
        assert!(!models.is_empty());

        for model in models {
            let model = model.model_reference;
            let model = client
                .get(model.project_id.as_str(), model.dataset_id.as_str(), model.model_id.as_str())
                .await
                .unwrap();
            assert_eq!(model.model_type.clone().unwrap(), ModelType::LinearRegression);
            let model = &client.patch(&model).await.unwrap().model_reference;
            client
                .delete(model.project_id.as_str(), model.dataset_id.as_str(), model.model_id.as_str())
                .await
                .unwrap();
        }
    }
}
