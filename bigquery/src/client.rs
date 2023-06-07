use crate::arrow::ArrowStructDecodable;

use crate::grpc::apiv1::conn_pool::{ReadConnectionManager, DOMAIN};
use crate::http::bigquery_client::BigqueryClient;
use crate::http::bigquery_dataset_client::BigqueryDatasetClient;
use crate::http::bigquery_job_client::BigqueryJobClient;
use crate::http::bigquery_routine_client::BigqueryRoutineClient;
use crate::http::bigquery_table_client::BigqueryTableClient;
use crate::http::bigquery_tabledata_client::BigqueryTabledataClient;
use crate::http::error::Error;
use crate::http::job::get_query_results::GetQueryResultsRequest;
use crate::http::job::query::QueryRequest;

use crate::http::table::TableReference;
use crate::iterator::{QueryIterator, TableDataError, TableDataIterator};
use google_cloud_gax::conn::Environment;

use google_cloud_gax::retry::RetrySetting;
use google_cloud_googleapis::cloud::bigquery::storage::v1::{
    read_session, CreateReadSessionRequest, DataFormat, ReadSession,
};
use google_cloud_token::{NopeTokenSourceProvider, TokenSourceProvider};
use std::collections::VecDeque;
use std::sync::Arc;
use crate::http::bigquery_row_access_policy_client::BigqueryRowAccessPolicyClient;

#[derive(Debug)]
pub struct ClientConfig {
    pub http: reqwest::Client,
    pub bigquery_endpoint: String,
    pub token_source_provider: Box<dyn TokenSourceProvider>,
    pub project_id: Option<String>,
    pub storage_environment: Environment,
    pub read_connection_size: usize,
}

impl ClientConfig {
    pub fn nope() -> Self {
        Self::new_with_default(Box::new(NopeTokenSourceProvider {}), Box::new(NopeTokenSourceProvider {}))
    }

    pub fn new_with_default(
        http_token_source_provider: Box<dyn TokenSourceProvider>,
        grpc_token_source_provider: Box<dyn TokenSourceProvider>,
    ) -> Self {
        Self {
            http: reqwest::Client::default(),
            bigquery_endpoint: "https://bigquery.googleapis.com".to_string(),
            token_source_provider: http_token_source_provider,
            project_id: None,
            storage_environment: Environment::GoogleCloud(grpc_token_source_provider),
            read_connection_size: 4,
        }
    }
}

pub struct Client {
    dataset_client: BigqueryDatasetClient,
    table_client: BigqueryTableClient,
    tabledata_client: BigqueryTabledataClient,
    job_client: BigqueryJobClient,
    routine_client: BigqueryRoutineClient,
    row_access_policy_client: BigqueryRowAccessPolicyClient,
    streaming_read_client_conn_pool: ReadConnectionManager,
    project_id: String,
}

impl Client {
    /// New client
    pub async fn new(config: ClientConfig) -> Result<Self, google_cloud_gax::conn::Error> {
        let ts = config.token_source_provider.token_source();
        let client = Arc::new(BigqueryClient::new(ts, config.bigquery_endpoint.as_str(), config.http));
        let streaming_read_client_conn_pool =
            ReadConnectionManager::new(config.read_connection_size, &config.storage_environment, DOMAIN).await?;
        Ok(Self {
            dataset_client: BigqueryDatasetClient::new(client.clone()),
            table_client: BigqueryTableClient::new(client.clone()),
            tabledata_client: BigqueryTabledataClient::new(client.clone()),
            job_client: BigqueryJobClient::new(client.clone()),
            routine_client: BigqueryRoutineClient::new(client.clone()),
            row_access_policy_client: BigqueryRowAccessPolicyClient::new(client.clone()),
            streaming_read_client_conn_pool,
            project_id: config.project_id.unwrap_or_default(),
        })
    }

    pub fn dataset(&self) -> &BigqueryDatasetClient {
        &self.dataset_client
    }

    pub fn table(&self) -> &BigqueryTableClient {
        &self.table_client
    }

