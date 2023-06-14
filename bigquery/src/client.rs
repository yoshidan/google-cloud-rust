use crate::arrow::ArrowStructDecodable;
use std::borrow::Cow;

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
use crate::query;
use crate::storage;
use google_cloud_gax::conn::Environment;

use crate::http::bigquery_model_client::BigqueryModelClient;
use crate::http::bigquery_row_access_policy_client::BigqueryRowAccessPolicyClient;
use google_cloud_gax::retry::RetrySetting;
use google_cloud_googleapis::cloud::bigquery::storage::v1::{
    read_session, CreateReadSessionRequest, DataFormat, ReadSession,
};
use google_cloud_token::TokenSourceProvider;
use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Debug)]
pub struct ClientConfig {
    http: reqwest::Client,
    bigquery_endpoint: Cow<'static, str>,
    token_source_provider: Box<dyn TokenSourceProvider>,
    storage_environment: Environment,
    read_connection_size: u8,
    debug: bool,
}

impl ClientConfig {
    pub fn new(
        http_token_source_provider: Box<dyn TokenSourceProvider>,
        grpc_token_source_provider: Box<dyn TokenSourceProvider>,
    ) -> Self {
        Self {
            http: reqwest::Client::default(),
            bigquery_endpoint: "https://bigquery.googleapis.com".into(),
            token_source_provider: http_token_source_provider,
            storage_environment: Environment::GoogleCloud(grpc_token_source_provider),
            read_connection_size: 4,
            debug: false,
        }
    }
    pub fn with_debug(mut self, value: bool) -> Self {
        self.debug = value;
        self
    }
    pub fn with_read_connection_size(mut self, value: u8) -> Self {
        self.read_connection_size = value;
        self
    }
    pub fn with_http_client(mut self, value: reqwest::Client) -> Self {
        self.http = value;
        self
    }
    pub fn with_endpoint(mut self, value: impl Into<Cow<'static, str>>) -> Self {
        self.bigquery_endpoint = value.into();
        self
    }
}

pub struct Client {
    dataset_client: BigqueryDatasetClient,
    table_client: BigqueryTableClient,
    tabledata_client: BigqueryTabledataClient,
    job_client: BigqueryJobClient,
    routine_client: BigqueryRoutineClient,
    row_access_policy_client: BigqueryRowAccessPolicyClient,
    model_client: BigqueryModelClient,
    streaming_read_client_conn_pool: ReadConnectionManager,
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
        let streaming_read_client_conn_pool =
            ReadConnectionManager::new(config.read_connection_size as usize, &config.storage_environment, DOMAIN)
                .await?;
        Ok(Self {
            dataset_client: BigqueryDatasetClient::new(client.clone()),
            table_client: BigqueryTableClient::new(client.clone()),
            tabledata_client: BigqueryTabledataClient::new(client.clone()),
            job_client: BigqueryJobClient::new(client.clone()),
            routine_client: BigqueryRoutineClient::new(client.clone()),
            row_access_policy_client: BigqueryRowAccessPolicyClient::new(client.clone()),
            model_client: BigqueryModelClient::new(client.clone()),
            streaming_read_client_conn_pool,
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

    pub fn row_access_policy(&self) -> &BigqueryRowAccessPolicyClient {
        &self.row_access_policy_client
    }

    pub fn model(&self) -> &BigqueryModelClient {
        &self.model_client
    }

    pub async fn query(&self, project_id: &str, request: QueryRequest) -> Result<query::Iterator, Error> {
        let result = self.job_client.query(project_id, &request).await?;
        Ok(query::Iterator {
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
    ) -> Result<storage::Iterator<T>, storage::Error>
    where
        T: ArrowStructDecodable<T>,
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
    use crate::client::{Client, ClientConfig};
    use crate::http::bigquery_client::SCOPES;
    use bigdecimal::BigDecimal;
    use google_cloud_auth::project::Config;
    use google_cloud_auth::token::DefaultTokenSourceProvider;
    use std::str::FromStr;

    use crate::grpc::apiv1;
    use crate::http::bigquery_client::test::TestData;
    use crate::http::job::query::QueryRequest;
    use crate::http::table::TableReference;
    use crate::query;
    use crate::storage::row::Row;
    use serial_test::serial;
    use time::{Date, OffsetDateTime, Time};

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt::try_init();
    }

    async fn create_client() -> (Client, String) {
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
        let project_id = http_tsp.source_credentials.clone().unwrap().project_id.unwrap();
        let client_config = ClientConfig::new(Box::new(http_tsp), Box::new(grpc_tsp)).with_debug(true);
        (Client::new(client_config).await.unwrap(), project_id)
    }

    #[tokio::test]
    #[serial]
    async fn test_query() {
        let (client, project_id) = create_client().await;
        let mut iterator = client
            .query(
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
                        [TIME(15, 30, 01),TIME(15, 30, 02)],
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
            )
            .await
            .unwrap();

        assert_eq!(1, iterator.total_size);

        while let Some(row) = iterator.next::<query::row::Row>().await.unwrap() {
            let v: &str = row.column(0).unwrap();
            assert_eq!(v, "A");
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
            let v: Option<&str> = row.column(6).unwrap();
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

            let v: Vec<&str> = row.column(7).unwrap();
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
            assert_eq!(v[0], time::macros::time!(15:30:01));
            assert_eq!(v[1], time::macros::time!(15:30:02));

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
    async fn test_read_table() {
        let (client, project_id) = create_client().await;
        let table = TableReference {
            project_id,
            dataset_id: "rust_test_job".to_string(),
            table_id: "table_data_1686707863".to_string(),
        };
        let mut iterator_as_struct = client.read_table::<TestData>(&table, None).await.unwrap();
        let mut iterator_as_row = client.read_table::<Row>(&table, None).await.unwrap();
        let mut data_as_row: Vec<TestData> = vec![];
        let mut data_as_struct: Vec<TestData> = vec![];
        loop {
            tokio::select! {
                row = iterator_as_struct.next() => {
                    tracing::info!("read struct");
                    if let Some(row) = row.unwrap() {
                        data_as_struct.push(row);
                    }else {
                        break;
                    }
                },
                row = iterator_as_row.next() => {
                    tracing::info!("read row");
                    if let Some(row) = row.unwrap() {
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
                        break;
                    }
                }
            }
        }
        assert_eq!(data_as_struct.len(), 3);
        assert_eq!(data_as_row.len(), 3);

        for (i, d) in data_as_struct.iter().enumerate() {
            assert_data(i, d.clone());
        }
        for (i, d) in data_as_row.iter().enumerate() {
            assert_data(i, d.clone());
        }
    }

    async fn assert_data(index: usize, d: TestData) {
        assert_eq!(d.col_string.unwrap(), format!("test_{}", index));
        assert_eq!(
            d.col_number.unwrap(),
            BigDecimal::from_str("-99999999999999999999999999999.999999999").unwrap()
        );
        let col_number_array = &d.col_number_array;
        assert_eq!(
            col_number_array[0],
            BigDecimal::from_str("578960446186580977117854925043439539266.34992332820282019728792003956564819967")
                .unwrap()
        );
        assert_eq!(
            col_number_array[1],
            BigDecimal::from_str("-578960446186580977117854925043439539266.34992332820282019728792003956564819968")
                .unwrap()
        );
        assert_eq!(d.col_binary, b"test".to_vec());
    }
}
