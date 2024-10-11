use crate::grpc::apiv1::bigquery_client::{create_write_stream_request, StreamingWriteClient};
use crate::grpc::apiv1::conn_pool::ConnectionManager;
use google_cloud_gax::grpc::{IntoStreamingRequest, Status, Streaming};
use google_cloud_googleapis::cloud::bigquery::storage::v1::big_query_write_client::BigQueryWriteClient;
use google_cloud_googleapis::cloud::bigquery::storage::v1::write_stream::Type::{Buffered, Committed};
use std::sync::Arc;
use crate::storage_write::pool::Pool;
use crate::storage_write::stream::{create_write_stream, AsStream, DisposableStream, ManagedStream, Stream};
use crate::storage_write::stream::buffered::BufferedStream;

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

    pub async fn create_write_stream(&mut self, table: &str) -> Result<CommittedStream, Status> {
        let stream = self.cons.create_stream(table, Committed).await?;
        Ok(CommittedStream::new(Stream::new(stream, self.cons.clone())))
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
}
impl ManagedStream for CommittedStream {}
impl DisposableStream for CommittedStream {}