    pub fn tabledata(&self) -> &BigqueryTabledataClient {
        &self.tabledata_client
    }

    pub fn job(&self) -> &BigqueryJobClient {
        &self.job_client
    }

    pub fn routine(&self) -> &BigqueryRoutineClient {
        &self.routine_client
    }

    pub fn row_access_policy(&self) -> &BigqueryRowAccessPolicyClient{
        &self.row_access_policy_client
    }

    pub async fn query(&self, request: QueryRequest) -> Result<QueryIterator, Error> {
        let result = self.job_client.query(self.project_id.as_str(), &request).await?;
        Ok(QueryIterator {
            client: self.job_client.clone(),
            project_id: result.job_reference.project_id,
            job_id: result.job_reference.job_id,
            request: GetQueryResultsRequest {
                start_index: 0,
                page_token: result.page_token,
                max_results: request.max_results,
                timeout_ms: request.timeout_ms,
                location: Some(request.location),
                format_options: request.format_options,
            },
            chunk: VecDeque::from(result.rows.unwrap_or_default()),
            total_size: result.total_rows.unwrap_or_default(),
        })
    }

    pub async fn read_table<T>(
        &self,
        table: &TableReference,
        option: Option<ReadTableOption>,
    ) -> Result<TableDataIterator<T>, TableDataError>
    where
        T: ArrowStructDecodable<T> + Default,
    {
        let option = option.unwrap_or_default();

        let mut client = self.streaming_read_client_conn_pool.conn();
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
        TableDataIterator::new(client, read_session, option.read_rows_retry_setting).await
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
    use crate::client::{Client, ClientConfig};
    use crate::http::bigquery_client::SCOPES;
    use google_cloud_auth::project::Config;
    use google_cloud_auth::token::DefaultTokenSourceProvider;

    use crate::grpc::apiv1;
    use crate::grpc::apiv1::test::TestData;
    use crate::http::job::query::QueryRequest;
    use crate::http::table::TableReference;
    use crate::value::Row;
    use serial_test::serial;

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt::try_init();
    }

    async fn create_client() -> Client {
        let http_tsp = DefaultTokenSourceProvider::new(Config {
            audience: None,
            scopes: Some(&SCOPES),
            sub: None,
        })
        .await
        .unwrap();
        let grpc_tsp = DefaultTokenSourceProvider::new(Config {
            audience: Some(apiv1::conn_pool::AUDIENCE),
            scopes: Some(&apiv1::conn_pool::SCOPES),
            sub: None,
        })
        .await
        .unwrap();
        let project_id = http_tsp.source_credentials.clone().unwrap().project_id;
        let mut client_config = ClientConfig::new_with_default(Box::new(http_tsp), Box::new(grpc_tsp));
        client_config.project_id = project_id;
        Client::new(client_config).await.unwrap()
    }

    #[tokio::test]
    #[serial]
    async fn test_query() {
        let client = create_client().await;
        let mut iterator = client
            .query(QueryRequest {
                max_results: Some(2),
                query: "SELECT 'A' as col1 ".to_string(),
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(1, iterator.total_size);

        while let Some(row) = iterator.next::<Row>().await.unwrap() {
            let v: &str = row.column(0).unwrap();
            assert_eq!(v, "A");
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_read_table() {
        let client = create_client().await;
        let table = TableReference {
            project_id: "atl-dev1".to_string(),
            dataset_id: "rust_test_table".to_string(),
            table_id: "table_data_1686033753".to_string(),
        };
        let mut iterator = client.read_table::<TestData>(&table, None).await.unwrap();

        let mut data = vec![];
        let mut interrupt = tokio::time::interval(tokio::time::Duration::from_micros(100));
        loop {
            tokio::select! {
                _ = interrupt.tick() => {
                    tracing::info!("interrupt");
                },
                row = iterator.next() => {
                    tracing::info!("read");
                    if let Some(row) = row.unwrap() {
                        data.push(row);
                    }else {
                        break;
                    }
                }
            }
        }
        assert_eq!(data.len(), 34);
    }
}
