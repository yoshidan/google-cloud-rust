use crate::grpc::apiv1::bigquery_client::{create_write_stream_request, StreamingWriteClient};
use crate::grpc::apiv1::conn_pool::ConnectionManager;
use google_cloud_gax::grpc::{IntoStreamingRequest, Status, Streaming};
use google_cloud_googleapis::cloud::bigquery::storage::v1::big_query_write_client::BigQueryWriteClient;
use google_cloud_googleapis::cloud::bigquery::storage::v1::write_stream::Type::Committed;
use google_cloud_googleapis::cloud::bigquery::storage::v1::{
    AppendRowsRequest, AppendRowsResponse, BatchCommitWriteStreamsRequest, BatchCommitWriteStreamsResponse,
    CreateWriteStreamRequest, FinalizeWriteStreamRequest, WriteStream,
};
use std::sync::Arc;

pub struct Writer {
    table: String,
    conn: Arc<ConnectionManager>,
    streams: Vec<String>,
}

impl Writer {
    pub(crate) fn new(table: String, conn: Arc<ConnectionManager>) -> Self {
        Self {
            table,
            conn,
            streams: Vec::new(),
        }
    }

    pub async fn create_write_stream(&mut self) -> Result<PendingStream, Status> {
        let mut client = StreamingWriteClient::new(BigQueryWriteClient::new(self.conn.conn()));
        let res = client
            .create_write_stream(create_write_stream_request(&self.table, Committed), None)
            .await?
            .into_inner();

        self.streams.push(res.name.clone());

        Ok(PendingStream::new(res, client))
    }

}

pub struct PendingStream {
    inner: WriteStream,
    client: StreamingWriteClient,
}

impl PendingStream {
    pub(crate) fn new(inner: WriteStream, client: StreamingWriteClient) -> Self {
        Self { inner, client }
    }

    //TODO serialize values and get schema
    pub async fn append_rows(&mut self, rows: Vec<AppendRowsRequest>) -> Result<Streaming<AppendRowsResponse>, Status> {
        let request = Box::pin(async_stream::stream! {
            for row in rows {
                yield row;
            }
        });
        let response = self.client.append_rows(request).await?.into_inner();
        Ok(response)
    }

    pub async fn finalize(mut self) -> Result<i64, Status> {
        let res = self
            .client
            .finalize_write_stream(
                FinalizeWriteStreamRequest {
                    name: self.inner.name.to_string(),
                },
                None,
            )
            .await?
            .into_inner();
        Ok(res.row_count)
    }
}
