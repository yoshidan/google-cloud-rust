use crate::arrow::{ArrowDecodable, ArrowStructDecodable};
use crate::grpc::apiv1::bigquery_client::StreamingReadClient;
use crate::http::bigquery_job_client::BigqueryJobClient;
use crate::http::error::Error as HttpError;
use crate::http::job::get_query_results::GetQueryResultsRequest;
use crate::http::tabledata::list::Tuple;
use arrow::error::ArrowError;
use arrow::ipc::reader::StreamReader;
use async_trait::async_trait;
use google_cloud_gax::grpc::{Status, Streaming};
use google_cloud_googleapis::cloud::bigquery::storage::v1::read_rows_response::{Rows, Schema};
use google_cloud_googleapis::cloud::bigquery::storage::v1::{
    ArrowSchema, ReadRowsRequest, ReadRowsResponse, ReadSession, ReadStream,
};
use std::collections::VecDeque;
use std::io::{BufReader, Cursor};

#[derive(thiserror::Error, Debug)]
pub enum QueryError {
    #[error(transparent)]
    Http(#[from] HttpError),
    #[error("invalid type {0}")]
    Decode(String),
}

pub struct QueryIterator {
    pub(crate) client: BigqueryJobClient,
    pub(crate) project_id: String,
    pub(crate) job_id: String,
    pub(crate) request: GetQueryResultsRequest,
    pub(crate) chunk: VecDeque<Tuple>,
    pub total_size: i64,
}

impl QueryIterator {
    pub async fn next<T: TryFrom<Tuple, Error = String>>(&mut self) -> Result<Option<T>, QueryError> {
        loop {
            if let Some(v) = self.chunk.pop_front() {
                return T::try_from(v).map(Some).map_err(QueryError::Decode);
            }
            if self.request.page_token.is_none() {
                return Ok(None);
            }
            let response = self
                .client
                .get_query_results(self.project_id.as_str(), self.job_id.as_str(), &self.request)
                .await?;
            if response.rows.is_none() {
                return Ok(None);
            }
            let v = response.rows.unwrap();
            self.chunk = VecDeque::from(v);
            self.request.page_token = response.page_token;
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum TableDataError {
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

pub struct TableDataIterator<T>
where
    T: ArrowStructDecodable<T> + Default,
{
    client: StreamingReadClient,
    session: ReadSession,
    // mutable
    stream_index: usize,
    current_stream: Streaming<ReadRowsResponse>,
    chunk: VecDeque<T>,
    schema: Option<ArrowSchema>,
}

impl<T> TableDataIterator<T>
where
    T: ArrowStructDecodable<T> + Default,
{
    pub async fn new(mut client: StreamingReadClient, session: ReadSession) -> Result<Self, TableDataError> {
        let current_stream = client
            .read_rows(
                ReadRowsRequest {
                    read_stream: session.streams[0].name.to_string(),
                    offset: 0,
                },
                None,
            )
            .await?
            .into_inner();
        Ok(Self {
            client,
            session,
            current_stream,
            stream_index: 0,
            chunk: VecDeque::new(),
            schema: None,
        })
    }

    pub async fn next(&mut self) -> Result<Option<T>, TableDataError> {
        loop {
            if let Some(row) = self.chunk.pop_front() {
                return Ok(Some(row));
            }
            if let Some(rows) = self.current_stream.message().await? {
                // Only first response contain schema information
                let schema = match &self.schema {
                    None => match rows.schema.ok_or(TableDataError::NoSchemaFound)? {
                        Schema::ArrowSchema(schema) => schema,
                        _ => return Err(TableDataError::InvalidSchemaFormat),
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
                    None,
                )
                .await?
                .into_inner();
        }
    }
}

fn rows_to_chunk<T>(schema: ArrowSchema, rows: Rows) -> Result<VecDeque<T>, TableDataError>
where
    T: ArrowStructDecodable<T> + Default,
{
    match rows {
        Rows::ArrowRecordBatch(rows) => {
            let mut rows_with_schema = schema.serialized_schema;
            rows_with_schema.extend_from_slice(&rows.serialized_record_batch);
            let rows = Cursor::new(rows_with_schema);
            let mut rows: StreamReader<BufReader<Cursor<Vec<u8>>>> = StreamReader::try_new(rows, None)?;
            let mut chunk: VecDeque<T> = VecDeque::new();
            while let Some(row) = rows.next() {
                let row = row?;
                for row_no in 0..row.num_rows() {
                    chunk.push_back(T::decode(row.columns(), row_no)?)
                }
            }
            Ok(chunk)
        }
        _ => Err(TableDataError::InvalidDateFormat),
    }
}
