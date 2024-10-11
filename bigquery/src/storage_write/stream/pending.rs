use crate::grpc::apiv1::bigquery_client::{create_write_stream_request, StreamingWriteClient};
use crate::grpc::apiv1::conn_pool::ConnectionManager;
use google_cloud_gax::grpc::{IntoStreamingRequest, Status, Streaming};
use google_cloud_googleapis::cloud::bigquery::storage::v1::big_query_write_client::BigQueryWriteClient;
use google_cloud_googleapis::cloud::bigquery::storage::v1::write_stream::Type::{Committed, Pending};
use google_cloud_googleapis::cloud::bigquery::storage::v1::{
    AppendRowsRequest, AppendRowsResponse, BatchCommitWriteStreamsRequest, BatchCommitWriteStreamsResponse,
};
use std::sync::Arc;
use crate::storage_write::pool::Pool;
use crate::storage_write::stream::{AsStream, DisposableStream, ManagedStream, Stream};

pub struct Writer {
    cons: Arc<Pool>,
    p_cons: Arc<ConnectionManager>,
    streams: Vec<String>,
    table: String
}

impl Writer {
    pub(crate) fn new(cons: Arc<Pool>, p_cons: Arc<ConnectionManager>, table: String) -> Self {
        Self {
            cons,
            p_cons,
            table,
            streams: Vec::new()
        }
    }

    pub async fn create_write_stream(&mut self) -> Result<PendingStream, Status> {
        let stream = self.cons.create_stream(&self.table, Pending).await?;
        self.streams.push(stream.name.clone());
        Ok(PendingStream::new(Stream::new(stream, self.cons.clone())))
    }

    pub async fn commit(self) -> Result<BatchCommitWriteStreamsResponse, Status> {
        let result = self.cons.client()
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
    inner: Stream
}

impl PendingStream {
    pub(crate) fn new(inner: Stream) -> Self {
        Self { inner }
    }
}

impl AsStream for PendingStream {
    fn as_mut(&mut self) -> &mut Stream {
        &mut self.inner
    }
}
impl ManagedStream for PendingStream {}
impl DisposableStream for PendingStream {}