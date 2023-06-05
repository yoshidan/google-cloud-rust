pub mod bigquery_client;
pub mod conn_pool;

#[cfg(test)]
pub mod test {
    use crate::arrow::ArrowDecodable;
    use crate::grpc::apiv1::bigquery_client::ReadClient;
    use crate::grpc::apiv1::conn_pool::{ReadConnectionManager, AUDIENCE, DOMAIN};
    use crate::http::bigquery_client::SCOPES;
    use crate::types::Numeric;
    use arrow::datatypes::{DataType, FieldRef, TimeUnit};
    use arrow::ipc::reader::StreamReader;
    use arrow::ipc::Field;
    use google_cloud_auth::project::Config;
    use google_cloud_auth::token::DefaultTokenSourceProvider;
    use google_cloud_gax::conn::Environment;
    use google_cloud_googleapis::cloud::bigquery::storage::v1::read_rows_response::{Rows, Schema};
    use google_cloud_googleapis::cloud::bigquery::storage::v1::{
        CreateReadSessionRequest, DataFormat, ReadRowsRequest, ReadSession,
    };
    use serial_test::serial;
    use std::io::{BufReader, Cursor};
    use time::OffsetDateTime;

    async fn create_read_client() -> ReadClient {
        let tsp = DefaultTokenSourceProvider::new(Config {
            audience: Some(AUDIENCE),
            scopes: Some(SCOPES.as_ref()),
            sub: None,
        })
        .await
        .unwrap();
        let cm = ReadConnectionManager::new(1, &Environment::GoogleCloud(Box::new(tsp)), DOMAIN)
            .await
            .unwrap();
        cm.conn()
    }

