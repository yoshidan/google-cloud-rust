use crate::http::bigquery_client::BigqueryClient;
use crate::http::error::Error;

use crate::http::tabledata::insert_all::{InsertAllRequest, InsertAllResponse};
use crate::http::tabledata::list::{FetchDataRequest, FetchDataResponse};

use crate::http::tabledata;
use serde::Serialize;
use std::sync::Arc;

#[derive(Clone)]
pub struct BigqueryTabledataClient {
    inner: Arc<BigqueryClient>,
}

impl BigqueryTabledataClient {
    pub fn new(inner: Arc<BigqueryClient>) -> Self {
        Self { inner }
    }

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
    use crate::http::bigquery_client::test::{create_client, create_table_schema};
    use crate::http::bigquery_table_client::BigqueryTableClient;
    use crate::http::bigquery_tabledata_client::BigqueryTabledataClient;

    use crate::http::table::Table;
    use crate::http::tabledata::insert_all::{InsertAllRequest, Row};
    use crate::http::tabledata::list;
    use crate::http::tabledata::list::{FetchDataRequest, Value};
    use serial_test::serial;
    use std::sync::Arc;
    use time::OffsetDateTime;

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt::try_init();
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    struct TestDataStruct {
        pub f1: bool,
        pub f2: Vec<i64>,
    }
    #[derive(serde::Serialize, serde::Deserialize)]
    struct TestData {
        pub col_string: Option<String>,
        pub col_number: Option<i32>,
        pub col_number_array: Vec<i32>,
        #[serde(default, with = "time::serde::rfc3339::option")]
        pub col_timestamp: Option<OffsetDateTime>,
        pub col_json: Option<String>,
        pub col_json_array: Vec<String>,
        pub col_struct: Option<TestDataStruct>,
        pub col_struct_array: Vec<TestDataStruct>,
    }

    #[tokio::test]
    #[serial]
    pub async fn table_data() {
        let (client, project) = create_client().await;
        let client = Arc::new(client);
        let table_client = BigqueryTableClient::new(client.clone());
        let client = BigqueryTabledataClient::new(client.clone());
        let mut table1 = Table::default();
        table1.table_reference.dataset_id = "rust_test_table".to_string();
        table1.table_reference.project_id = project.to_string();
        table1.table_reference.table_id = format!("table_data_{}", OffsetDateTime::now_utc().unix_timestamp());
        table1.schema = Some(create_table_schema());
        let table1 = table_client.create(&table1).await.unwrap();
        let ref1 = table1.table_reference;

        // json value
        let mut req = InsertAllRequest::<serde_json::Value>::default();
        req.rows.push(Row {
            insert_id: None,
            json: serde_json::from_str(
                r#"
                {"col_string": "test1", "col_number": 1, "col_number_array": [1,2,3], "col_timestamp":"2022-10-23T00:00:00", "col_json":"{\"field\":100}","col_json_array":["{\"field\":100}","{\"field\":200}"],"col_struct": {"f1":true, "f2":[3,4]},"col_struct_array": [{"f1":true, "f2":[3,4]},{"f1":false, "f2":[30,40]}]}
            "#,
            )
                .unwrap(),
        });
        req.rows.push(Row {
            insert_id: None,
            json: serde_json::from_str(
                r#"
                {"col_number_array": [1,2,3], "col_struct_array": [{"f1":true, "f2":[3,4]},{"f1":false, "f2":[30,40]}]}
            "#,
            )
            .unwrap(),
        });

        let res = client
            .insert(ref1.project_id.as_str(), ref1.dataset_id.as_str(), ref1.table_id.as_str(), &req)
            .await
            .unwrap();
        assert!(res.insert_errors.is_none());

        // struct
        let mut req2 = InsertAllRequest::<TestData>::default();
        req2.rows.push(Row {
            insert_id: None,
            json: TestData {
                col_string: Some("test3".to_string()),
                col_number: Some(1),
                col_number_array: vec![10, 11, 12],
                col_timestamp: Some(OffsetDateTime::now_utc()),
                col_json: Some("{\"field\":100}".to_string()),
                col_json_array: vec!["{\"field\":100}".to_string(), "{\"field\":200}".to_string()],
                col_struct: Some(TestDataStruct {
                    f1: true,
                    f2: vec![3, 4],
                }),
                col_struct_array: vec![
                    TestDataStruct {
                        f1: true,
                        f2: vec![3, 4],
                    },
                    TestDataStruct {
                        f1: false,
                        f2: vec![30, 40],
                    },
                ],
            },
        });
        req2.rows.push(Row {
            insert_id: None,
            json: TestData {
                col_string: None,
                col_number: None,
                col_number_array: vec![],
                col_timestamp: None,
                col_json: None,
                col_json_array: vec![],
                col_struct: None,
                col_struct_array: vec![],
            },
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
        let mut fetch_request = FetchDataRequest {
            max_results: Some(1),
            ..Default::default()
        };
        let mut data: Vec<list::Row> = vec![];
        loop {
            let result = client
                .read(
                    ref1.project_id.as_str(),
                    ref1.dataset_id.as_str(),
                    ref1.table_id.as_str(),
                    &fetch_request,
                )
                .await
                .unwrap();
            if let Some(rows) = result.rows {
                println!("{:?}", rows);
                data.extend(rows);
            }
            if result.page_token.is_none() {
                break;
            }
            fetch_request.page_token = result.page_token
        }
        assert_eq!(data.len(), 4, "{:?}", data.pop());
        match &data[0].f[0].v {
            Value::String(v) => assert_eq!("test1", v),
            _ => unreachable!(),
        }
        match &data[0].f[2].v {
            Value::Array(v) => assert_eq!(3, v.len()),
            _ => unreachable!(),
        }
        match &data[0].f[4].v {
            Value::String(v) => assert_eq!("{\"field\":100}", v),
            _ => unreachable!(),
        }
        match &data[0].f[6].v {
            Value::Struct(v) => match &v.f[1].v {
                Value::Array(v) => match &v[1].v {
                    Value::String(v) => assert_eq!("4", v, "invalid struct row"),
                    _ => unreachable!("7-1-1 {:?}", v[1].v),
                },
                _ => unreachable!("7-1 {:?}", v.f[1].v),
            },
            _ => unreachable!("7 {:?}", &data[0].f[7].v),
        }

        table_client
            .delete(ref1.project_id.as_str(), ref1.dataset_id.as_str(), ref1.table_id.as_str())
            .await
            .unwrap();
    }
}
