use crate::http::dataset::list::{DatasetOverview, ListDatasetsRequest, ListDatasetsResponse};
use crate::http::dataset::Dataset;
use crate::http::error::{Error, ErrorWrapper};
use crate::http::table::get_iam_policy::GetIamPolicyRequest;
use crate::http::table::set_iam_policy::SetIamPolicyRequest;
use crate::http::table::test_iam_permissions::{TestIamPermissionsRequest, TestIamPermissionsResponse};
use crate::http::table::{Table, TableReference};
use crate::http::tabledata::insert_all::{InsertAllRequest, InsertAllResponse};
use crate::http::types::Policy;
use crate::http::{dataset, table, tabledata};
use google_cloud_token::TokenSource;
use reqwest::{Client, RequestBuilder, Response};
use serde::Serialize;
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
    pub async fn create_dataset(&self, metadata: &Dataset) -> Result<Dataset, Error> {
        let builder = dataset::insert::build(self.endpoint.as_str(), &self.http, metadata);
        self.send(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn patch_dataset(&self, metadata: &Dataset) -> Result<Dataset, Error> {
        let builder = dataset::patch::build(self.endpoint.as_str(), &self.http, metadata);
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

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn create_table(&self, metadata: &Table) -> Result<Table, Error> {
        let builder = table::insert::build(self.endpoint.as_str(), &self.http, metadata);
        self.send(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn delete_table(&self, project_id: &str, dataset_id: &str, table_id: &str) -> Result<(), Error> {
        let builder = table::delete::build(self.endpoint.as_str(), &self.http, project_id, dataset_id, table_id);
        self.send_get_empty(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn patch_table(&self, metadata: &Table) -> Result<Table, Error> {
        let builder = table::patch::build(self.endpoint.as_str(), &self.http, metadata);
        self.send(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_table(&self, project_id: &str, dataset_id: &str, table_id: &str) -> Result<Table, Error> {
        let builder = table::get::build(self.endpoint.as_str(), &self.http, project_id, dataset_id, table_id);
        self.send(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_table_iam_policy(
        &self,
        project_id: &str,
        dataset_id: &str,
        table_id: &str,
        req: &GetIamPolicyRequest,
    ) -> Result<Policy, Error> {
        let builder =
            table::get_iam_policy::build(self.endpoint.as_str(), &self.http, project_id, dataset_id, table_id, req);
        self.send(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn set_table_iam_policy(
        &self,
        project_id: &str,
        dataset_id: &str,
        table_id: &str,
        req: &SetIamPolicyRequest,
    ) -> Result<Policy, Error> {
        let builder =
            table::set_iam_policy::build(self.endpoint.as_str(), &self.http, project_id, dataset_id, table_id, req);
        self.send(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn test_table_iam_permissions(
        &self,
        project_id: &str,
        dataset_id: &str,
        table_id: &str,
        req: &TestIamPermissionsRequest,
    ) -> Result<TestIamPermissionsResponse, Error> {
        let builder = table::test_iam_permissions::build(
            self.endpoint.as_str(),
            &self.http,
            project_id,
            dataset_id,
            table_id,
            req,
        );
        self.send(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn insert_into_table<T: Serialize>(
        &self,
        project_id: &str,
        dataset_id: &str,
        table_id: &str,
        req: &InsertAllRequest<T>,
    ) -> Result<InsertAllResponse, Error> {
        let builder =
            tabledata::insert_all::build(self.endpoint.as_str(), &self.http, project_id, dataset_id, table_id, req);
        self.send(builder).await
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
    use crate::http::table::get_iam_policy::GetIamPolicyRequest;
    use crate::http::table::set_iam_policy::SetIamPolicyRequest;
    use crate::http::table::test_iam_permissions::TestIamPermissionsRequest;
    use crate::http::table::{
        Clustering, CsvOptions, ExternalDataConfiguration, MaterializedViewDefinition, PartitionRange,
        RangePartitioning, RoundingMode, SourceFormat, Table, TableFieldMode, TableFieldSchema, TableFieldType,
        TableSchema, TimePartitionType, TimePartitioning, ViewDefinition,
    };
    use crate::http::tabledata::insert_all::{InsertAllRequest, Row};
    use crate::http::types::{Bindings, Collation, EncryptionConfiguration, Policy};
    use google_cloud_auth::project::Config;
    use google_cloud_auth::token::DefaultTokenSourceProvider;
    use google_cloud_token::TokenSourceProvider;
    use serde_json::{json, Value};
    use serial_test::serial;
    use std::collections::HashMap;
    use time::OffsetDateTime;

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
        ds1.dataset_reference.project_id = project.clone();
        ds1 = client.create_dataset(&ds1).await.unwrap();

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
        let ds2 = client.create_dataset(&ds2).await.unwrap();

        // test get
        let mut res1 = client
            .get_dataset(project.as_str(), &ds1.dataset_reference.dataset_id)
            .await
            .unwrap();
        let res2 = client
            .get_dataset(project.as_str(), &ds2.dataset_reference.dataset_id)
            .await
            .unwrap();
        assert_eq!(ds1, res1);
        assert_eq!(ds2, res2);

        // test update
        res1.description = Some("rust_test_empty_updated".to_string());
        client.patch_dataset(&res1).await.unwrap();

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

    #[tokio::test]
    #[serial]
    pub async fn crud_table() {
        let (client, project) = client().await;

        // empty
        let mut table1 = Table::default();
        table1.table_reference.dataset_id = "rust_test_table".to_string();
        table1.table_reference.project_id = project.to_string();
        table1.table_reference.table_id = "table1".to_string();
        table1.schema = Some(TableSchema {
            fields: vec![
                TableFieldSchema {
                    name: "col1".to_string(),
                    data_type: TableFieldType::String,
                    description: Some("column1".to_string()),
                    max_length: Some(32),
                    ..Default::default()
                },
                TableFieldSchema {
                    name: "col2".to_string(),
                    data_type: TableFieldType::Numeric,
                    description: Some("column2".to_string()),
                    precision: Some(10),
                    rounding_mode: Some(RoundingMode::RoundHalfEven),
                    scale: Some(2),
                    ..Default::default()
                },
                TableFieldSchema {
                    name: "col3".to_string(),
                    data_type: TableFieldType::Timestamp,
                    mode: Some(TableFieldMode::Required),
                    default_value_expression: Some("CURRENT_TIMESTAMP".to_string()),
                    ..Default::default()
                },
                TableFieldSchema {
                    name: "col4".to_string(),
                    data_type: TableFieldType::Int64,
                    mode: Some(TableFieldMode::Repeated),
                    ..Default::default()
                },
                TableFieldSchema {
                    name: "col5".to_string(),
                    data_type: TableFieldType::Int64,
                    ..Default::default()
                },
            ],
        });
        let table1 = client.create_table(&table1).await.unwrap();

        // iam
        let ref1 = &table1.table_reference;
        let policy = client
            .set_table_iam_policy(
                &ref1.project_id,
                &ref1.dataset_id,
                &ref1.table_id,
                &SetIamPolicyRequest {
                    policy: Policy {
                        bindings: vec![Bindings {
                            role: "roles/viewer".to_string(),
                            members: vec!["allAuthenticatedUsers".to_string()],
                            ..Default::default()
                        }],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let actual_policy = client
            .get_table_iam_policy(
                &ref1.project_id,
                &ref1.dataset_id,
                &ref1.table_id,
                &GetIamPolicyRequest::default(),
            )
            .await
            .unwrap();
        assert_eq!(policy, actual_policy);

        let mut view = Table::default();
        view.table_reference.dataset_id = table1.table_reference.dataset_id.to_string();
        view.table_reference.project_id = table1.table_reference.project_id.to_string();
        view.table_reference.table_id = "view1".to_string();
        view.view = Some(ViewDefinition {
            query: "SELECT col1 FROM rust_test_table.table1".to_string(),
            ..Default::default()
        });
        let view = client.create_table(&view).await.unwrap();

        // range partition
        let mut table2 = table1.clone();
        table2.table_reference.table_id = "range_partition".to_string();
        table2.range_partitioning = Some(RangePartitioning {
            field: "col5".to_string(),
            range: PartitionRange {
                start: "1".to_string(),
                end: "10000".to_string(),
                interval: "1".to_string(),
            },
        });
        table2.expiration_time = Some(3600);
        let table2 = client.create_table(&table2).await.unwrap();

        // time partition
        let mut table3 = table1.clone();
        table3.table_reference.table_id = "time_partition".to_string();
        table3.time_partitioning = Some(TimePartitioning {
            partition_type: TimePartitionType::Day,
            expiration_ms: Some(3600000),
            field: Some("col3".to_string()),
        });
        table3.clustering = Some(Clustering {
            fields: vec!["col1".to_string(), "col5".to_string()],
        });
        let table3 = client.create_table(&table3).await.unwrap();

        // materialized view
        let mut mv = Table::default();
        mv.table_reference.dataset_id = table1.table_reference.dataset_id.to_string();
        mv.table_reference.project_id = table1.table_reference.project_id.to_string();
        mv.table_reference.table_id = "materialized_view1".to_string();
        mv.materialized_view = Some(MaterializedViewDefinition {
            query: "SELECT col2 FROM rust_test_table.table1".to_string(),
            refresh_interval_ms: Some(3600000),
            ..Default::default()
        });
        let mv = client.create_table(&mv).await.unwrap();

        // delete
        let tables = vec![table1, table2, table3, view, mv];
        for table in tables {
            let table = table.table_reference;
            client
                .delete_table(table.project_id.as_str(), table.dataset_id.as_str(), table.table_id.as_str())
                .await
                .unwrap();
        }
    }

    #[tokio::test]
    #[serial]
    pub async fn external_table() {
        let (client, project) = client().await;

        // CSV
        let mut table = Table::default();
        table.table_reference.dataset_id = "rust_test_external_table".to_string();
        table.table_reference.project_id = project.to_string();
        table.table_reference.table_id = "csv_table".to_string();
        table.external_data_configuration = Some(ExternalDataConfiguration {
            source_uris: vec!["gs://rust-bq-test/external_data.csv".to_string()],
            autodetect: true,
            source_format: SourceFormat::Csv,
            csv_options: Some(CsvOptions {
                field_delimiter: Some("|".to_string()),
                encoding: Some("UTF-8".to_string()),
                skip_leading_rows: Some(0),
                ..Default::default()
            }),
            ..Default::default()
        });

        let create_result = client.create_table(&table).await.unwrap();
        let patch_result = client.patch_table(&create_result).await.unwrap();
        let tref = &patch_result.table_reference;
        let get_result = client
            .get_table(tref.project_id.as_str(), tref.dataset_id.as_str(), tref.table_id.as_str())
            .await
            .unwrap();
        assert_eq!(get_result, patch_result);

        // cleanup
        client
            .delete_table(tref.project_id.as_str(), tref.dataset_id.as_str(), tref.table_id.as_str())
            .await
            .unwrap();
    }

    #[derive(serde::Serialize)]
    struct TestData {
        pub col1: Option<String>,
        pub col2: Vec<i32>,
        #[serde(with = "time::serde::rfc3339")]
        pub col3: OffsetDateTime,
    }

    #[tokio::test]
    #[serial]
    pub async fn table_data() {
        let (client, project) = client().await;
        let mut table1 = Table::default();
        table1.table_reference.dataset_id = "rust_test_table".to_string();
        table1.table_reference.project_id = project.to_string();
        table1.table_reference.table_id = "table_data5".to_string();
        table1.schema = Some(TableSchema {
            fields: vec![
                TableFieldSchema {
                    name: "col1".to_string(),
                    data_type: TableFieldType::String,
                    description: Some("column1".to_string()),
                    max_length: Some(32),
                    ..Default::default()
                },
                TableFieldSchema {
                    name: "col2".to_string(),
                    data_type: TableFieldType::Numeric,
                    mode: Some(TableFieldMode::Repeated),
                    description: Some("column2".to_string()),
                    ..Default::default()
                },
                TableFieldSchema {
                    name: "col3".to_string(),
                    data_type: TableFieldType::Timestamp,
                    description: Some("column3".to_string()),
                    ..Default::default()
                },
            ],
        });
        let table1 = client.create_table(&table1).await.unwrap();
        let ref1 = table1.table_reference;

        // json value
        let mut req = InsertAllRequest::<Value>::default();
        req.rows.push(Row {
            insert_id: None,
            json: serde_json::from_str(
                r#"
                {"col1": "test1", "col2": [1,2,3], "col3":"2022-10-23T00:00:00"}
            "#,
            )
            .unwrap(),
        });
        req.rows.push(Row {
            insert_id: None,
            json: serde_json::from_str(
                r#"
                {"col2": [4,5,6], "col3":"2022-10-23T00:00:00"}
            "#,
            )
            .unwrap(),
        });
        let res = client
            .insert_into_table(ref1.project_id.as_str(), ref1.dataset_id.as_str(), ref1.table_id.as_str(), &req)
            .await
            .unwrap();

        // struct
        let mut req2 = InsertAllRequest::<TestData>::default();
        req2.rows.push(Row {
            insert_id: None,
            json: TestData {
                col1: Some("test3".to_string()),
                col2: vec![10, 11, 12],
                col3: OffsetDateTime::now_utc(),
            },
        });
        req2.rows.push(Row {
            insert_id: None,
            json: TestData {
                col1: None,
                col2: vec![100, 1100, 120],
                col3: OffsetDateTime::now_utc(),
            },
        });
        let res2 = client
            .insert_into_table(
                ref1.project_id.as_str(),
                ref1.dataset_id.as_str(),
                ref1.table_id.as_str(),
                &req2,
            )
            .await
            .unwrap();

        client
            .delete_table(ref1.project_id.as_str(), ref1.dataset_id.as_str(), ref1.table_id.as_str())
            .await
            .unwrap();

        assert!(res.insert_errors.is_none());
        assert!(res2.insert_errors.is_none());
    }
}
