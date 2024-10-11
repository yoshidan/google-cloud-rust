use crate::grpc::apiv1::conn_pool::ConnectionManager;
use google_cloud_gax::grpc::{IntoStreamingRequest, Status, Streaming};
use google_cloud_googleapis::cloud::bigquery::storage::v1::write_stream::Type::{Buffered, Committed};
use google_cloud_googleapis::cloud::bigquery::storage::v1::{AppendRowsRequest, AppendRowsResponse, BatchCommitWriteStreamsRequest, BatchCommitWriteStreamsResponse, CreateWriteStreamRequest, FinalizeWriteStreamRequest, FlushRowsRequest, WriteStream};
use std::sync::Arc;
use crate::grpc::apiv1::bigquery_client::create_write_stream_request;
use crate::storage_write::stream::{AsStream, DisposableStream, ManagedStream, Stream};

pub struct Writer {
    max_insert_count: usize,
    cm: Arc<ConnectionManager>,
}

impl Writer {
    pub(crate) fn new(max_insert_count: usize, cm: Arc<ConnectionManager>) -> Self {
        Self {
            max_insert_count,
            cm,
        }
    }

    pub async fn create_write_stream(&mut self, table: &str) -> Result<BufferedStream, Status> {
        let req = create_write_stream_request(table, Buffered);
        let stream = self.cm.writer().create_write_stream(req, None).await?.into_inner();
        Ok(BufferedStream::new(Stream::new(stream, self.cm.clone(), self.max_insert_count)))
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
        let res = stream.cons.writer()
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
