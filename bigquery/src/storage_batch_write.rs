use crate::grpc::apiv1::bigquery_client::StreamingWriteClient;
use crate::grpc::apiv1::conn_pool::ConnectionManager;
use google_cloud_gax::grpc::{IntoStreamingRequest, Status, Streaming};
use google_cloud_googleapis::cloud::bigquery::storage::v1::big_query_write_client::BigQueryWriteClient;
use google_cloud_googleapis::cloud::bigquery::storage::v1::{
    AppendRowsRequest, AppendRowsResponse, BatchCommitWriteStreamsRequest, BatchCommitWriteStreamsResponse,
    CreateWriteStreamRequest, FinalizeWriteStreamRequest, WriteStream,
};
use std::sync::Arc;

pub struct StorageBatchWriter {
    table: String,
    conn: Arc<ConnectionManager>,
    streams: Vec<String>,
}

impl StorageBatchWriter {
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
            .create_write_stream(
                CreateWriteStreamRequest {
                    parent: self.table.to_string(),
                    write_stream: None,
                },
                None,
            )
            .await?
            .into_inner();

        self.streams.push(res.name.clone());

        Ok(PendingStream::new(res, client))
    }

    pub async fn commit(self) -> Result<BatchCommitWriteStreamsResponse, Status> {
        let mut client = StreamingWriteClient::new(BigQueryWriteClient::new(self.conn.conn()));
        let result = client
            .batch_commit_write_streams(
                BatchCommitWriteStreamsRequest {
                    parent: self.table.to_string(),
                    write_streams: self.streams,
                },
                None,
            )
            .await?
            .into_inner();
        Ok(result)
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
    pub async fn write(
        &mut self,
        req: impl IntoStreamingRequest<Message = AppendRowsRequest>,
    ) -> Result<Streaming<AppendRowsResponse>, Status> {
        let response = self.client.append_rows(req).await?.into_inner();
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
