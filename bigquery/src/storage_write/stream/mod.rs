use std::sync::Arc;
use google_cloud_gax::grpc::{IntoStreamingRequest, Status, Streaming};
use google_cloud_googleapis::cloud::bigquery::storage::v1::{AppendRowsRequest, AppendRowsResponse, CreateWriteStreamRequest, FinalizeWriteStreamRequest, WriteStream};
use crate::grpc::apiv1::conn_pool::ConnectionManager;
use crate::storage_write::AppendRowsRequestBuilder;
use crate::storage_write::flow::FlowController;
use google_cloud_gax::grpc::codegen::tokio_stream::Stream as FuturesStream;

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
    fn as_ref(&self) -> &Stream;

    fn name(&self) -> &str {
        &self.as_ref().inner.name
    }
}

pub trait ManagedStream : AsStream {
    async fn append_rows(&self, req: impl IntoStreamingRequest<Message = AppendRowsRequest>) -> Result<Streaming<AppendRowsResponse>, Status> {
        let stream = self.as_ref();
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
    async fn finalize(&self) -> Result<i64, Status> {
        let stream = self.as_ref();
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


#[cfg(test)]
mod tests {
    use prost_types::{field_descriptor_proto, DescriptorProto, FieldDescriptorProto};
    use crate::storage_write::AppendRowsRequestBuilder;

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub(crate) struct TestData {
        #[prost(string, tag = "1")]
        pub col_string: String,
    }

    pub(crate) fn init() {
        let filter = tracing_subscriber::filter::EnvFilter::from_default_env()
            .add_directive("google_cloud_bigquery=trace".parse().unwrap());
        let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
    }

    pub(crate) fn create_append_rows_request(buf: Vec<Vec<u8>>) -> AppendRowsRequestBuilder {
        let proto = DescriptorProto {
            name: Some("TestData".to_string()),
            field: vec![FieldDescriptorProto {
                name: Some("col_string".to_string()),
                number: Some(1),
                label: None,
                r#type: Some(field_descriptor_proto::Type::String.into()),
                type_name: None,
                extendee: None,
                default_value: None,
                oneof_index: None,
                json_name: None,
                options: None,
                proto3_optional: None,
            }],
            extension: vec![],
            nested_type: vec![],
            enum_type: vec![],
            extension_range: vec![],
            oneof_decl: vec![],
            options: None,
            reserved_range: vec![],
            reserved_name: vec![],
        };
        return AppendRowsRequestBuilder::new(proto, buf)
    }
}
