use crate::http::bigquery_client::BigqueryClient;
use crate::http::error::Error;
use crate::http::job;

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
    pub async fn query(&self, project_id: &str, data: &QueryRequest) -> Result<QueryResponse, Error> {
        let builder = job::query::build(self.inner.endpoint(), self.inner.http(), project_id, data);
        self.inner.send(builder).await
    }
}

#[cfg(test)]
mod test {
    use crate::http::bigquery_client::test::create_client;

    use crate::http::bigquery_job_client::BigqueryJobClient;
    use crate::http::job::query::QueryRequest;
    use crate::http::job::{
        CreateDisposition, Job, JobConfiguration, JobConfigurationExtract, JobConfigurationExtractSource,
        JobConfigurationLoad, JobConfigurationQuery, JobConfigurationSourceTable, JobConfigurationTableCopy, JobType,
        OperationType, WriteDisposition,
    };
    use crate::http::table::{DestinationFormat, SourceFormat, TableReference};
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
        assert_eq!(job1.status.state, "DONE");
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
        assert_eq!(job1.status.state, "DONE");
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
        assert!(job1.status.state == "RUNNING" || job1.status.state == "DONE");

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
        assert!(job2.status.state == "RUNNING" || job2.status.state == "DONE");

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
        assert!(job3.status.state == "RUNNING" || job3.status.state == "DONE");
    }

    #[tokio::test]
    #[serial]
    pub async fn query() {
        let (client, project) = create_client().await;
        let client = Arc::new(client);
        let client = BigqueryJobClient::new(client);
        let result = client
            .query(
                project.as_str(),
                &QueryRequest {
                    query: "SELECT * FROM rust_test_job.table_data_1681472944".to_string(),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert_eq!(result.total_rows, 0);
        assert_eq!(result.total_bytes_processed, 0);
        assert!(result.job_complete);
        assert!(result.cache_hit);
    }
}
