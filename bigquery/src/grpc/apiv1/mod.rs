pub mod bigquery_client;
pub mod conn_pool;

#[cfg(test)]
pub mod test {
    use crate::arrow::{ArrowDecodable, ArrowStructDecodable, Decimal128, Error};
    use crate::grpc::apiv1::bigquery_client::{StreamingReadClient};
    use crate::grpc::apiv1::conn_pool::{ReadConnectionManager, AUDIENCE, DOMAIN};
    use crate::http::bigquery_client::test::TestDataStruct;
    use crate::http::bigquery_client::SCOPES;
    use arrow::array::{Array, ArrayRef};
    use arrow::datatypes::{DataType, TimeUnit};
    use arrow::ipc::reader::StreamReader;
    use google_cloud_auth::project::Config;
    use google_cloud_auth::token::DefaultTokenSourceProvider;
    use google_cloud_gax::conn::Environment;
    
    
    use google_cloud_googleapis::cloud::bigquery::storage::v1::read_rows_response::{Rows, Schema};
    use google_cloud_googleapis::cloud::bigquery::storage::v1::{
        ArrowSchema, CreateReadSessionRequest, DataFormat, ReadRowsRequest, ReadSession,
    };
    use serial_test::serial;
    use std::io::{BufReader, Cursor};
    use time::OffsetDateTime;

    async fn create_read_client() -> StreamingReadClient {
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

    #[derive(Debug, Default)]
    pub struct TestData {
        pub col_string: Option<String>,
        pub col_number: Option<Decimal128>,
        pub col_number_array: Vec<Decimal128>,
        pub col_timestamp: Option<OffsetDateTime>,
        pub col_json: Option<String>,
        pub col_json_array: Vec<String>,
        pub col_struct: Option<TestDataStruct>,
        pub col_struct_array: Vec<TestDataStruct>,
    }

    impl ArrowStructDecodable<TestData> for TestData {
        fn decode(col: &[ArrayRef], row_no: usize) -> Result<TestData, Error> {
            let col_string = Option::<String>::decode(&col[0], row_no)?;
            let col_number = Option::<Decimal128>::decode(&col[1], row_no)?;
            let col_number_array = Vec::<Decimal128>::decode(&col[2], row_no)?;
            let col_timestamp = Option::<OffsetDateTime>::decode(&col[3], row_no)?;
            let col_json = Option::<String>::decode(&col[4], row_no)?;
            let col_json_array = Vec::<String>::decode(&col[5], row_no)?;
            let col_struct = Option::<TestDataStruct>::decode(&col[6], row_no)?;
            let col_struct_array = Vec::<TestDataStruct>::decode(&col[7], row_no)?;
            Ok(TestData {
                col_string,
                col_number,
                col_number_array,
                col_timestamp,
                col_json,
                col_json_array,
                col_struct,
                col_struct_array,
            })
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_read() {
        let mut client = create_read_client().await;
        let table_id = "projects/atl-dev1/datasets/rust_test_table/tables/table_data_1686033753";
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
                    max_stream_count: 100,
                    preferred_min_stream_count: 10,
                },
                None,
            )
            .await
            .unwrap();
        assert_eq!(response.get_ref().table.as_str(), table_id);
        assert!(!response.get_ref().streams.is_empty());
        tracing::info!("stream count = {}", response.get_ref().streams.len());

        let streams = response.into_inner().streams;
        let requests: Vec<ReadRowsRequest> = streams
            .iter()
            .map(|e| ReadRowsRequest {
                read_stream: e.name.to_string(),
                offset: 0,
            })
            .collect();

        let mut table_data = vec![];
        for request in requests {
            let rows = client.read_rows(request, None).await.unwrap();
            let mut response = rows.into_inner();

            let mut schema: Option<ArrowSchema> = None;
            while let Some(response) = response.message().await.unwrap() {
                if let Some(first_row_schema) = response.schema {
                    schema = match first_row_schema {
                        Schema::ArrowSchema(first_row_schema) => {
                            //let schema_data = Cursor::new(schema.serialized_schema.clone());
                            //let arrow_schema: StreamReader<BufReader<Cursor<Vec<u8>>>> =arrow::ipc::reader::StreamReader::try_new(schema_data, None).unwrap();
                            // tracing::info!("schema {:?}", arrow_schema);
                            Some(first_row_schema)
                        }
                        _ => unreachable!("unsupported schema"),
                    }
                };
                let schema = schema.clone().unwrap();
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
                                    data[row_no].col_number = Option::<Decimal128>::decode(column, row_no).unwrap();
                                }
                                let column = row.column(2);
                                for row_no in 0..column.len() {
                                    data[row_no].col_number_array = Vec::<Decimal128>::decode(column, row_no).unwrap();
                                }
                                let column = row.column(3);
                                for row_no in 0..column.len() {
                                    data[row_no].col_timestamp =
                                        Option::<OffsetDateTime>::decode(column, row_no).unwrap();
                                }
                                let column = row.column(4);
                                for row_no in 0..column.len() {
                                    data[row_no].col_json = Option::<String>::decode(column, row_no).unwrap();
                                }
                                let column = row.column(5);
                                for row_no in 0..column.len() {
                                    data[row_no].col_json_array = Vec::<String>::decode(column, row_no).unwrap();
                                }
                                let column = row.column(6);
                                for row_no in 0..column.len() {
                                    data[row_no].col_struct = Option::<TestDataStruct>::decode(column, row_no).unwrap();
                                }
                                let column = row.column(7);
                                for row_no in 0..column.len() {
                                    data[row_no].col_struct_array =
                                        Vec::<TestDataStruct>::decode(column, row_no).unwrap();
                                }
                                table_data.extend(data);
                            });
                        }
                        _ => unreachable!("unsupported rows"),
                    }
                }
            }
        }
        assert_eq!(table_data.len(), 34);
    }
}
