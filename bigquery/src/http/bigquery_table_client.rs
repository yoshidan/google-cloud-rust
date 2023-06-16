use std::sync::Arc;

use crate::http::bigquery_client::BigqueryClient;
use crate::http::error::Error;
use crate::http::table;
use crate::http::table::get_iam_policy::GetIamPolicyRequest;
use crate::http::table::list::{ListTablesRequest, ListTablesResponse, TableOverview};
use crate::http::table::set_iam_policy::SetIamPolicyRequest;
use crate::http::table::test_iam_permissions::{TestIamPermissionsRequest, TestIamPermissionsResponse};
use crate::http::table::Table;
use crate::http::types::Policy;

#[derive(Debug, Clone)]
pub struct BigqueryTableClient {
    inner: Arc<BigqueryClient>,
}

impl BigqueryTableClient {
    pub fn new(inner: Arc<BigqueryClient>) -> Self {
        Self { inner }
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn create(&self, metadata: &Table) -> Result<Table, Error> {
        let builder = table::insert::build(self.inner.endpoint(), self.inner.http(), metadata);
        self.inner.send(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn delete(&self, project_id: &str, dataset_id: &str, table_id: &str) -> Result<(), Error> {
        let builder = table::delete::build(self.inner.endpoint(), self.inner.http(), project_id, dataset_id, table_id);
        self.inner.send_get_empty(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn patch(&self, metadata: &Table) -> Result<Table, Error> {
        let builder = table::patch::build(self.inner.endpoint(), self.inner.http(), metadata);
        self.inner.send(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get(&self, project_id: &str, dataset_id: &str, table_id: &str) -> Result<Table, Error> {
        let builder = table::get::build(self.inner.endpoint(), self.inner.http(), project_id, dataset_id, table_id);
        self.inner.send(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_iam_policy(
        &self,
        project_id: &str,
        dataset_id: &str,
        table_id: &str,
        req: &GetIamPolicyRequest,
    ) -> Result<Policy, Error> {
        let builder = table::get_iam_policy::build(
            self.inner.endpoint(),
            self.inner.http(),
            project_id,
            dataset_id,
            table_id,
            req,
        );
        self.inner.send(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn set_iam_policy(
        &self,
        project_id: &str,
        dataset_id: &str,
        table_id: &str,
        req: &SetIamPolicyRequest,
    ) -> Result<Policy, Error> {
        let builder = table::set_iam_policy::build(
            self.inner.endpoint(),
            self.inner.http(),
            project_id,
            dataset_id,
            table_id,
            req,
        );
        self.inner.send(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn test_iam_permissions(
        &self,
        project_id: &str,
        dataset_id: &str,
        table_id: &str,
        req: &TestIamPermissionsRequest,
    ) -> Result<TestIamPermissionsResponse, Error> {
        let builder = table::test_iam_permissions::build(
            self.inner.endpoint(),
            self.inner.http(),
            project_id,
            dataset_id,
            table_id,
            req,
        );
        self.inner.send(builder).await
    }

    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn list(
        &self,
        project_id: &str,
        dataset_id: &str,
        req: &ListTablesRequest,
    ) -> Result<Vec<TableOverview>, Error> {
        let mut page_token: Option<String> = None;
        let mut tables = vec![];
        loop {
            let builder = table::list::build(
                self.inner.endpoint(),
                self.inner.http(),
                project_id,
                dataset_id,
                req,
                page_token,
            );
            let response: ListTablesResponse = self.inner.send(builder).await?;
            tables.extend(response.tables);
            if response.next_page_token.is_none() {
                break;
            }
            page_token = response.next_page_token;
        }
        Ok(tables)
    }
}

#[cfg(test)]
mod test {
    use std::ops::Add;
    use std::sync::Arc;

    use serial_test::serial;
    use time::OffsetDateTime;

    use crate::http::bigquery_client::test::create_client;
    use crate::http::bigquery_table_client::BigqueryTableClient;
    use crate::http::table::get_iam_policy::GetIamPolicyRequest;
    use crate::http::table::list::ListTablesRequest;
    use crate::http::table::set_iam_policy::SetIamPolicyRequest;
    use crate::http::table::{
        Clustering, CsvOptions, ExternalDataConfiguration, MaterializedViewDefinition, PartitionRange,
        RangePartitioning, RoundingMode, SourceFormat, Table, TableFieldMode, TableFieldSchema, TableFieldType,
        TableSchema, TimePartitionType, TimePartitioning, ViewDefinition,
    };
    use crate::http::types::{Bindings, Policy};

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt::try_init();
    }

    #[tokio::test]
    #[serial]
    pub async fn crud_table() {
        let (client, project) = create_client().await;
        let client = BigqueryTableClient::new(Arc::new(client));

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
        let table1 = client.create(&table1).await.unwrap();

        // iam
        let ref1 = &table1.table_reference;
        let policy = client
            .set_iam_policy(
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
            .get_iam_policy(
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
        let _view = client.create(&view).await.unwrap();

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
        table2.expiration_time = Some(OffsetDateTime::now_utc().add(time::Duration::days(1)).unix_timestamp() * 1000);
        let _table2 = client.create(&table2).await.unwrap();

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
        let _table3 = client.create(&table3).await.unwrap();

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
        let _mv = client.create(&mv).await.unwrap();

        // delete
        let tables = client
            .list(
                project.as_str(),
                &table1.table_reference.dataset_id,
                &ListTablesRequest::default(),
            )
            .await
            .unwrap();
        for table in tables {
            let table = table.table_reference;
            client
                .delete(table.project_id.as_str(), table.dataset_id.as_str(), table.table_id.as_str())
                .await
                .unwrap();
        }
    }

    #[tokio::test]
    #[serial]
    pub async fn external_table() {
        let (client, project) = create_client().await;
        let client = BigqueryTableClient::new(Arc::new(client));

        // CSV
        let mut table = Table::default();
        table.table_reference.dataset_id = "rust_test_external_table".to_string();
        table.table_reference.project_id = project.to_string();
        table.table_reference.table_id = format!("csv_table_{}", OffsetDateTime::now_utc().unix_timestamp());
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

        let create_result = client.create(&table).await.unwrap();
        let patch_result = client.patch(&create_result).await.unwrap();
        let tref = &patch_result.table_reference;
        let get_result = client
            .get(tref.project_id.as_str(), tref.dataset_id.as_str(), tref.table_id.as_str())
            .await
            .unwrap();
        assert_eq!(get_result, patch_result);

        // cleanup
        client
            .delete(tref.project_id.as_str(), tref.dataset_id.as_str(), tref.table_id.as_str())
            .await
            .unwrap();
    }
}
