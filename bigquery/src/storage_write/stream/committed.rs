use crate::grpc::apiv1::bigquery_client::{create_write_stream_request, StreamingWriteClient};
use crate::grpc::apiv1::conn_pool::ConnectionManager;
use google_cloud_gax::grpc::{IntoStreamingRequest, Status, Streaming};
use google_cloud_googleapis::cloud::bigquery::storage::v1::write_stream::Type::{Buffered, Committed};
use std::sync::Arc;
use crate::storage_write::stream::{ AsStream, DisposableStream, ManagedStream, Stream};

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

    pub async fn create_write_stream(&mut self, table: &str) -> Result<CommittedStream, Status> {
        let req = create_write_stream_request(table, Committed);
        let stream = self.cm.writer().create_write_stream(req, None).await?.into_inner();
        Ok(CommittedStream::new(Stream::new(stream, self.cm.clone(),self.max_insert_count)))
    }

}

pub struct CommittedStream {
    inner: Stream
}

impl CommittedStream {
    pub(crate) fn new(inner: Stream) -> Self {
        Self { inner }
    }

}

impl AsStream for CommittedStream {
    fn as_mut(&mut self) -> &mut Stream {
        &mut self.inner
    }
    fn as_ref(&self) -> &Stream {
        &self.inner
    }
}
impl ManagedStream for CommittedStream {}
impl DisposableStream for CommittedStream {}
