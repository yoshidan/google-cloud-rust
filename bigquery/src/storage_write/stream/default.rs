use crate::grpc::apiv1::conn_pool::ConnectionManager;
use google_cloud_gax::grpc::{Status, };
use std::sync::Arc;
use google_cloud_googleapis::cloud::bigquery::storage::v1::GetWriteStreamRequest;
use crate::storage_write::stream::{AsStream, ManagedStream, Stream};

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

    pub async fn create_write_stream(&mut self, table: &str) -> Result<DefaultStream, Status> {
        let stream = self.cm.writer().get_write_stream(GetWriteStreamRequest {
            name: format!("{table}/streams/_default"),
            ..Default::default()
        }, None).await?.into_inner();
        Ok(DefaultStream::new(Stream::new(stream, self.cm.clone(), self.max_insert_count)))
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

