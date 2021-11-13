use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use async_trait::async_trait;
use prost_types::{value::Kind, Value};
use tonic::{Code, Response, Status, Streaming};

use google_cloud_gax::call_option::BackoffRetrySettings;
use google_cloud_googleapis::spanner::v1::struct_type::Field;
use google_cloud_googleapis::spanner::v1::{ExecuteSqlRequest, PartialResultSet, ReadRequest};

use crate::row::Row;
use crate::sessions::SessionHandle;

#[async_trait]
pub trait AsyncIterator {
    fn column_metadata(&self, column_name: &str) -> Option<(usize, Field)>;

    async fn next(&mut self) -> Result<Option<Row>, tonic::Status>;
}

#[async_trait]
pub trait Reader {
    async fn read(
        &self,
        session: &mut SessionHandle,
    ) -> Result<Response<Streaming<PartialResultSet>>, Status>;

    fn update_token(&mut self, resume_token: Vec<u8>);

    fn can_retry(&self) -> bool;
}

pub struct StatementReader {
    pub request: ExecuteSqlRequest,
    pub call_setting: Option<BackoffRetrySettings>,
}

#[async_trait]
impl Reader for StatementReader {
    async fn read(
        &self,
        session: &mut SessionHandle,
    ) -> Result<Response<Streaming<PartialResultSet>>, Status> {
        let client = &mut session.spanner_client;
        let result = client
            .execute_streaming_sql(self.request.clone(), self.call_setting.clone())
            .await;
        return session.invalidate_if_needed(result).await;
    }

    fn update_token(&mut self, resume_token: Vec<u8>) {
        self.request.resume_token = resume_token;
    }

    fn can_retry(&self) -> bool {
        !self.request.resume_token.is_empty()
    }
}

pub struct TableReader {
    pub request: ReadRequest,
    pub call_setting: Option<BackoffRetrySettings>,
}

#[async_trait]
impl Reader for TableReader {
    async fn read(
        &self,
        session: &mut SessionHandle,
    ) -> Result<Response<Streaming<PartialResultSet>>, Status> {
        let client = &mut session.spanner_client;
        let result = client
            .streaming_read(self.request.clone(), self.call_setting.clone())
            .await;
        return session.invalidate_if_needed(result).await;
    }

    fn update_token(&mut self, resume_token: Vec<u8>) {
        self.request.resume_token = resume_token;
    }

    fn can_retry(&self) -> bool {
        !self.request.resume_token.is_empty()
    }
}

pub struct RowIterator<'a> {
    streaming: Streaming<PartialResultSet>,
    session: &'a mut SessionHandle,
    reader: Box<dyn Reader + Sync + Send>,
    fields: Arc<Vec<Field>>,
    index: Arc<HashMap<String, usize>>,
    rows: VecDeque<Value>,
    chunked_value: bool,
    chunked_record: bool,
}

