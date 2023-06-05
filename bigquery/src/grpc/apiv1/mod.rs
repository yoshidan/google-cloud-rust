pub mod bigquery_client;
pub mod conn_pool;


#[cfg(test)]
pub mod test {
    use std::io::{BufReader, Cursor};
    use arrow::ipc::reader::StreamReader;
    use serial_test::serial;
    use google_cloud_auth::project::Config;
    use google_cloud_auth::token::DefaultTokenSourceProvider;
    use google_cloud_gax::conn::Environment;
    use google_cloud_googleapis::cloud::bigquery::storage::v1::{CreateReadSessionRequest, DataFormat, ReadRowsRequest, ReadSession};
    use google_cloud_googleapis::cloud::bigquery::storage::v1::read_rows_response::{Rows, Schema};
    use crate::grpc::apiv1::bigquery_client::ReadClient;
    use crate::grpc::apiv1::conn_pool::{AUDIENCE, DOMAIN, ReadConnectionManager};
    use crate::http::bigquery_client::SCOPES;

    async fn create_read_client() -> ReadClient {
        let tsp = DefaultTokenSourceProvider::new(Config {
            audience: Some(AUDIENCE),
            scopes: Some(SCOPES.as_ref()),
            sub: None,
        }).await.unwrap();
        let cm = ReadConnectionManager::new(1, &Environment::GoogleCloud(Box::new(tsp)), DOMAIN)
            .await
            .unwrap();
        cm.conn()
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial]
    async fn test_read() {
        let mut client = create_read_client().await;
        let table_id =  "projects/atl-dev1/datasets/rust_test_table/tables/table_data_1682321746";
        let response = client.create_read_session(CreateReadSessionRequest {
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
            preferred_min_stream_count: 0
        }, None).await.unwrap();
        assert_eq!(response.get_ref().table.as_str(), table_id);
        assert_eq!(response.get_ref().estimated_row_count, 10);
        assert!(response.get_ref().streams.len() > 0);

        let streams = response.into_inner().streams;
        let requests : Vec<ReadRowsRequest> = streams.iter().map(|e| ReadRowsRequest {
            read_stream: e.name.to_string() ,
            offset: 0
        }).collect();

        for request in requests {
            let rows = client.read_rows(request, None).await.unwrap();
            let mut response = rows.into_inner();
            while let Some(response) = response.message().await.unwrap() {
                let schema = match response.schema.unwrap() {
                    Schema::ArrowSchema(schema) => schema,
                    _ => unreachable!("unsupported schema")
                };
                let schema_data = Cursor::new(schema.serialized_schema);
                let arrow_schema: StreamReader<BufReader<Cursor<Vec<u8>>>> = arrow::ipc::reader::StreamReader::try_new(schema_data, None).unwrap();
                tracing::info!("schema {:?}", arrow_schema);

                if let Some(rows) = response.rows {
                    match rows {
                        Rows::ArrowRecordBatch(rows) => {
                        }
                        _ => unreachable!("unsupported rows")
                    }
                }
            }
        }


    }
}