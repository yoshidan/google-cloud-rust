use std::collections::HashMap;

use arrow::error::ArrowError;
use arrow::ipc::writer::{
    write_message, CompressionContext, DictionaryTracker, EncodedData, IpcDataGenerator, IpcWriteOptions,
};
use arrow::record_batch::RecordBatch;
use google_cloud_gax::grpc::codegen::tokio_stream::Stream;
use google_cloud_googleapis::cloud::bigquery::storage::v1::append_rows_request::{ArrowData, ProtoData, Rows};
use google_cloud_googleapis::cloud::bigquery::storage::v1::{
    AppendRowsRequest, ArrowRecordBatch, ArrowSchema, ProtoRows, ProtoSchema,
};
use prost_types::DescriptorProto;

mod flow;
pub mod stream;

enum Payload {
    Proto {
        schema: DescriptorProto,
        rows: Vec<Vec<u8>>,
    },
    Arrow {
        serialized_schema: Vec<u8>,
        serialized_record_batch: Vec<u8>,
    },
}

pub struct AppendRowsRequestBuilder {
    offset: Option<i64>,
    trace_id: Option<String>,
    missing_value_interpretations: Option<HashMap<String, i32>>,
    default_missing_value_interpretation: Option<i32>,
    payload: Payload,
}

impl AppendRowsRequestBuilder {
    pub fn new(schema: DescriptorProto, data: Vec<Vec<u8>>) -> Self {
        Self::with_payload(Payload::Proto { schema, rows: data })
    }

    pub fn new_arrow(serialized_schema: Vec<u8>, serialized_record_batch: Vec<u8>) -> Self {
        Self::with_payload(Payload::Arrow {
            serialized_schema,
            serialized_record_batch,
        })
    }

    pub fn from_record_batch(batch: &RecordBatch) -> Result<Self, ArrowError> {
        let options = IpcWriteOptions::default();
        let generator = IpcDataGenerator::default();
        let mut dict_tracker = DictionaryTracker::new(true);
        let mut compression = CompressionContext::default();

        let schema_encoded =
            generator.schema_to_bytes_with_dictionary_tracker(&batch.schema(), &mut dict_tracker, &options);
        let serialized_schema = encoded_to_bytes(vec![schema_encoded], &options)?;

        let (dict_encoded, batch_encoded) =
            generator.encode(batch, &mut dict_tracker, &options, &mut compression)?;
        let mut encoded = dict_encoded;
        encoded.push(batch_encoded);
        let serialized_record_batch = encoded_to_bytes(encoded, &options)?;

        Ok(Self::new_arrow(serialized_schema, serialized_record_batch))
    }

    fn with_payload(payload: Payload) -> Self {
        Self {
            offset: None,
            trace_id: None,
            missing_value_interpretations: None,
            default_missing_value_interpretation: None,
            payload,
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
        let rows = match self.payload {
            Payload::Proto { schema, rows } => Rows::ProtoRows(ProtoData {
                writer_schema: Some(ProtoSchema {
                    proto_descriptor: Some(schema),
                }),
                rows: Some(ProtoRows { serialized_rows: rows }),
            }),
            Payload::Arrow {
                serialized_schema,
                serialized_record_batch,
            } => Rows::ArrowRows(ArrowData {
                writer_schema: Some(ArrowSchema { serialized_schema }),
                rows: Some(ArrowRecordBatch {
                    serialized_record_batch,
                    #[allow(deprecated)]
                    row_count: 0,
                }),
            }),
        };
        AppendRowsRequest {
            write_stream: stream.to_string(),
            offset: self.offset,
            trace_id: self.trace_id.unwrap_or_default(),
            missing_value_interpretations: self.missing_value_interpretations.unwrap_or_default(),
            default_missing_value_interpretation: self.default_missing_value_interpretation.unwrap_or(0),
            rows: Some(rows),
        }
    }
}

fn encoded_to_bytes(messages: Vec<EncodedData>, options: &IpcWriteOptions) -> Result<Vec<u8>, ArrowError> {
    let mut buf = Vec::new();
    for message in messages {
        write_message(&mut buf, message, options)?;
    }
    Ok(buf)
}

pub fn into_streaming_request(rows: Vec<AppendRowsRequest>) -> impl Stream<Item = AppendRowsRequest> {
    async_stream::stream! {
        for row in rows {
            yield row;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Cursor};
    use std::sync::Arc;

    use arrow::array::{Int64Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::ipc::reader::StreamReader;
    use arrow::record_batch::RecordBatch;
    use google_cloud_googleapis::cloud::bigquery::storage::v1::append_rows_request::Rows;

    use super::AppendRowsRequestBuilder;

    fn sample_batch() -> RecordBatch {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("name", DataType::Utf8, false),
        ]));
        let ids = Arc::new(Int64Array::from(vec![1, 2, 3]));
        let names = Arc::new(StringArray::from(vec!["a", "b", "c"]));
        RecordBatch::try_new(schema, vec![ids, names]).unwrap()
    }

    #[test]
    fn from_record_batch_emits_arrow_rows_and_round_trips() {
        let batch = sample_batch();
        let expected_rows = batch.num_rows();

        let builder = AppendRowsRequestBuilder::from_record_batch(&batch).unwrap();
        let request = builder.build("projects/p/datasets/d/tables/t/streams/_default");

        let Rows::ArrowRows(arrow_data) = request.rows.expect("rows set") else {
            panic!("expected Arrow rows variant");
        };
        let schema_bytes = arrow_data.writer_schema.expect("writer_schema").serialized_schema;
        let batch_bytes = arrow_data.rows.expect("rows").serialized_record_batch;
        assert!(!schema_bytes.is_empty());
        assert!(!batch_bytes.is_empty());

        // Mirror storage.rs: concat schema + batch and decode with StreamReader.
        let mut combined = schema_bytes;
        combined.extend_from_slice(&batch_bytes);
        let reader = StreamReader::try_new(BufReader::new(Cursor::new(combined)), None).unwrap();
        let decoded: Vec<RecordBatch> = reader.collect::<Result<_, _>>().unwrap();
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].num_rows(), expected_rows);
        assert_eq!(decoded[0].num_columns(), 2);
    }
}
