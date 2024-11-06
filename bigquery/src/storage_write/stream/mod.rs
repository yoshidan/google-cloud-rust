use crate::grpc::apiv1::conn_pool::ConnectionManager;
use crate::storage_write::flow::FlowController;
use crate::storage_write::AppendRowsRequestBuilder;
use google_cloud_gax::grpc::{IntoStreamingRequest, Status, Streaming};
use google_cloud_googleapis::cloud::bigquery::storage::v1::{
    AppendRowsRequest, AppendRowsResponse, FinalizeWriteStreamRequest, WriteStream,
};
use std::sync::Arc;

pub mod buffered;
pub mod committed;
pub mod default;
pub mod pending;

pub struct Stream {
    inner: WriteStream,
    cons: Arc<ConnectionManager>,
    fc: Option<FlowController>,
}

impl Stream {
    pub(crate) fn new(inner: WriteStream, cons: Arc<ConnectionManager>, max_insert_count: usize) -> Self {
        Self {
            inner,
            cons,
            fc: if max_insert_count > 0 {
                Some(FlowController::new(max_insert_count))
            } else {
                None
            },
        }
    }
}

pub trait AsStream: Sized {
    fn as_ref(&self) -> &Stream;

    fn name(&self) -> &str {
        &self.as_ref().inner.name
    }

    fn create_streaming_request(
        &self,
        rows: Vec<AppendRowsRequestBuilder>,
    ) -> impl google_cloud_gax::grpc::codegen::tokio_stream::Stream<Item = AppendRowsRequest> {
        let name = self.name().to_string();
        async_stream::stream! {
            for row in rows {
                yield row.build(&name);
            }
        }
    }
}

pub(crate) struct ManagedStreamDelegate {}

impl ManagedStreamDelegate {
    async fn append_rows(
        stream: &Stream,
        rows: Vec<AppendRowsRequestBuilder>,
    ) -> Result<Streaming<AppendRowsResponse>, Status> {
        let name = stream.inner.name.to_string();
        let req = async_stream::stream! {
            for row in rows {
                yield row.build(&name);
            }
        };
        Self::append_streaming_request(stream, req).await
    }

    async fn append_streaming_request(
        stream: &Stream,
        req: impl IntoStreamingRequest<Message = AppendRowsRequest>,
    ) -> Result<Streaming<AppendRowsResponse>, Status> {
        match &stream.fc {
            None => {
                let mut client = stream.cons.writer();
                Ok(client.append_rows(req).await?.into_inner())
            }
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

pub(crate) struct DisposableStreamDelegate {}
impl DisposableStreamDelegate {
    async fn finalize(stream: &Stream) -> Result<i64, Status> {
        let res = stream
            .cons
            .writer()
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
    use crate::storage_write::AppendRowsRequestBuilder;
    use prost_types::{field_descriptor_proto, DescriptorProto, FieldDescriptorProto};

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
        AppendRowsRequestBuilder::new(proto, buf)
    }
}
