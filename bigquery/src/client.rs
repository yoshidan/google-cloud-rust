use std::collections::VecDeque;
use crate::http::bigquery_client::BigqueryClient;
use crate::http::bigquery_dataset_client::BigqueryDatasetClient;
use crate::http::bigquery_job_client::BigqueryJobClient;
use crate::http::bigquery_table_client::BigqueryTableClient;
use crate::http::bigquery_tabledata_client::BigqueryTabledataClient;
use google_cloud_token::{NopeTokenSourceProvider, TokenSourceProvider};
use std::ops::Deref;
use std::sync::Arc;
use crate::http::error::Error;
use crate::http::job::get_query_results::GetQueryResultsRequest;
use crate::http::job::query::{QueryRequest, QueryResponse};
use crate::iterator::RowIterator;

#[derive(Debug)]
pub struct ClientConfig {
    pub http: reqwest::Client,
    pub bigquery_endpoint: String,
    pub token_source_provider: Box<dyn TokenSourceProvider>,
    pub project_id: Option<String>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            http: reqwest::Client::default(),
            bigquery_endpoint: "https://bigquery.googleapis.com".to_string(),
            token_source_provider: Box::new(NopeTokenSourceProvider {}),
            project_id: None,
        }
    }
}

pub struct Client {
    dataset_client: BigqueryDatasetClient,
    table_client: BigqueryTableClient,
    tabledata_client: BigqueryTabledataClient,
    job_client: BigqueryJobClient,
    project_id: String
}

impl Default for Client {
    fn default() -> Self {
        Self::new(ClientConfig::default())
    }
}

impl Client {
    /// New client
    pub fn new(config: ClientConfig) -> Self {
        let ts = config.token_source_provider.token_source();
        let client = Arc::new(BigqueryClient::new(ts, config.bigquery_endpoint.as_str(), config.http));
        Self {
            dataset_client: BigqueryDatasetClient::new(client.clone()),
            table_client: BigqueryTableClient::new(client.clone()),
            tabledata_client: BigqueryTabledataClient::new(client.clone()),
            job_client: BigqueryJobClient::new(client.clone()),
            project_id: config.project_id.unwrap_or_default()
        }
    }

    pub fn dataset(&self) -> &BigqueryDatasetClient{
        return &self.dataset_client
    }

    pub fn table(&self) -> &BigqueryTableClient {
        return &self.table_client
    }

    pub fn tabledata(&self) -> &BigqueryTabledataClient {
        return &self.tabledata_client
    }

    pub fn job(&self) -> &BigqueryJobClient {
        return &self.job_client
    }

    pub async fn query(&self, request: QueryRequest) -> Result<RowIterator,Error> {
        let result = self.job_client.query(self.project_id.as_str(), &request).await?;
        Ok(RowIterator {
            client: self.job_client.clone(),
            project_id: result.job_reference.project_id,
            job_id: result.job_reference.job_id,
            request: GetQueryResultsRequest {
                start_index: 0,
                page_token: result.page_token,
                max_results: request.max_results,
                timeout_ms: request.timeout_ms,
                location: Some(request.location),
                format_options: request.format_options
            },
            chunk: VecDeque::from(result.rows.unwrap_or_default()),
            total_size: result.total_rows.unwrap_or_default()
        })
    }
}

#[cfg(test)]
mod tests {
    use google_cloud_auth::project::Config;
    use google_cloud_auth::token::DefaultTokenSourceProvider;
    use google_cloud_token::TokenSourceProvider;
    use crate::client::{Client, ClientConfig};
    use crate::http::bigquery_client::SCOPES;

    use serial_test::serial;
    use crate::http::job::query::QueryRequest;

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt::try_init();
    }

    async fn create_client() -> Client {
        let mut client_config = ClientConfig::default();
        let tsp = DefaultTokenSourceProvider::new(Config {
            audience: None,
            scopes: Some(&SCOPES),
        }).await.unwrap();
        client_config.token_source_provider = Box::new(tsp);
        client_config.project_id = tsp.source_credentials.unwrap().project_id;
        Client::new(client_config)
    }

    #[tokio::test]
    #[serial]
    async fn test_query() {
        let client = create_client().await;
        let mut iterator= client.query(QueryRequest {
            max_results: Some(2),
            query: "SELECT * from dual".to_string(),
            ..Default::default()
        }).await.unwrap();

        while let Some(row)  = iterator.next().await.unwrap() {

        }
    }
}