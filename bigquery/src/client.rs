use backon::{ExponentialBuilder, Retryable};
use core::time::Duration;
use std::borrow::Cow;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::sync::Arc;

use google_cloud_gax::conn::{ConnectionOptions, Environment};
use google_cloud_gax::retry::RetrySetting;
use google_cloud_googleapis::cloud::bigquery::storage::v1::{
    read_session, CreateReadSessionRequest, DataFormat, ReadSession,
};
use google_cloud_token::TokenSourceProvider;

use crate::grpc::apiv1::conn_pool::ConnectionManager;
use crate::http::bigquery_client::BigqueryClient;
use crate::http::bigquery_dataset_client::BigqueryDatasetClient;
use crate::http::bigquery_job_client::BigqueryJobClient;
use crate::http::bigquery_model_client::BigqueryModelClient;
use crate::http::bigquery_routine_client::BigqueryRoutineClient;
use crate::http::bigquery_row_access_policy_client::BigqueryRowAccessPolicyClient;
use crate::http::bigquery_table_client::BigqueryTableClient;
use crate::http::bigquery_tabledata_client::BigqueryTabledataClient;
use crate::http::job::get_query_results::GetQueryResultsRequest;
use crate::http::job::query::QueryRequest;
use crate::http::job::{is_script, is_select_query, JobConfiguration, JobReference, JobStatistics, JobType};
use crate::http::table::TableReference;
use crate::query::{QueryOption, QueryResult};
use crate::storage;
use crate::{http, query};

const JOB_RETRY_REASONS: [&str; 3] = ["backendError", "rateLimitExceeded", "internalError"];

#[derive(Debug)]
pub struct ClientConfig {
    http: reqwest_middleware::ClientWithMiddleware,
    bigquery_endpoint: Cow<'static, str>,
    token_source_provider: Box<dyn TokenSourceProvider>,
    environment: Environment,
    streaming_read_config: ChannelConfig,
    streaming_write_config: StreamingWriteConfig,
    debug: bool,
}

#[derive(Clone, Debug)]
pub struct StreamingWriteConfig {
    channel_config: ChannelConfig,
    max_insert_count: usize,
}

impl StreamingWriteConfig {
    pub fn with_channel_config(mut self, value: ChannelConfig) -> Self {
        self.channel_config = value;
        self
    }
    pub fn with_max_insert_count(mut self, value: usize) -> Self {
        self.max_insert_count = value;
        self
    }
}

impl Default for StreamingWriteConfig {
    fn default() -> Self {
        Self {
            channel_config: ChannelConfig::default(),
            max_insert_count: 1000,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ChannelConfig {
    /// num_channels is the number of gRPC channels.
    num_channels: usize,
    connect_timeout: Option<Duration>,
    timeout: Option<Duration>,
}

impl ChannelConfig {
    pub fn with_num_channels(mut self, value: usize) -> Self {
        self.num_channels = value;
        self
    }
    pub fn with_connect_timeout(mut self, value: Duration) -> Self {
        self.connect_timeout = Some(value);
        self
    }
    pub fn with_timeout(mut self, value: Duration) -> Self {
        self.timeout = Some(value);
        self
    }

    async fn into_connection_manager(
        self,
        environment: &Environment,
    ) -> Result<ConnectionManager, google_cloud_gax::conn::Error> {
        ConnectionManager::new(
            self.num_channels,
            environment,
            &ConnectionOptions {
                timeout: self.timeout,
                connect_timeout: self.connect_timeout,
            },
        )
        .await
    }
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            num_channels: 4,
            connect_timeout: Some(Duration::from_secs(30)),
            timeout: None,
        }
    }
}

impl ClientConfig {
    pub fn new(
        http_token_source_provider: Box<dyn TokenSourceProvider>,
        grpc_token_source_provider: Box<dyn TokenSourceProvider>,
    ) -> Self {
        Self {
            http: reqwest_middleware::ClientBuilder::new(reqwest::Client::default()).build(),
            bigquery_endpoint: "https://bigquery.googleapis.com".into(),
            token_source_provider: http_token_source_provider,
            environment: Environment::GoogleCloud(grpc_token_source_provider),
            streaming_read_config: ChannelConfig::default(),
            streaming_write_config: StreamingWriteConfig::default(),
            debug: false,
        }
    }
    pub fn with_debug(mut self, value: bool) -> Self {
        self.debug = value;
        self
    }
    pub fn with_streaming_read_config(mut self, value: ChannelConfig) -> Self {
        self.streaming_read_config = value;
        self
    }
    pub fn with_streaming_write_config(mut self, value: StreamingWriteConfig) -> Self {
        self.streaming_write_config = value;
        self
    }
    pub fn with_http_client(mut self, value: reqwest_middleware::ClientWithMiddleware) -> Self {
        self.http = value;
        self
    }
    pub fn with_endpoint(mut self, value: impl Into<Cow<'static, str>>) -> Self {
        self.bigquery_endpoint = value.into();
        self
    }
}

use crate::http::job::get::GetJobRequest;
use crate::http::job::list::ListJobsRequest;

use crate::grpc::apiv1::bigquery_client::StreamingReadClient;
use crate::storage_write::stream::{buffered, committed, default, pending};
#[cfg(feature = "auth")]
pub use google_cloud_auth;
use google_cloud_googleapis::cloud::bigquery::storage::v1::big_query_read_client::BigQueryReadClient;

#[cfg(feature = "auth")]
impl ClientConfig {
    pub async fn new_with_auth() -> Result<(Self, Option<String>), google_cloud_auth::error::Error> {
        let ts_http =
            google_cloud_auth::token::DefaultTokenSourceProvider::new(Self::bigquery_http_auth_config()).await?;
        let ts_grpc =
            google_cloud_auth::token::DefaultTokenSourceProvider::new(Self::bigquery_grpc_auth_config()).await?;
        let project_id = ts_grpc.project_id.clone();
        let config = Self::new(Box::new(ts_http), Box::new(ts_grpc));
        Ok((config, project_id))
    }

