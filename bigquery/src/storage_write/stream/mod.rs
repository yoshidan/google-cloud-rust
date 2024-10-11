use std::sync::Arc;
use google_cloud_gax::grpc::{Status, Streaming};
use google_cloud_googleapis::cloud::bigquery::storage::v1::{AppendRowsRequest, AppendRowsResponse, CreateWriteStreamRequest, FinalizeWriteStreamRequest, WriteStream};
use google_cloud_googleapis::cloud::bigquery::storage::v1::big_query_write_client::BigQueryWriteClient;
use google_cloud_googleapis::cloud::bigquery::storage::v1::write_stream::Type::Buffered;
use crate::grpc::apiv1::bigquery_client::{create_write_stream_request, StreamingWriteClient};
use crate::grpc::apiv1::conn_pool::ConnectionManager;
use crate::storage_write::into_streaming_request;
use crate::storage_write::pool::{Pool};
use crate::storage_write::stream::buffered::BufferedStream;

pub mod default;
pub mod pending;
pub mod committed;
pub mod buffered;

pub(crate) struct Stream {
    pub(crate) inner: WriteStream,
    pub(crate) cons: Arc<Pool>,
}

impl Stream {
    pub(crate) fn new(inner: WriteStream, cons: Arc<Pool>) -> Self {
        Self { inner, cons }
    }
}

pub(crate) trait AsStream : Sized {
    fn as_mut(&mut self) -> &mut Stream;
}

pub trait ManagedStream : AsStream {
    async fn append_rows(&mut self, rows: Vec<AppendRowsRequest>) -> Result<Streaming<AppendRowsResponse>, Status> {
        let stream = self.as_mut();
        let cons = stream.cons.regional(&stream.inner.location);
        let con = cons.pick(&stream.inner.name).unwrap();
        con.locking_append(into_streaming_request(rows)).await
    }

}

pub trait DisposableStream : ManagedStream {
    async fn finalize(mut self) -> Result<i64, Status> {
        let stream = self.as_mut();
        let res = stream
            .cons.client()
            .finalize_write_stream(
                FinalizeWriteStreamRequest {
                    name: stream.inner.name.to_string(),
                },
                None,
            )
            .await?
            .into_inner();
        Ok(res.row_count)
    }
}
