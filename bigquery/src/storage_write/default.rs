use crate::grpc::apiv1::bigquery_client::StreamingWriteClient;
use crate::grpc::apiv1::conn_pool::ConnectionManager;
use google_cloud_gax::grpc::{Status, Streaming};
use google_cloud_googleapis::cloud::bigquery::storage::v1::big_query_write_client::BigQueryWriteClient;
use google_cloud_googleapis::cloud::bigquery::storage::v1::{AppendRowsRequest, AppendRowsResponse};
use std::sync::Arc;

pub struct Writer {
    conn: Arc<ConnectionManager>,
}

impl Writer {
    pub(crate) fn new(conn: Arc<ConnectionManager>) -> Self {
        Self { conn }
    }

    //TODO use default stream name
    pub async fn append_rows(&mut self, rows: Vec<AppendRowsRequest>) -> Result<Streaming<AppendRowsResponse>, Status> {
        let mut client = StreamingWriteClient::new(BigQueryWriteClient::new(self.conn.conn()));
        let request = Box::pin(async_stream::stream! {
            for row in rows {
                yield row;
            }
        });
        let response = client.append_rows(request).await?.into_inner();
        Ok(response)
    }
}