    #[derive(Default)]
    struct TestDataStruct {
        pub f1: bool,
        pub f2: Vec<i64>,
    }
    #[derive(Default)]
    struct TestData {
        pub col_string: Option<String>,
        pub col_number: Option<Numeric>,
        pub col_number_array: Vec<Numeric>,
        pub col_timestamp: Option<OffsetDateTime>,
        pub col_json: Option<String>,
        pub col_json_array: Vec<String>,
        pub col_struct: Option<TestDataStruct>,
        pub col_struct_array: Vec<TestDataStruct>,
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_read() {
        let mut client = create_read_client().await;
        let table_id = "projects/atl-dev1/datasets/rust_test_table/tables/table_data_1682321746";
        let response = client
            .create_read_session(
                CreateReadSessionRequest {
                    parent: "projects/atl-dev1".to_string(),
                    read_session: Some(ReadSession {
                        name: "".to_string(),
                        expire_time: None,
                        data_format: DataFormat::Arrow.into(),
                        table: table_id.to_string(),
                        table_modifiers: None,
                        read_options: None,
                        streams: vec![],
                        estimated_total_bytes_scanned: 0,
                        estimated_row_count: 0,
                        trace_id: "".to_string(),
                        schema: None,
                    }),
                    max_stream_count: 0,
                    preferred_min_stream_count: 0,
                },
                None,
            )
            .await
            .unwrap();
        assert_eq!(response.get_ref().table.as_str(), table_id);
        assert_eq!(response.get_ref().estimated_row_count, 10);
        assert!(response.get_ref().streams.len() > 0);

        let streams = response.into_inner().streams;
        let requests: Vec<ReadRowsRequest> = streams
            .iter()
            .map(|e| ReadRowsRequest {
                read_stream: e.name.to_string(),
                offset: 0,
            })
            .collect();

        for request in requests {
            let rows = client.read_rows(request, None).await.unwrap();
            let mut response = rows.into_inner();
            while let Some(response) = response.message().await.unwrap() {
                let schema = match response.schema.unwrap() {
                    Schema::ArrowSchema(schema) => schema,
                    _ => unreachable!("unsupported schema"),
                };
                let schema_data = Cursor::new(schema.serialized_schema.clone());
                let arrow_schema: StreamReader<BufReader<Cursor<Vec<u8>>>> =
                    arrow::ipc::reader::StreamReader::try_new(schema_data, None).unwrap();
                tracing::info!("schema {:?}", arrow_schema);

                if let Some(rows) = response.rows {
                    match rows {
                        Rows::ArrowRecordBatch(rows) => {
                            let mut rows_with_schema = schema.clone().serialized_schema;
                            rows_with_schema.extend_from_slice(&rows.serialized_record_batch);
                            let rows = Cursor::new(rows_with_schema);
                            let rows: StreamReader<BufReader<Cursor<Vec<u8>>>> =
                                arrow::ipc::reader::StreamReader::try_new(rows, None).unwrap();
                            rows.for_each(|row| {
                                let row = row.unwrap();
                                assert_eq!(row.schema().fields().len(), 8);
                                assert_eq!(row.schema().fields()[0].data_type(), &DataType::Utf8);
                                assert_eq!(row.schema().fields()[0].name(), "col_string");
                                assert!(row.schema().fields()[0].is_nullable());

                                assert_eq!(row.schema().fields()[1].data_type(), &DataType::Decimal128(38, 9));
                                assert_eq!(row.schema().fields()[1].name(), "col_number");
                                assert!(row.schema().fields()[1].is_nullable());

                                match row.schema().fields()[2].data_type() {
                                    DataType::List(field) => {
                                        assert_eq!(field.name(), "item");
                                        assert_eq!(field.data_type(), &DataType::Decimal128(38, 9));
                                        assert!(field.is_nullable());
                                    }
                                    _ => unreachable!("unsupported rows"),
                                };
                                assert_eq!(row.schema().fields()[2].name(), "col_number_array");
                                assert!(!row.schema().fields()[2].is_nullable());

                                assert_eq!(
                                    row.schema().fields()[3].data_type(),
                                    &DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into()))
                                );
                                assert_eq!(row.schema().fields()[3].name(), "col_timestamp");
                                assert!(row.schema().fields()[3].is_nullable());

                                assert_eq!(row.schema().fields()[4].data_type(), &DataType::Utf8);
                                assert_eq!(row.schema().fields()[4].name(), "col_json");
                                assert!(row.schema().fields()[4].is_nullable());
                                assert_eq!(
                                    row.schema().fields()[4].metadata().get("ARROW:extension:name").unwrap(),
                                    "google:sqlType:json"
                                );

                                match row.schema().fields()[5].data_type() {
                                    DataType::List(field) => {
                                        assert_eq!(field.name(), "item");
                                        assert_eq!(field.data_type(), &DataType::Utf8);
                                        assert!(field.is_nullable());
                                        assert!(field.metadata().is_empty());
                                    }
                                    _ => unreachable!("invalid array type"),
                                };
                                assert_eq!(row.schema().fields()[5].name(), "col_json_array");
                                assert!(!row.schema().fields()[5].is_nullable());
                                assert_eq!(
                                    row.schema().fields()[5].metadata().get("ARROW:extension:name").unwrap(),
                                    "google:sqlType:json"
                                );

                                match row.schema().fields()[6].data_type() {
                                    DataType::Struct(fields) => {
                                        assert_eq!(fields[0].name(), "f1");
                                        assert_eq!(fields[0].data_type(), &DataType::Boolean);
                                        assert!(fields[0].is_nullable());
                                        assert_eq!(fields[1].name(), "f2");
                                        match fields[1].data_type() {
                                            DataType::List(fields) => {
                                                assert_eq!(fields.name(), "item");
                                                assert_eq!(fields.data_type(), &DataType::Int64);
                                                assert!(fields.is_nullable());
                                            }
                                            _ => unreachable!("invalid array in struct type"),
                                        }
                                        assert!(!fields[1].is_nullable());
                                    }
                                    _ => unreachable!("invalid struct type"),
                                }
                                assert_eq!(row.schema().fields()[6].name(), "col_struct");
                                assert!(row.schema().fields()[6].is_nullable());

                                match row.schema().fields()[7].data_type() {
                                    DataType::List(field) => {
                                        assert_eq!(field.name(), "item");
                                        match field.data_type() {
                                            DataType::Struct(fields) => {
                                                assert_eq!(fields[0].name(), "f1");
                                                assert_eq!(fields[0].data_type(), &DataType::Boolean);
                                                assert!(fields[0].is_nullable());
                                                assert_eq!(fields[1].name(), "f2");
                                                match fields[1].data_type() {
                                                    DataType::List(fields) => {
                                                        assert_eq!(fields.name(), "item");
                                                        assert_eq!(fields.data_type(), &DataType::Int64);
                                                        assert!(fields.is_nullable());
                                                    }
                                                    _ => unreachable!("invalid array in struct type"),
                                                }
                                                assert!(!fields[1].is_nullable());
                                            }
                                            _ => unreachable!("invalid array in struct type"),
                                        }
                                        assert!(field.is_nullable());
                                        assert!(field.metadata().is_empty());
                                    }
                                    _ => unreachable!("invalid array type"),
                                };
                                assert_eq!(row.schema().fields()[7].name(), "col_struct_array");
                                assert!(!row.schema().fields()[7].is_nullable());

                                let mut data: Vec<TestData> = Vec::with_capacity(row.num_rows());
                                for _i in 0..row.num_rows() {
                                    data.push(TestData::default())
                                }
                                let column = row.column(0);
                                for row_no in 0..column.len() {
                                    data[row_no].col_string = Option::<String>::decode(column, row_no).unwrap();
                                }
                                let column = row.column(1);
                                for row_no in 0..column.len() {
                                    data[row_no].col_number = Option::<Numeric>::decode(column, row_no).unwrap();
                                }
                            });
                        }
                        _ => unreachable!("unsupported rows"),
                    }
                }
            }
        }
    }
}