    pub async fn new_with_credentials(
        credentials: google_cloud_auth::credentials::CredentialsFile,
    ) -> Result<(Self, Option<String>), google_cloud_auth::error::Error> {
        let ts_http = google_cloud_auth::token::DefaultTokenSourceProvider::new_with_credentials(
            Self::bigquery_http_auth_config(),
            Box::new(credentials.clone()),
        )
        .await?;
        let ts_grpc = google_cloud_auth::token::DefaultTokenSourceProvider::new_with_credentials(
            Self::bigquery_grpc_auth_config(),
            Box::new(credentials),
        )
        .await?;
        let project_id = ts_grpc.project_id.clone();
        let config = Self::new(Box::new(ts_http), Box::new(ts_grpc));
        Ok((config, project_id))
    }

    fn bigquery_http_auth_config() -> google_cloud_auth::project::Config<'static> {
        google_cloud_auth::project::Config::default().with_scopes(&http::bigquery_client::SCOPES)
    }

    fn bigquery_grpc_auth_config() -> google_cloud_auth::project::Config<'static> {
        google_cloud_auth::project::Config::default()
            .with_audience(crate::grpc::apiv1::conn_pool::AUDIENCE)
            .with_scopes(&crate::grpc::apiv1::conn_pool::SCOPES)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum QueryError {
    #[error(transparent)]
    Storage(#[from] storage::Error),
    #[error(transparent)]
    JobHttp(#[from] http::error::Error),
    #[error("job has no destination table to read : job={0:?}")]
    NoDestinationTable(JobReference),
    #[error("failed to resolve table for script job: no child jobs found : job={0:?}")]
    NoChildJobs(JobReference),
    #[error("job type must be query: job={0:?}, jobType={1:?}")]
    InvalidJobType(JobReference, String),
    #[error(transparent)]
    RunQuery(#[from] query::run::Error),
}

#[derive(Clone)]
pub struct Client {
    dataset_client: BigqueryDatasetClient,
    table_client: BigqueryTableClient,
    tabledata_client: BigqueryTabledataClient,
    job_client: BigqueryJobClient,
    routine_client: BigqueryRoutineClient,
    row_access_policy_client: BigqueryRowAccessPolicyClient,
    model_client: BigqueryModelClient,
    streaming_read_conn_pool: Arc<ConnectionManager>,
    streaming_write_conn_pool: Arc<ConnectionManager>,
    stereaming_write_max_insert_count: usize,
}

impl Client {
    /// New client
    pub async fn new(config: ClientConfig) -> Result<Self, google_cloud_gax::conn::Error> {
        let ts = config.token_source_provider.token_source();
        let client = Arc::new(BigqueryClient::new(
            ts,
            config.bigquery_endpoint.into_owned().as_str(),
            config.http,
            config.debug,
        ));

        Ok(Self {
            dataset_client: BigqueryDatasetClient::new(client.clone()),
            table_client: BigqueryTableClient::new(client.clone()),
            tabledata_client: BigqueryTabledataClient::new(client.clone()),
            job_client: BigqueryJobClient::new(client.clone()),
            routine_client: BigqueryRoutineClient::new(client.clone()),
            row_access_policy_client: BigqueryRowAccessPolicyClient::new(client.clone()),
            model_client: BigqueryModelClient::new(client.clone()),
            streaming_read_conn_pool: Arc::new(
                config
                    .streaming_read_config
                    .into_connection_manager(&config.environment)
                    .await?,
            ),
            streaming_write_conn_pool: Arc::new(
                config
                    .streaming_write_config
                    .channel_config
                    .into_connection_manager(&config.environment)
                    .await?,
            ),
            stereaming_write_max_insert_count: config.streaming_write_config.max_insert_count,
        })
    }

    /// https://cloud.google.com/bigquery/docs/reference/rest/v2/datasets
    /// [BigqueryDatasetClient](crate::http::bigquery_dataset_client::BigqueryDatasetClient)
    pub fn dataset(&self) -> &BigqueryDatasetClient {
        &self.dataset_client
    }

    /// https://cloud.google.com/bigquery/docs/reference/rest/v2/tables
    /// [BigqueryTableClient](crate::http::bigquery_table_client::BigqueryTableClient)
    pub fn table(&self) -> &BigqueryTableClient {
        &self.table_client
    }

    /// https://cloud.google.com/bigquery/docs/reference/rest/v2/tabledata
    /// [BigqueryTabledataClient](crate::http::bigquery_tabledata_client::BigqueryTabledataClient)
    pub fn tabledata(&self) -> &BigqueryTabledataClient {
        &self.tabledata_client
    }

    /// https://cloud.google.com/bigquery/docs/reference/rest/v2/jobs
    /// [BigqueryJobClient](crate::http::bigquery_job_client::BigqueryJobClient)
    pub fn job(&self) -> &BigqueryJobClient {
        &self.job_client
    }

    /// https://cloud.google.com/bigquery/docs/reference/rest/v2/routines
    /// [BigqueryRoutineClient](crate::http::bigquery_routine_client::BigqueryRoutineClient)
    pub fn routine(&self) -> &BigqueryRoutineClient {
        &self.routine_client
    }

    /// https://cloud.google.com/bigquery/docs/reference/rest/v2/rowAccessPolicy
    /// [BigqueryRowAccessPolicyClient](crate::http::bigquery_row_access_policy_client::BigqueryRowAccessPolicyClient)
    pub fn row_access_policy(&self) -> &BigqueryRowAccessPolicyClient {
        &self.row_access_policy_client
    }

    /// https://cloud.google.com/bigquery/docs/reference/rest/v2/models
    /// [BigqueryModelClient](crate::http::bigquery_model_client::BigqueryModelClient)
    pub fn model(&self) -> &BigqueryModelClient {
        &self.model_client
    }

    /// Creates a new pending type storage writer for the specified table.
    /// https://cloud.google.com/bigquery/docs/write-api#pending_type
    pub fn pending_storage_writer(&self, table: String) -> pending::Writer {
        pending::Writer::new(1, self.streaming_write_conn_pool.clone(), table)
    }

    /// Creates a new default type storage writer.
    /// https://cloud.google.com/bigquery/docs/write-api#default_stream
    pub fn default_storage_writer(&self) -> default::Writer {
        default::Writer::new(self.stereaming_write_max_insert_count, self.streaming_write_conn_pool.clone())
    }

    /// Creates a new committed type storage writer.
    /// https://cloud.google.com/bigquery/docs/write-api#committed_type
    pub fn committed_storage_writer(&self) -> committed::Writer {
        committed::Writer::new(self.stereaming_write_max_insert_count, self.streaming_write_conn_pool.clone())
    }

    /// Creates a new buffered type storage writer.
    /// https://cloud.google.com/bigquery/docs/write-api#buffered_type
    pub fn buffered_storage_writer(&self) -> buffered::Writer {
        buffered::Writer::new(self.stereaming_write_max_insert_count, self.streaming_write_conn_pool.clone())
    }

    /// Run query job and get result.
    /// ```rust
    /// use google_cloud_bigquery::http::job::query::QueryRequest;
    /// use google_cloud_bigquery::query::row::Row;
    /// use google_cloud_bigquery::client::Client;
    ///
    /// async fn run(client: &Client, project_id: &str) {
    ///     let request = QueryRequest {
    ///         query: "SELECT * FROM dataset.table".to_string(),
    ///         ..Default::default()
    ///     };
    ///     let mut iter = client.query::<Row>(project_id, request).await.unwrap();
    ///     while let Some(row) = iter.next().await.unwrap() {
    ///         let col1 = row.column::<String>(0);
    ///         let col2 = row.column::<Option<String>>(1);
    ///     }
    /// }
    pub async fn query<T>(&self, project_id: &str, request: QueryRequest) -> Result<query::Iterator<T>, QueryError>
    where
        T: http::query::value::StructDecodable + storage::value::StructDecodable,
    {
        self.query_with_option(project_id, request, QueryOption::default())
            .await
    }

    /// Run query job and get result.
    /// ```rust
    /// use google_cloud_bigquery::http::job::query::QueryRequest;
    /// use google_cloud_bigquery::query::row::Row;
    /// use google_cloud_bigquery::client::Client;
    /// use google_cloud_bigquery::query::QueryOption;
    /// use google_cloud_bigquery::query::ExponentialBuilder;
    ///
    /// async fn run(client: &Client, project_id: &str) {
    ///     let request = QueryRequest {
    ///         query: "SELECT * FROM dataset.table".to_string(),
    ///         ..Default::default()
    ///     };
    ///     let retry = ExponentialBuilder::default().with_max_times(10);
    ///     let option = QueryOption::default().with_retry(retry).with_enable_storage_read(true);
    ///     let mut iter = client.query_with_option::<Row>(project_id, request, option).await.unwrap();
    ///     while let Some(row) = iter.next().await.unwrap() {
    ///         let col1 = row.column::<String>(0);
    ///         let col2 = row.column::<Option<String>>(1);
    ///     }
    /// }
    pub async fn query_with_option<T>(
        &self,
        project_id: &str,
        request: QueryRequest,
        option: QueryOption,
    ) -> Result<query::Iterator<T>, QueryError>
    where
        T: http::query::value::StructDecodable + storage::value::StructDecodable,
    {
        let result = self.job_client.query(project_id, &request).await?;
        let (total_rows, page_token, rows, force_first_fetch) = if result.job_complete {
            (
                result.total_rows.unwrap_or_default(),
                result.page_token,
                result.rows.unwrap_or_default(),
                false,
            )
        } else {
            (
                self.wait_for_query(&result.job_reference, option.retry, &request.timeout_ms)
                    .await?,
                None,
                vec![],
                true,
            )
        };

        //use storage api instead of rest API
        if option.enable_storage_read && (page_token.is_none() || page_token.as_ref().unwrap().is_empty()) {
            tracing::trace!("use storage read api for query {:?}", result.job_reference);
            let job = self
                .job_client
                .get(
                    &result.job_reference.project_id,
                    &result.job_reference.job_id,
                    &GetJobRequest {
                        location: result.job_reference.location.clone(),
                    },
                )
                .await?;
            let iter = self
                .new_storage_row_iterator_from_job::<T>(job.job_reference, job.statistics, job.configuration)
                .await?;
            return Ok(query::Iterator {
                inner: QueryResult::Storage(iter),
                total_size: total_rows,
            });
        }

        let http_query_iterator = http::query::Iterator {
            client: self.job_client.clone(),
            project_id: result.job_reference.project_id,
            job_id: result.job_reference.job_id,
            request: GetQueryResultsRequest {
                start_index: 0,
                page_token,
                max_results: request.max_results,
                timeout_ms: request.timeout_ms,
                location: result.job_reference.location,
                format_options: request.format_options,
            },
            chunk: VecDeque::from(rows),
            total_size: total_rows,
            force_first_fetch,
            _marker: PhantomData,
        };
        Ok(query::Iterator {
            inner: QueryResult::Http(http_query_iterator),
            total_size: total_rows,
        })
    }

    async fn new_storage_row_iterator_from_job<T>(
        &self,
        mut job: JobReference,
        mut statistics: Option<JobStatistics>,
        mut config: JobConfiguration,
    ) -> Result<storage::Iterator<T>, QueryError>
    where
        T: http::query::value::StructDecodable + storage::value::StructDecodable,
    {
        loop {
            tracing::trace!("check child job result {:?}, {:?}, {:?}", job, statistics, config);
            let query_config = match &config.job {
                JobType::Query(config) => config,
                _ => return Err(QueryError::InvalidJobType(job.clone(), config.job_type.clone())),
            };
            if let Some(dst) = &query_config.destination_table {
                return Ok(self.read_table(dst, None).await?);
            }
            if !is_script(&statistics, &config) {
                return Err(QueryError::NoDestinationTable(job.clone()));
            }
            let children = self
                .job_client
                .list(
                    &job.project_id,
                    &ListJobsRequest {
                        parent_job_id: job.job_id.to_string(),
                        ..Default::default()
                    },
                )
                .await?;

            let mut found = false;
            for j in children.into_iter() {
                if !is_select_query(&j.statistics, &j.configuration) {
                    continue;
                }
                job = j.job_reference;
                statistics = j.statistics;
                config = j.configuration;
                found = true;
                break;
            }
            if !found {
                break;
            }
        }
        Err(QueryError::NoChildJobs(job.clone()))
    }

    async fn wait_for_query(
        &self,
        job: &JobReference,
        builder: ExponentialBuilder,
        timeout_ms: &Option<i64>,
    ) -> Result<i64, query::run::Error> {
        // Use get_query_results only to wait for completion, not to read results.
        let request = GetQueryResultsRequest {
            max_results: Some(0),
            timeout_ms: *timeout_ms,
            location: job.location.clone(),
            ..Default::default()
        };
        let action = || async {
            tracing::debug!("waiting for job completion {:?}", job);
            let result = self
                .job_client
                .get_query_results(&job.project_id, &job.job_id, &request)
                .await
                .map_err(query::run::Error::Http)?;
            if result.job_complete {
                Ok(result.total_rows)
            } else {
                Err(query::run::Error::JobIncomplete)
            }
        };
        action
            .retry(builder)
            .when(|e: &query::run::Error| match e {
                query::run::Error::JobIncomplete => true,
                query::run::Error::Http(http::error::Error::HttpClient(_)) => true,
                query::run::Error::Http(http::error::Error::Response(r)) => r.is_retryable(&JOB_RETRY_REASONS),
                _ => false,
            })
            .await
    }

    /// Read table data by BigQuery Storage Read API.
    /// ```rust
    /// use google_cloud_bigquery::storage::row::Row;
    /// use google_cloud_bigquery::client::Client;
    /// use google_cloud_bigquery::http::table::TableReference;
    ///
    /// async fn run(client: &Client, project_id: &str) {
    ///     let table = TableReference {
    ///         project_id: project_id.to_string(),
    ///         dataset_id: "dataset".to_string(),
    ///         table_id: "table".to_string(),
    ///     };
    ///     let mut iter = client.read_table::<Row>(&table, None).await.unwrap();
    ///     while let Some(row) = iter.next().await.unwrap() {
    ///         let col1 = row.column::<String>(0);
    ///         let col2 = row.column::<Option<String>>(1);
    ///     }
    /// }
    /// ```
    pub async fn read_table<T>(
        &self,
        table: &TableReference,
        option: Option<ReadTableOption>,
    ) -> Result<storage::Iterator<T>, storage::Error>
    where
        T: storage::value::StructDecodable,
    {
        let option = option.unwrap_or_default();

        let mut client = StreamingReadClient::new(BigQueryReadClient::new(self.streaming_read_conn_pool.conn()));
        let read_session = client
            .create_read_session(
                CreateReadSessionRequest {
                    parent: format!("projects/{}", table.project_id),
                    read_session: Some(ReadSession {
                        name: "".to_string(),
                        expire_time: None,
                        data_format: DataFormat::Arrow.into(),
                        table: table.resource(),
                        table_modifiers: option.session_table_modifiers,
                        read_options: option.session_read_options,
                        streams: vec![],
                        estimated_total_bytes_scanned: 0,
                        estimated_total_physical_file_size: 0,
                        estimated_row_count: 0,
                        trace_id: "".to_string(),
                        schema: option.session_schema,
                    }),
                    max_stream_count: 0,
                    preferred_min_stream_count: 0,
                },
                option.session_retry_setting,
            )
            .await?
            .into_inner();
        storage::Iterator::new(client, read_session, option.read_rows_retry_setting).await
    }
}

#[derive(Debug, Default, Clone)]
pub struct ReadTableOption {
    session_read_options: Option<read_session::TableReadOptions>,
    session_table_modifiers: Option<read_session::TableModifiers>,
    session_schema: Option<read_session::Schema>,
    session_retry_setting: Option<RetrySetting>,
    read_rows_retry_setting: Option<RetrySetting>,
}

impl ReadTableOption {
    pub fn with_session_read_options(mut self, value: read_session::TableReadOptions) -> Self {
        self.session_read_options = Some(value);
        self
    }

    pub fn with_session_table_modifiers(mut self, value: read_session::TableModifiers) -> Self {
        self.session_table_modifiers = Some(value);
        self
    }

    pub fn with_session_schema(mut self, value: read_session::Schema) -> Self {
        self.session_schema = Some(value);
        self
    }

    pub fn with_session_retry_setting(mut self, value: RetrySetting) -> Self {
        self.session_retry_setting = Some(value);
        self
    }

    pub fn with_read_rows_retry_setting(mut self, value: RetrySetting) -> Self {
        self.read_rows_retry_setting = Some(value);
        self
    }
}

#[cfg(test)]
mod tests {
    use bigdecimal::BigDecimal;
    use serial_test::serial;
    use std::collections::HashMap;
    use std::ops::AddAssign;
    use std::time::Duration;

    use time::{Date, OffsetDateTime, Time};

    use google_cloud_googleapis::cloud::bigquery::storage::v1::read_session::TableReadOptions;

    use crate::client::{Client, ClientConfig, ReadTableOption};
    use crate::http::bigquery_client::test::{create_table_schema, dataset_name, TestData};
    use crate::http::job::query::QueryRequest;
    use crate::http::table::{Table, TableReference};
    use crate::http::tabledata::insert_all::{InsertAllRequest, Row};
    use crate::http::types::{QueryParameter, QueryParameterStructType, QueryParameterType, QueryParameterValue};
    use crate::query;
    use crate::query::QueryOption;

    #[ctor::ctor]
    fn init() {
        let filter = tracing_subscriber::filter::EnvFilter::from_default_env()
            .add_directive("google_cloud_bigquery=trace".parse().unwrap());
        let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
    }

    async fn create_client() -> (Client, String) {
        let (client_config, project_id) = ClientConfig::new_with_auth().await.unwrap();
        (Client::new(client_config).await.unwrap(), project_id.unwrap())
    }

    #[tokio::test]
    #[serial]
    async fn test_query_from_storage() {
        let option = QueryOption::default().with_enable_storage_read(true);
        test_query(option).await
    }

    #[tokio::test]
    #[serial]
    async fn test_query_from_rest() {
        let option = QueryOption::default();
        test_query(option).await
    }

    async fn test_query(option: QueryOption) {
        let (client, project_id) = create_client().await;
        let mut iterator = client
            .query_with_option::<query::row::Row>(
                &project_id,
                QueryRequest {
                    max_results: Some(2),
                    query: "SELECT
                        'A',
                        TIMESTAMP_MICROS(1230219000000019),
                        100,
                        0.432899,
                        DATE(2023,9,1),
                        TIME(15, 30, 01),
                        NULL,
                        ['A','B'],
                        [TIMESTAMP_MICROS(1230219000000019), TIMESTAMP_MICROS(1230219000000020)],
                        [100,200],
                        [0.432899,0.432900],
                        [DATE(2023,9,1),DATE(2023,9,2)],
                        [TIME_ADD(TIME(15,30,1), INTERVAL 10 MICROSECOND),TIME(0, 0, 0),TIME(23,59,59)],
                        b'test',
                        true,
                        [b'test',b'test2'],
                        [true,false],
                        cast('-5.7896044618658097711785492504343953926634992332820282019728792003956564819968E+38' as BIGNUMERIC),
                        cast('5.7896044618658097711785492504343953926634992332820282019728792003956564819967E+38' as BIGNUMERIC),
                        cast('-9.9999999999999999999999999999999999999E+28' as NUMERIC),
                        cast('9.9999999999999999999999999999999999999E+28' as NUMERIC),
                        [cast('-5.7896044618658097711785492504343953926634992332820282019728792003956564819968E+38' as BIGNUMERIC),cast('5.7896044618658097711785492504343953926634992332820282019728792003956564819967E+38' as BIGNUMERIC)]
                    ".to_string(),
                    ..Default::default()
                },
                option,
            )
            .await
            .unwrap();

        assert_eq!(1, iterator.total_size);

        while let Some(row) = iterator.next().await.unwrap() {
            let v: String = row.column(0).unwrap();
            assert_eq!(v, "A");
            let v: OffsetDateTime = row.column(1).unwrap();
            assert_eq!(v.unix_timestamp_nanos(), 1230219000000019000);
            let v: i64 = row.column(2).unwrap();
            assert_eq!(v, 100);
            let v: f64 = row.column(3).unwrap();
            assert_eq!(v, 0.432899);
            let v: Date = row.column(4).unwrap();
            assert_eq!(v, time::macros::date!(2023 - 09 - 01));
            let v: Time = row.column(5).unwrap();
            assert_eq!(v, time::macros::time!(15:30:01));
            let v: Option<String> = row.column(6).unwrap();
            assert!(v.is_none());
            let v: Option<OffsetDateTime> = row.column(6).unwrap();
            assert!(v.is_none());
            let v: Option<i64> = row.column(6).unwrap();
            assert!(v.is_none());
            let v: Option<f64> = row.column(6).unwrap();
            assert!(v.is_none());
            let v: Option<Date> = row.column(6).unwrap();
            assert!(v.is_none());
            let v: Option<Time> = row.column(6).unwrap();
            assert!(v.is_none());
            let v: Option<Vec<Time>> = row.column(6).unwrap();
            assert!(v.is_none());
            let v: Option<BigDecimal> = row.column(6).unwrap();
            assert!(v.is_none());
            let v: Option<bool> = row.column(6).unwrap();
            assert!(v.is_none());
            let v: Option<String> = row.column(6).unwrap();
            assert!(v.is_none());
            let v: Option<Vec<u8>> = row.column(6).unwrap();
            assert!(v.is_none());

            let v: Vec<String> = row.column(7).unwrap();
            assert_eq!(v, vec!["A", "B"]);
            let v: Vec<OffsetDateTime> = row.column(8).unwrap();
            assert_eq!(v[0].unix_timestamp_nanos(), 1230219000000019000);
            assert_eq!(v[1].unix_timestamp_nanos(), 1230219000000020000);
            let v: Vec<i64> = row.column(9).unwrap();
            assert_eq!(v, vec![100, 200]);
            let v: Vec<f64> = row.column(10).unwrap();
            assert_eq!(v, vec![0.432899, 0.432900]);
            let v: Vec<Date> = row.column(11).unwrap();
            assert_eq!(v[0], time::macros::date!(2023 - 09 - 01));
            assert_eq!(v[1], time::macros::date!(2023 - 09 - 02));
            let v: Vec<Time> = row.column(12).unwrap();
            let mut tm = time::macros::time!(15:30:01);
            tm.add_assign(Duration::from_micros(10));
            assert_eq!(v[0], tm);
            assert_eq!(v[1], time::macros::time!(0:0:0));
            assert_eq!(v[2], time::macros::time!(23:59:59));

            let v: Vec<u8> = row.column(13).unwrap();
            assert_eq!(v, b"test");
            let v: bool = row.column(14).unwrap();
            assert!(v);
            let v: Vec<Vec<u8>> = row.column(15).unwrap();
            assert_eq!(v[0], b"test");
            assert_eq!(v[1], b"test2");
            let v: Vec<bool> = row.column(16).unwrap();
            assert!(v[0]);
            assert!(!v[1]);
            let v: BigDecimal = row.column(17).unwrap();
            assert_eq!(
                v.to_string(),
                "-578960446186580977117854925043439539266.34992332820282019728792003956564819968"
            );
            let v: BigDecimal = row.column(18).unwrap();
            assert_eq!(
                v.to_string(),
                "578960446186580977117854925043439539266.34992332820282019728792003956564819967"
            );
            let v: BigDecimal = row.column(19).unwrap();
            assert_eq!(v.to_string(), "-99999999999999999999999999999.999999999");
            let v: BigDecimal = row.column(20).unwrap();
            assert_eq!(v.to_string(), "99999999999999999999999999999.999999999");
            let v: Vec<BigDecimal> = row.column(21).unwrap();
            assert_eq!(
                v[0].to_string(),
                "-578960446186580977117854925043439539266.34992332820282019728792003956564819968"
            );
            assert_eq!(
                v[1].to_string(),
                "578960446186580977117854925043439539266.34992332820282019728792003956564819967"
            );
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_query_table_from_storage() {
        test_query_table(None, QueryOption::default().with_enable_storage_read(true)).await
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_query_table_from_rest() {
        test_query_table(Some(1), QueryOption::default()).await
    }

    async fn insert(client: &Client, project: &str, dataset: &str, table: &str, size: usize, now: &OffsetDateTime) {
        let mut table1 = Table::default();
        table1.table_reference.dataset_id = dataset.to_string();
        table1.table_reference.project_id = project.to_string();
        table1.table_reference.table_id = table.to_string();
        table1.schema = Some(create_table_schema());
        let _table1 = client.table_client.create(&table1).await.unwrap();
        let mut req = InsertAllRequest::<TestData>::default();
        for i in 0..size {
            req.rows.push(Row {
                insert_id: None,
                json: TestData::default(i, *now + Duration::from_secs(i as u64)),
            });
        }
        client.tabledata().insert(project, dataset, table, &req).await.unwrap();
    }

    async fn test_query_table(max_results: Option<i64>, option: QueryOption) {
        let dataset = dataset_name("table");
        let (client, project_id) = create_client().await;
        let now = OffsetDateTime::from_unix_timestamp(OffsetDateTime::now_utc().unix_timestamp()).unwrap();
        let table = format!("test_query_table_{}", now.unix_timestamp());
        insert(&client, &project_id, &dataset, &table, 3, &now).await;

        // query
        let mut data_as_row: Vec<TestData> = vec![];
        let mut iterator_as_row = client
            .query_with_option::<query::row::Row>(
                &project_id,
                QueryRequest {
                    max_results,
                    query: format!("SELECT * FROM {}.{}", dataset, table),
                    ..Default::default()
                },
                option.clone(),
            )
            .await
            .unwrap();
        while let Some(row) = iterator_as_row.next().await.unwrap() {
            data_as_row.push(TestData {
                col_string: row.column(0).unwrap(),
                col_number: row.column(1).unwrap(),
                col_number_array: row.column(2).unwrap(),
                col_timestamp: row.column(3).unwrap(),
                col_json: row.column(4).unwrap(),
                col_json_array: row.column(5).unwrap(),
                col_struct: row.column(6).unwrap(),
                col_struct_array: row.column(7).unwrap(),
                col_binary: row.column(8).unwrap(),
            });
        }
        let mut data_as_struct: Vec<TestData> = vec![];
        let mut iterator_as_struct = client
            .query_with_option::<TestData>(
                &project_id,
                QueryRequest {
                    query: format!("SELECT * FROM {}.{}", dataset, table),
                    ..Default::default()
                },
                option,
            )
            .await
            .unwrap();
        while let Some(row) = iterator_as_struct.next().await.unwrap() {
            data_as_struct.push(row);
        }
        assert_eq!(iterator_as_struct.total_size, 3);
        assert_eq!(iterator_as_row.total_size, 3);
        assert_eq!(data_as_struct.len(), 3);
        assert_eq!(data_as_row.len(), 3);

        assert_data(&now, data_as_struct);
        assert_data(&now, data_as_row);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_read_table() {
        let dataset = dataset_name("table");
        let (client, project_id) = create_client().await;
        let now = OffsetDateTime::from_unix_timestamp(OffsetDateTime::now_utc().unix_timestamp()).unwrap();
        let table = format!("test_read_table_{}", now.unix_timestamp());
        insert(&client, &project_id, &dataset, &table, 3, &now).await;

        let table = TableReference {
            project_id,
            dataset_id: dataset.to_string(),
            table_id: table.to_string(),
        };
        let mut iterator_as_struct = client.read_table::<TestData>(&table, None).await.unwrap();

        let option = ReadTableOption {
            session_read_options: Some(TableReadOptions {
                row_restriction: "col_string = \"test_0\"".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let mut iterator_as_row = client
            .read_table::<crate::storage::row::Row>(&table, Some(option))
            .await
            .unwrap();
        let mut data_as_row: Vec<TestData> = vec![];
        let mut data_as_struct: Vec<TestData> = vec![];
        let mut finish1 = false;
        let mut finish2 = false;
        loop {
            tokio::select! {
                row = iterator_as_struct.next() => {
                    if let Some(row) = row.unwrap() {
                        tracing::info!("read struct some");
                        data_as_struct.push(row);
                    }else {
                        tracing::info!("read struct none");
                        finish1 = true;
                        if finish1 && finish2 {
                            break;
                        }
                    }
                },
                row = iterator_as_row.next() => {
                    if let Some(row) = row.unwrap() {
                        tracing::info!("read row some");
                        data_as_row.push(TestData {
                            col_string: row.column(0).unwrap(),
                            col_number: row.column(1).unwrap(),
                            col_number_array: row.column(2).unwrap(),
                            col_timestamp: row.column(3).unwrap(),
                            col_json: row.column(4).unwrap(),
                            col_json_array: row.column(5).unwrap(),
                            col_struct: row.column(6).unwrap(),
                            col_struct_array: row.column(7).unwrap(),
                            col_binary: row.column(8).unwrap(),
            }           );
                    }else {
                        tracing::info!("read row none");
                        finish2 = true;
                        if finish1 && finish2 {
                            break;
                        }
                    }
                }
            }
        }
        assert_eq!(data_as_struct.len(), 3);
        assert_eq!(data_as_row.len(), 1);

        assert_data(&now, data_as_struct);
        assert_data(&now, data_as_row);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_query_job_incomplete_from_storage() {
        test_query_job_incomplete(None, QueryOption::default().with_enable_storage_read(true)).await
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_query_job_incomplete_from_rest() {
        test_query_job_incomplete(Some(4999), QueryOption::default()).await
    }

    async fn test_query_job_incomplete(max_results: Option<i64>, option: QueryOption) {
        let dataset = dataset_name("table");
        let (client, project_id) = create_client().await;
        let now = OffsetDateTime::now_utc();
        let table = format!("test_query_job_incomplete_{}", now.unix_timestamp());
        const SIZE: usize = 10000;
        insert(&client, &project_id, &dataset, &table, SIZE, &now).await;

        let mut data: Vec<query::row::Row> = vec![];
        let mut iter = client
            .query_with_option::<query::row::Row>(
                &project_id,
                QueryRequest {
                    timeout_ms: Some(5), // pass wait_for_query
                    use_query_cache: Some(false),
                    max_results,
                    query: format!("SELECT 1 FROM {}.{}", dataset, table),
                    ..Default::default()
                },
                option,
            )
            .await
            .unwrap();
        while let Some(row) = iter.next().await.unwrap() {
            data.push(row);
        }
        assert_eq!(iter.total_size, SIZE as i64);
        assert_eq!(data.len(), SIZE);
    }

    #[derive(Debug, Clone)]
    struct Val {
        pub val1: String,
        pub val2: String,
    }

    #[tokio::test]
    #[serial]
    async fn test_query_with_parameter() {
        let array_val = [
            Val {
                val1: "val1-1".to_string(),
                val2: "val1-2".to_string(),
            },
            Val {
                val1: "val2-1".to_string(),
                val2: "val2-2".to_string(),
            },
        ];

        let query_parameter = QueryParameter {
            name: Some("p1".to_string()),
            parameter_type: QueryParameterType {
                parameter_type: "ARRAY".to_string(),
                array_type: Some(Box::new(QueryParameterType {
                    parameter_type: "STRUCT".to_string(),
                    struct_types: Some(vec![
                        QueryParameterStructType {
                            name: Some("val1".to_string()),
                            field_type: QueryParameterType {
                                parameter_type: "STRING".to_string(),
                                ..Default::default()
                            },
                            description: None,
                        },
                        QueryParameterStructType {
                            name: Some("val2".to_string()),
                            field_type: QueryParameterType {
                                parameter_type: "STRING".to_string(),
                                ..Default::default()
                            },
                            description: None,
                        },
                    ]),
                    array_type: None,
                })),
                struct_types: None,
            },
            parameter_value: QueryParameterValue {
                array_values: Some(
                    array_val
                        .iter()
                        .map(|val| {
                            let mut param_map = HashMap::new();
                            param_map.insert(
                                "val1".to_string(),
                                QueryParameterValue {
                                    value: Some(val.val1.clone()),
                                    ..Default::default()
                                },
                            );
                            param_map.insert(
                                "val2".to_string(),
                                QueryParameterValue {
                                    value: Some(val.val2.clone()),
                                    ..Default::default()
                                },
                            );
                            QueryParameterValue {
                                struct_values: Some(param_map),
                                value: None,
                                array_values: None,
                            }
                        })
                        .collect(),
                ),
                ..Default::default()
            },
        };
        let (client, project_id) = create_client().await;
        let mut result = client
            .query::<query::row::Row>(
                &project_id,
                QueryRequest {
                    query: "
            WITH VAL AS (SELECT @p1 AS col1)
            SELECT
                ARRAY(SELECT val1 FROM UNNEST(col1)) AS val1,
                ARRAY(SELECT val2 FROM UNNEST(col1)) AS val2
            FROM VAL
            "
                    .to_string(),
                    query_parameters: vec![query_parameter],
                    ..QueryRequest::default()
                },
            )
            .await
            .unwrap();
        let row = result.next().await.unwrap().unwrap();
        let col = row.column::<Vec<String>>(0).unwrap();
        assert_eq!(col[0], "val1-1".to_string());
        assert_eq!(col[1], "val2-1".to_string());
        let col = row.column::<Vec<String>>(1).unwrap();
        assert_eq!(col[0], "val1-2".to_string());
        assert_eq!(col[1], "val2-2".to_string());
    }

    fn assert_data(now: &OffsetDateTime, data: Vec<TestData>) {
        for (i, d) in data.iter().enumerate() {
            assert_eq!(&TestData::default(i, *now + Duration::from_secs(i as u64)), d);
        }
    }
}
