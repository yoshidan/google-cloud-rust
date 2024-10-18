use std::sync::Arc;
use google_cloud_gax::grpc::{IntoStreamingRequest, Status, Streaming};
use google_cloud_googleapis::cloud::bigquery::storage::v1::{AppendRowsRequest, AppendRowsResponse, CreateWriteStreamRequest, FinalizeWriteStreamRequest, WriteStream};
use crate::grpc::apiv1::conn_pool::ConnectionManager;
use crate::storage_write::flow::FlowController;

pub mod default;
pub mod pending;
pub mod committed;
pub mod buffered;

pub(crate) struct Stream {
    inner: WriteStream,
    cons: Arc<ConnectionManager>,
    fc: Option<FlowController>
}

impl Stream {
    pub(crate) fn new(inner: WriteStream, cons: Arc<ConnectionManager>, max_insert_count: usize) -> Self {
        Self {
            inner,
            cons ,
            fc: if max_insert_count > 0 {
                Some(FlowController::new(max_insert_count))
            }else {
                None
            }
        }
    }
}

pub(crate) trait AsStream : Sized {
    fn as_mut(&mut self) -> &mut Stream;
}

pub trait ManagedStream : AsStream {
    async fn append_rows(&mut self, req: impl IntoStreamingRequest<Message = AppendRowsRequest>) -> Result<Streaming<AppendRowsResponse>, Status> {
        let stream = self.as_mut();
        match &stream.fc {
            None => {
                let mut client = stream.cons.writer();
                Ok(client.append_rows(req).await?.into_inner())
            },
            Some(fc) => {
                let permit = fc.acquire().await;
                let mut client = stream.cons.writer();
                let result = client.append_rows(req).await?.into_inner();
                drop(permit);
                Ok(result)
            }
        }
    }

}

pub trait DisposableStream : ManagedStream {
    async fn finalize(mut self) -> Result<i64, Status> {
        let stream = self.as_mut();
        let res = stream
            .cons.writer()
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
