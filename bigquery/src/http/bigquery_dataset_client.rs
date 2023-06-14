use std::sync::Arc;

use crate::http::bigquery_client::BigqueryClient;
use crate::http::dataset;
use crate::http::dataset::list::{DatasetOverview, ListDatasetsRequest, ListDatasetsResponse};
use crate::http::dataset::Dataset;
use crate::http::error::Error;

#[derive(Clone)]
pub struct BigqueryDatasetClient {
    inner: Arc<BigqueryClient>,
}

impl BigqueryDatasetClient {
    pub fn new(inner: Arc<BigqueryClient>) -> Self {
        Self { inner }
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn create(&self, metadata: &Dataset) -> Result<Dataset, Error> {
        let builder = dataset::insert::build(self.inner.endpoint(), self.inner.http(), metadata);
        self.inner.send(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn patch(&self, metadata: &Dataset) -> Result<Dataset, Error> {
        let builder = dataset::patch::build(self.inner.endpoint(), self.inner.http(), metadata);
        self.inner.send(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn delete(&self, project_id: &str, dataset_id: &str) -> Result<(), Error> {
        let builder = dataset::delete::build(self.inner.endpoint(), self.inner.http(), project_id, dataset_id);
        self.inner.send_get_empty(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get(&self, project_id: &str, dataset_id: &str) -> Result<Dataset, Error> {
        let builder = dataset::get::build(self.inner.endpoint(), self.inner.http(), project_id, dataset_id);
        self.inner.send(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn list(
        &self,
        project_id: &str,
        req: Option<&ListDatasetsRequest>,
    ) -> Result<Vec<DatasetOverview>, Error> {
        let mut page_token: Option<String> = None;
        let mut datasets = vec![];
        loop {
            let builder = dataset::list::build(self.inner.endpoint(), self.inner.http(), project_id, req, page_token);
            let response: ListDatasetsResponse = self.inner.send(builder).await?;
            datasets.extend(response.datasets);
            if response.next_page_token.is_none() {
                break;
            }
            page_token = response.next_page_token;
        }
        Ok(datasets)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::sync::Arc;

    use serial_test::serial;

    use crate::http::bigquery_client::test::create_client;
    use crate::http::bigquery_dataset_client::BigqueryDatasetClient;
    use crate::http::dataset::list::ListDatasetsRequest;
    use crate::http::dataset::{Access, Dataset, DatasetReference, SpecialGroup, StorageBillingModel};
    use crate::http::types::{Collation, EncryptionConfiguration};

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt::try_init();
    }

    #[tokio::test]
    #[serial]
    pub async fn crud_dataset() {
        let (client, project) = create_client().await;
        let client = BigqueryDatasetClient::new(Arc::new(client));

        // minimum dataset
        let mut ds1 = Dataset::default();
        ds1.dataset_reference.dataset_id = "rust_test_empty".to_string();
        ds1.dataset_reference.project_id = project.clone();
        ds1 = client.create(&ds1).await.unwrap();

        // full prop dataset
        let mut labels = HashMap::new();
        labels.insert("key".to_string(), "value".to_string());
        let ds2 = Dataset {
            dataset_reference: DatasetReference {
                dataset_id: "rust_test_full".to_string(),
                project_id: project.to_string(),
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
            default_collation: Some(Collation::UndeterminedLocaleCaseInsensitive),
            max_time_travel_hours: Some(48),
            storage_billing_model: Some(StorageBillingModel::Logical),
            ..Default::default()
        };
        let ds2 = client.create(&ds2).await.unwrap();

        // test get
        let mut res1 = client
            .get(project.as_str(), &ds1.dataset_reference.dataset_id)
            .await
            .unwrap();
        let res2 = client
            .get(project.as_str(), &ds2.dataset_reference.dataset_id)
            .await
            .unwrap();
        assert_eq!(ds1, res1);
        assert_eq!(ds2, res2);

        // test update
        res1.description = Some("rust_test_empty_updated".to_string());
        client.patch(&res1).await.unwrap();

        // test list
        let result = client.list(project.as_str(), None).await.unwrap();
        assert!(result.len() >= 2);
        let result = client
            .list(
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
            .list(
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
            .delete(project.as_str(), ds1.dataset_reference.dataset_id.as_str())
            .await
            .unwrap();
        client
            .delete(project.as_str(), ds2.dataset_reference.dataset_id.as_str())
            .await
            .unwrap();
    }
}