impl<'a> RowIterator<'a> {
    pub(crate) async fn new(
        session: &'a mut SessionHandle,
        reader: Box<dyn Reader + Sync + Send>,
    ) -> Result<RowIterator<'a>, Status> {
        let streaming = reader.read(session).await?.into_inner();
        Ok(Self {
            streaming,
            session,
            fields: Arc::new(vec![]),
            index: Arc::new(HashMap::new()),
            reader,
            rows: VecDeque::new(),
            chunked_value: false,
            chunked_record: false,
        })
    }

    /// Merge tries to combine two protobuf Values if possible.
    fn merge(previous_last: Value, current_first: Value) -> Result<Value, Status> {
        return match previous_last.kind.unwrap() {
            Kind::StringValue(last) => match current_first.kind.unwrap() {
                Kind::StringValue(first) => {
                    log::trace!("previous_last={}, current_first={}", &last, first);
                    Ok(Value {
                        kind: Some(Kind::StringValue(last + &first)),
                    })
                }
                _ => {
                    return Err(Status::new(
                        Code::Internal,
                        "chunks kind mismatch: current_first must be StringKind",
                    ))
                }
            },
            Kind::ListValue(mut last) => match current_first.kind.unwrap() {
                Kind::ListValue(mut first) => {
                    let last_value_of_previous = last.values.pop().unwrap();
                    let first_value_of_next = first.values.remove(0);
                    let merged = RowIterator::merge(last_value_of_previous, first_value_of_next)?;
                    last.values.push(merged);
                    last.values.extend(first.values);
                    Ok(Value {
                        kind: Some(Kind::ListValue(last)),
                    })
                }
                _ => {
                    return Err(Status::new(
                        Code::Internal,
                        "chunks kind mismatch: current_first must be ListValue",
                    ))
                }
            },
            _ => {
                return Err(Status::new(
                    Code::Internal,
                    "previous_last kind mismatch: only StringValue and ListValue can be chunked",
                ))
            }
        };
    }

    async fn try_recv(&mut self) -> Result<bool, tonic::Status> {
        // try getting records from server
        let result_set_option = match self.streaming.message().await {
            Ok(s) => s,
            Err(e) => {
                if !self.reader.can_retry() {
                    return Err(e);
                }
                log::debug!("streaming error: {}. resume reading by resume_token", e);
                let result = self.reader.read(&mut self.session).await?;
                self.streaming = result.into_inner();
                self.streaming.message().await?
            }
        };

        let mut result_set = match result_set_option {
            Some(s) => s,
            None => return Ok(false),
        };

        if result_set.values.is_empty() {
            return Ok(false);
        }

        // get metadata only once.
        if self.fields.is_empty() && result_set.metadata.is_some() {
            let metadata = result_set.metadata.unwrap();
            self.fields = match metadata.row_type {
                Some(row_type) => Arc::new(row_type.fields),
                None => return Err(Status::new(Code::Internal, "no field metadata found {}")),
            };
            // create index for Row::column_by_name("column_name")
            let mut index = HashMap::new();
            for (i, f) in self.fields.iter().enumerate() {
                index.insert(f.name.clone(), i);
            }
            self.index = Arc::new(index);
        }

        //if resume_token changes set new resume_token
        if !result_set.resume_token.is_empty() {
            self.reader.update_token(result_set.resume_token);
        }

        if self.chunked_value {
            //merge when the chunked value is found.
            let first = result_set.values.remove(0);
            let merged = RowIterator::merge(self.rows.pop_back().unwrap(), first)?;
            self.rows.push_back(merged);
        }
        self.rows.extend(result_set.values);
        self.chunked_record = self.rows.len() % self.fields.len() > 0;
        self.chunked_value = result_set.chunked_value;
        Ok(true)
    }
}

#[async_trait]
impl<'a> AsyncIterator for RowIterator<'a> {
    fn column_metadata(&self, column_name: &str) -> Option<(usize, Field)> {
        for (i, val) in self.fields.iter().enumerate() {
            if val.name == column_name {
                return Some((i, val.clone()));
            }
        }
        None
    }

    /// next returns the next result.
    /// Its second return value is None if there are no more results.
    async fn next(&mut self) -> Result<Option<Row>, tonic::Status> {
        if !self.rows.is_empty() {
            let column_length = self.fields.len();
            let target_record_is_chunked = self.rows.len() < column_length;
            let target_record_contains_chunked_value =
                self.chunked_value && self.rows.len() == column_length;

            if !target_record_is_chunked && !target_record_contains_chunked_value {
                // get column_length values
                let mut values = Vec::with_capacity(column_length);
                for _ in 0..column_length {
                    values.push(self.rows.pop_front().unwrap());
                }
                return Ok(Some(Row::new(
                    Arc::clone(&self.index),
                    Arc::clone(&self.fields),
                    values,
                )));
            }
        }

        // no data found or record chunked.
        if !self.try_recv().await? {
            return Ok(None);
        }
        return self.next().await;
    }
}
