use crate::arrow::ArrowStructDecodable;
use crate::grpc::apiv1::bigquery_client::StreamingReadClient;
use crate::http::error::Error as HttpError;
use crate::http::tabledata::list::Tuple;
use arrow::ipc::reader::StreamReader;

use arrow::error::ArrowError;
use google_cloud_gax::grpc::{Status, Streaming};
use google_cloud_gax::retry::RetrySetting;
use google_cloud_googleapis::cloud::bigquery::storage::v1::read_rows_response::{Rows, Schema};
use google_cloud_googleapis::cloud::bigquery::storage::v1::{
    ArrowSchema, ReadRowsRequest, ReadRowsResponse, ReadSession,
};
use std::collections::VecDeque;
use std::io::{BufReader, Cursor};
use arrow::array::ArrayRef;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    GRPC(#[from] Status),
    #[error(transparent)]
    ArrowNative(#[from] ArrowError),
    #[error(transparent)]
    ArrowConvert(#[from] crate::arrow::Error),
    #[error("data format must be arrow")]
    InvalidDateFormat,
    #[error("schema format must be arrow")]
    InvalidSchemaFormat,
    #[error("no schema found in first response")]
    NoSchemaFound,
}

pub struct Iterator<T>
where
    T: ArrowStructDecodable<T> + Default,
{
    client: StreamingReadClient,
    session: ReadSession,
    retry: Option<RetrySetting>,
    // mutable
    stream_index: usize,
    current_stream: Streaming<ReadRowsResponse>,
    chunk: VecDeque<T>,
    schema: Option<ArrowSchema>,
}

impl<T> Iterator<T>
where
    T: ArrowStructDecodable<T> + Default,
{
    pub async fn new(
        mut client: StreamingReadClient,
        session: ReadSession,
        retry: Option<RetrySetting>,
    ) -> Result<Self, Error> {
        let current_stream = client
            .read_rows(
                ReadRowsRequest {
                    read_stream: session.streams[0].name.to_string(),
                    offset: 0,
                },
                retry.clone(),
            )
            .await?
            .into_inner();
        Ok(Self {
            client,
            session,
            retry,
            current_stream,
            stream_index: 0,
            chunk: VecDeque::new(),
            schema: None,
        })
    }

    pub async fn next(&mut self) -> Result<Option<T>, Error> {
        loop {
            if let Some(row) = self.chunk.pop_front() {
                return Ok(Some(row));
            }
            if let Some(rows) = self.current_stream.message().await? {
                // Only first response contain schema information
                let schema = match &self.schema {
                    None => match rows.schema.ok_or(Error::NoSchemaFound)? {
                        Schema::ArrowSchema(schema) => schema,
                        _ => return Err(Error::InvalidSchemaFormat),
                    },
                    Some(schema) => schema.clone(),
                };
                if let Some(rows) = rows.rows {
                    self.chunk = rows_to_chunk(schema, rows)?;
                    return Ok(self.chunk.pop_front());
                }
            }

            if self.stream_index == self.session.streams.len() - 1 {
                return Ok(None);
            } else {
                self.stream_index += 1
            }
            let stream = &self.session.streams[self.stream_index].name;
            self.current_stream = self
                .client
                .read_rows(
                    ReadRowsRequest {
                        read_stream: stream.to_string(),
                        offset: 0,
                    },
                    self.retry.clone(),
                )
                .await?
                .into_inner();
        }
    }
}

fn rows_to_chunk<T>(schema: ArrowSchema, rows: Rows) -> Result<VecDeque<T>, Error>
where
    T: ArrowStructDecodable<T> + Default,
{
    match rows {
        Rows::ArrowRecordBatch(rows) => {
            let mut rows_with_schema = schema.serialized_schema;
            rows_with_schema.extend_from_slice(&rows.serialized_record_batch);
            let rows = Cursor::new(rows_with_schema);
            let rows: StreamReader<BufReader<Cursor<Vec<u8>>>> = StreamReader::try_new(rows, None)?;
            let mut chunk: VecDeque<T> = VecDeque::new();
            for row in rows {
                let row = row?;
                for row_no in 0..row.num_rows() {
                    chunk.push_back(T::decode(row.columns(), row_no)?)
                }
            }
            Ok(chunk)
        }
        _ => Err(Error::InvalidDateFormat),
    }
}

pub mod row {
    use arrow::array::{Array, ArrayRef};
    use crate::arrow::{ArrowDecodable, ArrowStructDecodable};

    #[derive(thiserror::Error, Debug)]
    pub enum Error {
        #[error("UnexpectedColumnIndex: {0}")]
        UnexpectedColumnIndex(usize),
        #[error(transparent)]
        ArrowError(#[from] crate::arrow::Error)
    }

    pub struct Row {
        fields: Vec<ArrayRef>,
        row_no: usize,
    }

    impl ArrowStructDecodable<Row> for Row {
        fn decode(fields: &[ArrayRef], row_no: usize) -> Result<Row, crate::arrow::Error> {
            Ok(Self {
                fields: fields.to_vec(),
                row_no
            })
        }
    }

    impl Row {
        pub fn column<T: ArrowDecodable<T>>(&self, index: usize) -> Result<T, Error>{
            let column = self.fields.get(index).ok_or(Error::UnexpectedColumnIndex(index))?;
            Ok(T::decode(column, self.row_no)?)
        }
    }
}