use crate::http::dataset;
use crate::http::dataset::list::{DatasetOverview, ListDatasetsRequest, ListDatasetsResponse};
use crate::http::dataset::Dataset;
use crate::http::error::{Error, ErrorWrapper};
use google_cloud_token::TokenSource;
use reqwest::{Client, RequestBuilder, Response};
use std::sync::Arc;

pub const SCOPES: [&str; 7] = [
    "https://www.googleapis.com/auth/bigquery",
    "https://www.googleapis.com/auth/bigquery.insertdata",
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/cloud-platform.read-only",
    "https://www.googleapis.com/auth/devstorage.full_control",
    "https://www.googleapis.com/auth/devstorage.read_only",
    "https://www.googleapis.com/auth/devstorage.read_write",
];

#[derive(Clone)]
pub struct BigqueryClient {
    ts: Arc<dyn TokenSource>,
    endpoint: String,
    http: Client,
}

impl BigqueryClient {
    pub(crate) fn new(ts: Arc<dyn TokenSource>, endpoint: &str, http: Client) -> Self {
        Self {
            ts,
            endpoint: format!("{endpoint}/bigquery/v2"),
            http,
        }
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn insert_dataset(&self, project_id: &str, metadata: &Dataset) -> Result<Dataset, Error> {
        let builder = dataset::insert::build(self.endpoint.as_str(), &self.http, project_id, metadata);
        self.send(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn delete_dataset(&self, project_id: &str, dataset_id: &str) -> Result<(), Error> {
        let builder = dataset::delete::build(self.endpoint.as_str(), &self.http, project_id, dataset_id);
        self.send_get_empty(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_dataset(&self, project_id: &str, dataset_id: &str) -> Result<Dataset, Error> {
        let builder = dataset::get::build(self.endpoint.as_str(), &self.http, project_id, dataset_id);
        self.send(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn list_datasets(
        &self,
        project_id: &str,
        req: Option<&ListDatasetsRequest>,
    ) -> Result<Vec<DatasetOverview>, Error> {
        let mut page_token: Option<String> = None;
        let mut datasets = vec![];
        loop {
            let builder = dataset::list::build(self.endpoint.as_str(), &self.http, project_id, req, page_token);
            let response: ListDatasetsResponse = self.send(builder).await?;
            datasets.extend(response.datasets);
            if response.next_page_token.is_none() {
                break;
            }
            page_token = response.next_page_token;
        }
        Ok(datasets)
    }

    async fn with_headers(&self, builder: RequestBuilder) -> Result<RequestBuilder, Error> {
        let token = self.ts.token().await.map_err(Error::TokenSource)?;
        Ok(builder
            .header("X-Goog-Api-Client", "rust")
            .header(reqwest::header::USER_AGENT, "google-cloud-bigquery")
            .header(reqwest::header::AUTHORIZATION, token))
    }

    async fn send<T>(&self, builder: RequestBuilder) -> Result<T, Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let request = self.with_headers(builder).await?;
        let response = request.send().await?;
        let response = Self::check_response_status(response).await?;
        //TODO json
        let text = response.text().await?;
        tracing::info!("{}", text);
        Ok(serde_json::from_str(text.as_str()).unwrap())
    }

    async fn send_get_empty(&self, builder: RequestBuilder) -> Result<(), Error> {
        let builder = self.with_headers(builder).await?;
        let response = builder.send().await?;
        Self::check_response_status(response).await?;
        Ok(())
    }

    /// Checks whether an HTTP response is successful and returns it, or returns an error.
    async fn check_response_status(response: Response) -> Result<Response, Error> {
        // Check the status code, returning the response if it is not an error.
        let error = match response.error_for_status_ref() {
            Ok(_) => return Ok(response),
            Err(error) => error,
        };

        // try to extract a response error, falling back to the status error if it can not be parsed.
        Err(response
            .json::<ErrorWrapper>()
            .await
            .map(|wrapper| Error::Response(wrapper.error))
            .unwrap_or(Error::HttpClient(error)))
    }
}

#[cfg(test)]
mod test {
    use crate::http::bigquery_client::{BigqueryClient, SCOPES};
    use crate::http::dataset::list::ListDatasetsRequest;
    use crate::http::dataset::{Access, Dataset, DatasetReference, SpecialGroup, StorageBillingModel};
    use crate::http::types::EncryptionConfiguration;
    use google_cloud_auth::project::Config;
    use google_cloud_auth::token::DefaultTokenSourceProvider;
    use google_cloud_token::TokenSourceProvider;
    use serial_test::serial;
    use std::collections::HashMap;

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt::try_init();
    }

    async fn client() -> (BigqueryClient, String) {
        let tsp = DefaultTokenSourceProvider::new(Config {
            audience: None,
            scopes: Some(&SCOPES),
        })
        .await
        .unwrap();
        let cred = tsp.source_credentials.clone();
        let ts = tsp.token_source();
        let client = BigqueryClient::new(ts, "https://bigquery.googleapis.com", reqwest::Client::new());
        (client, cred.unwrap().project_id.unwrap())
    }

    #[tokio::test]
    #[serial]
    pub async fn crud_dataset() {
        let (client, project) = client().await;

        // minimum dataset
        let mut ds1 = Dataset::default();
        ds1.dataset_reference.dataset_id = "rust_test_empty".to_string();
        ds1 = client.insert_dataset(project.as_str(), &ds1).await.unwrap();

        // full prop dataset
        let mut labels = HashMap::new();
        labels.insert("key".to_string(), "value".to_string());
        let ds2 = Dataset {
            dataset_reference: DatasetReference {
                dataset_id: "rust_test_full".to_string(),
                project_id: Some(project.to_string()),
            },
            friendly_name: Some("gcr_test_friendly_name".to_string()),
            description: Some("gcr_test_description".to_string()),
            default_table_expiration_ms: Some(3600000),
            default_partition_expiration_ms: Some(3600000),
            labels: Some(labels),
            access: vec![Access {
                role: "READER".to_string(),
                special_group: Some(SpecialGroup::AllAuthenticatedUsers),
                ..Default::default()
            }],
            location: "asia-northeast1".to_string(),
            default_encryption_configuration: Some(EncryptionConfiguration {
                kms_key_name: Some(format!(
                    "projects/{}/locations/asia-northeast1/keyRings/gcr_test/cryptoKeys/gcr_test",
                    project.as_str()
                )),
            }),
            is_case_insensitive: Some(true),
            default_collation: Some("und:ci".to_string()),
            max_time_travel_hours: Some(48),
            storage_billing_model: Some(StorageBillingModel::Logical),
            ..Default::default()
        };
        let ds2 = client.insert_dataset(project.as_str(), &ds2).await.unwrap();

        // test get
        let res1 = client
            .get_dataset(project.as_str(), &ds1.dataset_reference.dataset_id)
            .await
            .unwrap();
        let res2 = client
            .get_dataset(project.as_str(), &ds2.dataset_reference.dataset_id)
            .await
            .unwrap();
        assert_eq!(ds1, res1);
        assert_eq!(ds2, res2);

        // test list
        let result = client.list_datasets(project.as_str(), None).await.unwrap();
        assert!(result.len() >= 2);
        let result = client
            .list_datasets(
                project.as_str(),
                Some(&ListDatasetsRequest {
                    max_results: Some(100),
                    all: true,
                    filter: "".to_string(),
                }),
            )
            .await
            .unwrap();
        assert!(result.len() >= 2);

        let result = client
            .list_datasets(
                project.as_str(),
                Some(&ListDatasetsRequest {
                    max_results: None,
                    all: true,
                    filter: "labels.key:value".to_string(),
                }),
            )
            .await
            .unwrap();
        assert_eq!(1, result.len());
        assert_eq!(res2.id, result[0].id);

        // test delete
        client
            .delete_dataset(project.as_str(), ds1.dataset_reference.dataset_id.as_str())
            .await
            .unwrap();
        client
            .delete_dataset(project.as_str(), ds2.dataset_reference.dataset_id.as_str())
            .await
            .unwrap();
    }
}
