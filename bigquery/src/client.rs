use crate::http::bigquery_client::BigqueryClient;
use crate::http::bigquery_dataset_client::BigqueryDatasetClient;
use crate::http::bigquery_job_client::BigqueryJobClient;
use crate::http::bigquery_table_client::BigqueryTableClient;
use crate::http::bigquery_tabledata_client::BigqueryTabledataClient;
use google_cloud_token::{NopeTokenSourceProvider, TokenSourceProvider};
use std::ops::Deref;
use std::sync::Arc;

#[derive(Debug)]
pub struct ClientConfig {
    pub http: Option<reqwest::Client>,
    pub bigquery_endpoint: String,
    pub token_source_provider: Box<dyn TokenSourceProvider>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            http: None,
            bigquery_endpoint: "https://bigquery.googleapis.com".to_string(),
            token_source_provider: Box::new(NopeTokenSourceProvider {}),
        }
    }
}

pub struct Client {
    dataset_client: BigqueryDatasetClient,
    table_client: BigqueryTableClient,
    tabledata_client: BigqueryTabledataClient,
    job_client: BigqueryJobClient,
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
        let http = config.http.unwrap_or_default();
        let client = Arc::new(BigqueryClient::new(ts, config.bigquery_endpoint.as_str(), http));
        Self {
            dataset_client: BigqueryDatasetClient::new(client.clone()),
            table_client: BigqueryTableClient::new(client.clone()),
            tabledata_client: BigqueryTabledataClient::new(client.clone()),
            job_client: BigqueryJobClient::new(client.clone()),
        }
    }
}
