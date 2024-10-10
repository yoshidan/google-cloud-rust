use crate::grpc::apiv1::bigquery_client::{create_write_stream_request, StreamingWriteClient};
use crate::grpc::apiv1::conn_pool::ConnectionManager;
use google_cloud_gax::grpc::{IntoStreamingRequest, Status, Streaming};
use google_cloud_googleapis::cloud::bigquery::storage::v1::big_query_write_client::BigQueryWriteClient;
use google_cloud_googleapis::cloud::bigquery::storage::v1::write_stream::Type::{Buffered, Committed};
use google_cloud_googleapis::cloud::bigquery::storage::v1::{AppendRowsRequest, AppendRowsResponse, BatchCommitWriteStreamsRequest, BatchCommitWriteStreamsResponse, CreateWriteStreamRequest, FinalizeWriteStreamRequest, FlushRowsRequest, WriteStream};
use std::sync::Arc;
use crate::storage_write::stream::{AsStream, DisposableStream, ManagedStream, Stream};

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

    pub async fn create_write_stream(&mut self) -> Result<BufferedStream, Status> {
        let mut client = StreamingWriteClient::new(BigQueryWriteClient::new(self.conn.conn()));
        let res = client
            .create_write_stream(create_write_stream_request(&self.table, Buffered), None)
            .await?
            .into_inner();

        self.streams.push(res.name.clone());

        Ok(BufferedStream::new(Stream::new(res, client)))
    }

}

pub struct BufferedStream {
    inner: Stream
}

impl BufferedStream {
    pub(crate) fn new(inner: Stream) -> Self {
        Self { inner }
    }
}

impl AsStream for BufferedStream {
    fn as_mut(&mut self) -> &mut Stream {
        &mut self.inner
    }
}
impl ManagedStream for BufferedStream {}
impl DisposableStream for BufferedStream {}

impl BufferedStream {

    pub async fn flush_rows(mut self) -> Result<i64, Status> {
        let stream = self.as_mut();
        let res = stream.client
            .flush_rows(
                FlushRowsRequest{
                    write_stream: stream.inner.name.to_string(),
                    offset: None,
                },
                None,
            )
            .await?
            .into_inner();
        Ok(res.offset)
    }

}
