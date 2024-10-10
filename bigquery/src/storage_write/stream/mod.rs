use google_cloud_gax::grpc::{Status, Streaming};
use google_cloud_googleapis::cloud::bigquery::storage::v1::{AppendRowsRequest, AppendRowsResponse, FinalizeWriteStreamRequest, WriteStream};
use crate::grpc::apiv1::bigquery_client::StreamingWriteClient;
use crate::storage_write::into_streaming_request;

pub mod default;
pub mod pending;
pub mod committed;
pub mod buffered;

pub(crate) struct Stream {
    pub(crate) inner: WriteStream,
    pub(crate) client: StreamingWriteClient,
}

impl Stream {
    pub(crate) fn new(inner: WriteStream, client: StreamingWriteClient) -> Self {
        Self { inner, client }
    }
}

pub(crate) trait AsStream : Sized {
    fn as_mut(&mut self) -> &mut Stream;
}

pub trait ManagedStream : AsStream {
    async fn append_rows(&mut self, rows: Vec<AppendRowsRequest>) -> Result<Streaming<AppendRowsResponse>, Status> {
        let response = self.as_mut().client.append_rows(into_streaming_request(rows)).await?.into_inner();
        Ok(response)
    }

}

pub trait DisposableStream : ManagedStream {
    async fn finalize(mut self) -> Result<i64, Status> {
        let stream = self.as_mut();
        let res = stream
            .client
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