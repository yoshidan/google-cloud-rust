use crate::grpc::apiv1::bigquery_client::{create_write_stream_request, StreamingWriteClient};
use crate::grpc::apiv1::conn_pool::ConnectionManager;
use google_cloud_gax::grpc::{IntoStreamingRequest, Status, Streaming};
use google_cloud_googleapis::cloud::bigquery::storage::v1::write_stream::Type::{Buffered, Committed};
use google_cloud_googleapis::cloud::bigquery::storage::v1::{AppendRowsRequest, AppendRowsResponse, BatchCommitWriteStreamsRequest, BatchCommitWriteStreamsResponse, CreateWriteStreamRequest, FinalizeWriteStreamRequest, FlushRowsRequest, WriteStream};
use std::sync::Arc;
use crate::storage_write::pool::{Pool};
use crate::storage_write::stream::{AsStream, DisposableStream, ManagedStream, Stream};

pub struct Writer {
    cons: Arc<Pool>,
    p_cons: Arc<ConnectionManager>,
}

impl Writer {
    pub(crate) fn new(cons: Arc<Pool>, p_cons: Arc<ConnectionManager>) -> Self {
        Self {
            cons,
            p_cons,
        }
    }

    pub async fn create_write_stream(&mut self, table: &str) -> Result<BufferedStream, Status> {
        let stream = self.cons.create_stream(table, Buffered).await?;
        Ok(BufferedStream::new(Stream::new(stream, self.cons.clone())))
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
        let res = stream.cons.client()
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
