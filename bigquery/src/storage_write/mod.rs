use google_cloud_googleapis::cloud::bigquery::storage::v1::append_rows_request::{ProtoData, Rows};
use google_cloud_googleapis::cloud::bigquery::storage::v1::{AppendRowsRequest, ProtoRows, ProtoSchema};
use prost_types::DescriptorProto;
use std::collections::HashMap;

pub mod default;
pub mod pending;
pub mod committed;
pub mod buffered;
mod flow_controller;

pub struct AppendRowsRequestBuilder {
    offset: Option<i64>,
    trace_id: Option<String>,
    missing_value_interpretations: Option<HashMap<String, i32>>,
    default_missing_value_interpretation: Option<i32>,
    data: Vec<Vec<u8>>,
    schema: DescriptorProto,
}

impl AppendRowsRequestBuilder {
    pub fn new(schema: DescriptorProto, data: Vec<Vec<u8>>) -> Self {
        Self {
            offset: None,
            trace_id: None,
            missing_value_interpretations: None,
            default_missing_value_interpretation: None,
            data,
            schema,
        }
    }

    pub fn with_offset(mut self, offset: i64) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn with_trace_id(mut self, trace_id: String) -> Self {
        self.trace_id = Some(trace_id);
        self
    }

    pub fn with_missing_value_interpretations(mut self, missing_value_interpretations: HashMap<String, i32>) -> Self {
        self.missing_value_interpretations = Some(missing_value_interpretations);
        self
    }

    pub fn with_default_missing_value_interpretation(mut self, default_missing_value_interpretation: i32) -> Self {
        self.default_missing_value_interpretation = Some(default_missing_value_interpretation);
        self
    }

    pub(crate) fn build(self, stream: &str) -> AppendRowsRequest {
        AppendRowsRequest {
            write_stream: stream.to_string(),
            offset: self.offset,
            trace_id: self.trace_id.unwrap_or_default(),
            missing_value_interpretations: self.missing_value_interpretations.unwrap_or_default(),
            default_missing_value_interpretation: self.default_missing_value_interpretation.unwrap_or(0),
            rows: Some(Rows::ProtoRows(ProtoData {
                writer_schema: Some(ProtoSchema {
                    proto_descriptor: Some(self.schema),
                }),
                rows: Some(ProtoRows {
                    serialized_rows: self.data,
                }),
            })),
        }
    }
}
