use std::sync::Arc;

use serde::Serialize;

use crate::http::bigquery_client::BigqueryClient;
use crate::http::error::Error;
use crate::http::tabledata;
use crate::http::tabledata::insert_all::{InsertAllRequest, InsertAllResponse};
use crate::http::tabledata::list::{FetchDataRequest, FetchDataResponse};

#[derive(Debug, Clone)]
pub struct BigqueryTabledataClient {
    inner: Arc<BigqueryClient>,
}

impl BigqueryTabledataClient {
    pub fn new(inner: Arc<BigqueryClient>) -> Self {
        Self { inner }
    }

    /// https://cloud.google.com/bigquery/docs/reference/rest/v2/tabledata/insert
    /// ```rust
    /// use google_cloud_bigquery::http::tabledata::insert_all::{InsertAllRequest, Row};
    /// use google_cloud_bigquery::http::bigquery_tabledata_client::BigqueryTabledataClient;
    ///
    /// #[derive(serde::Serialize)]
    /// pub struct TestData {
    ///     pub col1: String,
    /// }
    ///
    /// async fn run(client: &BigqueryTabledataClient, project_id: &str, data: TestData) {
    ///     let data1 = Row {
    ///         insert_id: None,
    ///         json: data,
    ///     };
    ///     let request = InsertAllRequest {
    ///         rows: vec![data1],
    ///         ..Default::default()
    ///     };
    ///     let result = client.insert(project_id, "dataset", "table", &request).await.unwrap();
    ///     let error = result.insert_errors;
    /// }
    /// ```
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn insert<T: Serialize>(
        &self,
        project_id: &str,
        dataset_id: &str,
        table_id: &str,
        req: &InsertAllRequest<T>,
    ) -> Result<InsertAllResponse, Error> {
        let builder = tabledata::insert_all::build(
            self.inner.endpoint(),
            self.inner.http(),
            project_id,
            dataset_id,
            table_id,
            req,
        );
        self.inner.send(builder).await
    }

    /// https://cloud.google.com/bigquery/docs/reference/rest/v2/tabledata/list
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn read(
        &self,
        project_id: &str,
        dataset_id: &str,
        table_id: &str,
        req: &FetchDataRequest,
    ) -> Result<FetchDataResponse, Error> {
        let builder =
            tabledata::list::build(self.inner.endpoint(), self.inner.http(), project_id, dataset_id, table_id, req);
        self.inner.send(builder).await
    }
}

#[cfg(test)]
mod test {

    use std::sync::Arc;

    use serial_test::serial;
    use time::OffsetDateTime;

    use crate::http::bigquery_client::test::{create_client, create_table_schema, dataset_name, TestData};
    use crate::http::bigquery_table_client::BigqueryTableClient;
    use crate::http::bigquery_tabledata_client::BigqueryTabledataClient;
    use crate::http::table::Table;
    use crate::http::tabledata::insert_all::{InsertAllRequest, Row};
    use crate::http::tabledata::list;
    use crate::http::tabledata::list::FetchDataRequest;

    #[tokio::test]
    #[serial]
    pub async fn insert_all() {
        let dataset = dataset_name("table");
        let (client, project) = create_client().await;
        let client = Arc::new(client);
        let table_client = BigqueryTableClient::new(client.clone());
        let client = BigqueryTabledataClient::new(client.clone());
        let mut table1 = Table::default();
        table1.table_reference.dataset_id = dataset.to_string();
        table1.table_reference.project_id = project.to_string();
        table1.table_reference.table_id = format!("table_data_{}", OffsetDateTime::now_utc().unix_timestamp());
        table1.schema = Some(create_table_schema());
        let table1 = table_client.create(&table1).await.unwrap();
        let ref1 = table1.table_reference;

        // insert as json string
        let mut req = InsertAllRequest::<serde_json::Value>::default();
        req.rows.push(Row {
            insert_id: None,
            json: serde_json::from_str(
                r#"
                {"col_string": "test1", "col_number": 1, "col_number_array": [1,2,3], "col_timestamp":"2022-10-23T00:00:00", "col_json":"{\"field\":100}","col_json_array":["{\"field\":100}","{\"field\":200}"],"col_struct": {"f1":true, "f2":[3,4]},"col_struct_array": [{"f1":true, "f2":[3,4]},{"f1":false, "f2":[30,40]}], "col_binary": "dGVzdAo="}
            "#,
            )
                .unwrap(),
        });
        let res = client
            .insert(ref1.project_id.as_str(), ref1.dataset_id.as_str(), ref1.table_id.as_str(), &req)
            .await
            .unwrap();
        assert!(res.insert_errors.is_none());

        // isnert as struct
        let mut req2 = InsertAllRequest::<TestData>::default();
        req2.rows.push(Row {
            insert_id: None,
            json: TestData::default(1, OffsetDateTime::now_utc()),
        });
        let res2 = client
            .insert(
                ref1.project_id.as_str(),
                ref1.dataset_id.as_str(),
                ref1.table_id.as_str(),
                &req2,
            )
            .await
            .unwrap();
        assert!(res2.insert_errors.is_none());

        table_client
            .delete(ref1.project_id.as_str(), ref1.dataset_id.as_str(), ref1.table_id.as_str())
            .await
            .unwrap();
    }

    #[tokio::test]
    #[serial]
    pub async fn read_all() {
        let dataset = dataset_name("job");
        let (client, project) = create_client().await;
        let client = Arc::new(client);
        let client = BigqueryTabledataClient::new(client.clone());

        // fetch
        let mut fetch_request = FetchDataRequest {
            max_results: Some(500),
            ..Default::default()
        };
        let mut data: Vec<list::Tuple> = vec![];
        loop {
            let result = client
                .read(project.as_str(), dataset.as_str(), "reading_data", &fetch_request)
                .await
                .unwrap();
            if let Some(rows) = result.rows {
                data.extend(rows);
            }
            if result.page_token.is_none() {
                break;
            }
            fetch_request.page_token = result.page_token
        }
        assert_eq!(data.len(), 1000, "{:?}", data.pop());
    }
}
