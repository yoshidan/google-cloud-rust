use crate::grpc::apiv1::bigquery_client::StreamingWriteClient;
use crate::grpc::apiv1::conn_pool::ConnectionManager;
use google_cloud_gax::grpc::{Status, Streaming};
use google_cloud_googleapis::cloud::bigquery::storage::v1::big_query_write_client::BigQueryWriteClient;
use google_cloud_googleapis::cloud::bigquery::storage::v1::{AppendRowsRequest, AppendRowsResponse};
use std::sync::Arc;
use google_cloud_googleapis::cloud::bigquery::storage::v1::write_stream::Type::Buffered;
use crate::storage_write::pool::Pool;
use crate::storage_write::stream::{AsStream, DisposableStream, ManagedStream, Stream};
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

    pub async fn create_write_stream(&mut self, table: &str) -> Result<DefaultStream, Status> {
        let stream = self.cons.get_stream(&format!("{table}/streams/_default")).await?;
        Ok(DefaultStream::new(Stream::new(stream, self.cons.clone())))
    }
}


pub struct DefaultStream {
    inner: Stream
}

impl DefaultStream {
    pub(crate) fn new(inner: Stream) -> Self {
        Self { inner }
    }

}

impl AsStream for DefaultStream {
    fn as_mut(&mut self) -> &mut Stream {
        &mut self.inner
    }
}
impl ManagedStream for DefaultStream {}

