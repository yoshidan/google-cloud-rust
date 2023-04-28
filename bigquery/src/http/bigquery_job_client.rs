use crate::http::bigquery_client::BigqueryClient;
use crate::http::error::Error;
use crate::http::job;

use crate::http::job::cancel::{CancelJobRequest, CancelJobResponse};
use crate::http::job::get::GetJobRequest;
use crate::http::job::get_query_results::{GetQueryResultsRequest, GetQueryResultsResponse};
use crate::http::job::list::{JobOverview, ListJobsRequest, ListJobsResponse};
use crate::http::job::query::{QueryRequest, QueryResponse};
use crate::http::job::Job;
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

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get(&self, project_id: &str, job_id: &str, data: &GetJobRequest) -> Result<Job, Error> {
        let builder = job::get::build(self.inner.endpoint(), self.inner.http(), project_id, job_id, data);
        self.inner.send(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn cancel(
        &self,
        project_id: &str,
        job_id: &str,
        data: &CancelJobRequest,
    ) -> Result<CancelJobResponse, Error> {
        let builder = job::cancel::build(self.inner.endpoint(), self.inner.http(), project_id, job_id, data);
        self.inner.send(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn query(&self, project_id: &str, data: &QueryRequest) -> Result<QueryResponse, Error> {
        let builder = job::query::build(self.inner.endpoint(), self.inner.http(), project_id, data);
        self.inner.send(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_query_results(
        &self,
        project_id: &str,
        job_id: &str,
        data: &GetQueryResultsRequest,
    ) -> Result<GetQueryResultsResponse, Error> {
        let builder = job::get_query_results::build(self.inner.endpoint(), self.inner.http(), project_id, job_id, data);
        self.inner.send(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn list(&self, project_id: &str, req: &ListJobsRequest) -> Result<Vec<JobOverview>, Error> {
        let mut page_token: Option<String> = None;
        let mut jobs = vec![];
        loop {
            let builder = job::list::build(self.inner.endpoint(), self.inner.http(), project_id, req, page_token);
            let response: ListJobsResponse = self.inner.send(builder).await?;
            jobs.extend(response.jobs);
            if response.next_page_token.is_none() {
                break;
            }
            page_token = response.next_page_token;
        }
        Ok(jobs)
    }
}

#[cfg(test)]
mod test {
    use crate::http::bigquery_client::test::{create_client, create_table_schema, TestData, TestDataStruct};

    use crate::http::bigquery_job_client::BigqueryJobClient;
    use crate::http::bigquery_table_client::BigqueryTableClient;
    use crate::http::bigquery_tabledata_client::BigqueryTabledataClient;
    use crate::http::job::cancel::CancelJobRequest;
    use crate::http::job::get::GetJobRequest;
    use crate::http::job::get_query_results::GetQueryResultsRequest;
    use crate::http::job::query::QueryRequest;
    use crate::http::job::{
        CreateDisposition, Job, JobConfiguration, JobConfigurationExtract, JobConfigurationExtractSource,
        JobConfigurationLoad, JobConfigurationQuery, JobConfigurationSourceTable, JobConfigurationTableCopy, JobState,
        JobType, OperationType, TrainingType, WriteDisposition,
    };
    use crate::http::model::ModelType;
    use crate::http::table::{DestinationFormat, SourceFormat, Table, TableReference};
    use crate::http::tabledata::insert_all::{InsertAllRequest, Row};
    use core::default::Default;
    use serial_test::serial;
    use std::sync::Arc;
    use time::OffsetDateTime;

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt::try_init();
    }

    #[tokio::test]
    #[serial]
    pub async fn create_job_error() {
        let (client, project) = create_client().await;
        let client = BigqueryJobClient::new(Arc::new(client));

        let mut job1 = Job::default();
        job1.job_reference.job_id = format!("rust_test_{}", OffsetDateTime::now_utc().unix_timestamp());
        job1.job_reference.project_id = project.to_string();
        job1.job_reference.location = Some("asia-northeast1".to_string());
        job1.configuration = JobConfiguration {
            job: JobType::Query(JobConfigurationQuery {
                query: "SELECT 1 FROM invalid_table".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let job1 = client.create(&job1).await.unwrap();
        assert!(job1.status.errors.is_some());
        assert!(job1.status.error_result.is_some());
        let error_result = job1.status.error_result.unwrap();
        assert_eq!(error_result.reason.unwrap().as_str(), "invalid");
        assert_eq!(error_result.location.unwrap().as_str(), "invalid_table");
        assert_eq!(job1.status.state, JobState::Done);
    }

    #[tokio::test]
    #[serial]
    pub async fn create_job() {
        let (client, project) = create_client().await;
        let client = Arc::new(client);
        let client = BigqueryJobClient::new(client);

        // query job
        let mut job1 = Job::default();
        job1.job_reference.job_id = format!("rust_test_query_{}", OffsetDateTime::now_utc().unix_timestamp());
        job1.job_reference.project_id = project.to_string();
        job1.job_reference.location = Some("asia-northeast1".to_string());
        job1.configuration = JobConfiguration {
            job: JobType::Query(JobConfigurationQuery {
                use_legacy_sql: Some(false),
                query: "SELECT * FROM rust_test_job.table_data_1681472944".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let job1 = client.create(&job1).await.unwrap();
        assert!(job1.status.errors.is_none());
        assert!(job1.status.error_result.is_none());
        assert_eq!(job1.status.state, JobState::Done);
        assert_eq!(
            job1.statistics.unwrap().query.unwrap().statement_type.unwrap().as_str(),
            "SELECT"
        );

        // load job
        let mut job1 = Job::default();
        job1.job_reference.job_id = format!("rust_test_load_{}", OffsetDateTime::now_utc().unix_timestamp());
        job1.job_reference.project_id = project.to_string();
        job1.job_reference.location = Some("asia-northeast1".to_string());
        job1.configuration = JobConfiguration {
            job: JobType::Load(JobConfigurationLoad {
                source_uris: vec!["gs://rust-bq-test/external_data.csv".to_string()],
                source_format: Some(SourceFormat::Csv),
                field_delimiter: Some("|".to_string()),
                encoding: Some("UTF-8".to_string()),
                skip_leading_rows: Some(0),
                autodetect: Some(true),
                write_disposition: Some(WriteDisposition::WriteTruncate),
                destination_table: TableReference {
                    project_id: project.to_string(),
                    dataset_id: "rust_test_job".to_string(),
                    table_id: "rust_test_load_result".to_string(),
                },
                ..Default::default()
            }),
            ..Default::default()
        };
        let job1 = client.create(&job1).await.unwrap();
        assert!(job1.status.errors.is_none());
        assert!(job1.status.error_result.is_none());
        assert!(job1.status.state == JobState::Running || job1.status.state == JobState::Done);

        // copy job
        let mut job2 = Job::default();
        job2.job_reference.job_id = format!("rust_test_copy_{}", OffsetDateTime::now_utc().unix_timestamp());
        job2.job_reference.project_id = project.to_string();
        job2.job_reference.location = Some("asia-northeast1".to_string());
        job2.configuration = JobConfiguration {
            job: JobType::Copy(JobConfigurationTableCopy {
                source_table: JobConfigurationSourceTable::SourceTable(TableReference {
                    project_id: project.to_string(),
                    dataset_id: "rust_test_job".to_string(),
                    table_id: "rust_test_load_result".to_string(),
                }),
                destination_table: TableReference {
                    project_id: project.to_string(),
                    dataset_id: "rust_test_job".to_string(),
                    table_id: "rust_test_load_result_copy".to_string(),
                },
                create_disposition: Some(CreateDisposition::CreateIfNeeded),
                write_disposition: Some(WriteDisposition::WriteTruncate),
                operation_type: Some(OperationType::Copy),
                ..Default::default()
            }),
            ..Default::default()
        };
        let job2 = client.create(&job2).await.unwrap();
        assert!(job2.status.errors.is_none());
        assert!(job2.status.error_result.is_none());
        assert!(job2.status.state == JobState::Running || job2.status.state == JobState::Done);

        // extract table job
        let mut job3 = Job::default();
        job3.job_reference.job_id = format!("rust_test_extract_{}", OffsetDateTime::now_utc().unix_timestamp());
        job3.job_reference.project_id = project.to_string();
        job3.job_reference.location = Some("asia-northeast1".to_string());
        job3.configuration = JobConfiguration {
            job: JobType::Extract(JobConfigurationExtract {
                destination_uris: vec!["gs://rust-bq-test/extracted_data.json".to_string()],
                destination_format: Some(DestinationFormat::NewlineDelimitedJson),
                source: JobConfigurationExtractSource::SourceTable(TableReference {
                    project_id: project.to_string(),
                    dataset_id: "rust_test_job".to_string(),
                    table_id: "rust_test_load_result".to_string(),
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        let job3 = client.create(&job3).await.unwrap();
        assert!(job3.status.errors.is_none());
        assert!(job3.status.error_result.is_none());
        assert!(job3.status.state == JobState::Running || job3.status.state == JobState::Done);

        // cancel
        let cancelled = client
            .cancel(
                job3.job_reference.project_id.as_str(),
                job3.job_reference.job_id.as_str(),
                &CancelJobRequest {
                    location: job3.job_reference.location,
                },
            )
            .await
            .unwrap();
        assert!(cancelled.job.status.state == JobState::Running || cancelled.job.status.state == JobState::Done);
    }

    #[tokio::test]
    #[serial]
    pub async fn query() {
        let (client, project) = create_client().await;
        let client = Arc::new(client);
        let table_client = BigqueryTableClient::new(client.clone());
        let tabledata_client = BigqueryTabledataClient::new(client.clone());

        // insert test data
        let mut table1 = Table::default();
        table1.table_reference.dataset_id = "rust_test_job".to_string();
        table1.table_reference.project_id = project.to_string();
        table1.table_reference.table_id = format!("table_data_{}", OffsetDateTime::now_utc().unix_timestamp());
        table1.schema = Some(create_table_schema());
        let table1 = table_client.create(&table1).await.unwrap();
        let ref1 = table1.table_reference;

        // json value
        let mut req = InsertAllRequest::<TestData>::default();
        for i in 0..3 {
            req.rows.push(Row {
                insert_id: None,
                json: TestData {
                    col_string: Some(format!("test{}", i)),
                    col_number: Some(1),
                    col_number_array: vec![10, 11, 12],
                    col_timestamp: Some(OffsetDateTime::now_utc()),
                    col_json: Some("{\"field\":100}".to_string()),
                    col_json_array: vec!["{\"field\":100}".to_string(), "{\"field\":200}".to_string()],
                    col_struct: Some(TestDataStruct {
                        f1: true,
                        f2: vec![3, 4],
                    }),
                    col_struct_array: vec![TestDataStruct {
                        f1: true,
                        f2: vec![3, 4],
                    }],
                },
            });
        }
        let res = tabledata_client
            .insert(ref1.project_id.as_str(), ref1.dataset_id.as_str(), ref1.table_id.as_str(), &req)
            .await
            .unwrap();
        assert!(res.insert_errors.is_none());

        // query
        let client = BigqueryJobClient::new(client);
        let result = client
            .query(
                project.as_str(),
                &QueryRequest {
                    max_results: Some(2),
                    query: format!("SELECT * FROM rust_test_job.{}", ref1.table_id.as_str()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert!(result.page_token.is_some());
        assert_eq!(result.rows.unwrap().len(), 2);
        assert_eq!(result.total_rows.unwrap(), 3);
        assert_eq!(result.total_bytes_processed, 0);
        assert!(result.job_complete);

        // query all results
        let mut page_token = result.page_token;
        let location = result.job_reference.location;
        loop {
            let query_results = client
                .get_query_results(
                    result.job_reference.project_id.as_str(),
                    result.job_reference.job_id.as_str(),
                    &GetQueryResultsRequest {
                        page_token,
                        location: location.clone(),
                        ..Default::default()
                    },
                )
                .await
                .unwrap();
            assert_eq!(query_results.rows.unwrap().len(), 1);
            assert_eq!(query_results.total_rows, 3);
            if query_results.page_token.is_none() {
                break;
            }
            page_token = query_results.page_token
        }

        // dry run
        let result = client
            .query(
                project.as_str(),
                &QueryRequest {
                    dry_run: Some(true),
                    max_results: Some(10),
                    query: format!("SELECT * FROM rust_test_job.{}", ref1.table_id.as_str()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert!(result.job_reference.job_id.is_empty());
        assert!(result.total_rows.is_none());
        assert_eq!(result.total_bytes_processed, 0);
        assert!(result.job_complete);
    }

    #[tokio::test]
    #[serial]
    pub async fn get_model_training_result() {
        let (client, project) = create_client().await;
        let client = Arc::new(client);
        let client = BigqueryJobClient::new(client);
        let job = client
            .get(
                project.as_str(),
                "bquxjob_2314a540_187c62eab1d",
                &GetJobRequest {
                    location: Some("US".to_string()),
                },
            )
            .await
            .unwrap();
        let statistics = job.statistics.unwrap().query.unwrap().ml_statistics;
        let ml = statistics.unwrap();
        assert_eq!(ml.training_type, TrainingType::SingleTraining);
        assert_eq!(ml.model_type, ModelType::LogisticRegression);
        assert_eq!(ml.max_iterations, Some(15));
    }
}
