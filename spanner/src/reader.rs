use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use async_trait::async_trait;
use prost_types::{value::Kind, Value};

use google_cloud_gax::grpc::{Code, Response, Status, Streaming};
use google_cloud_googleapis::spanner::v1::struct_type::Field;
use google_cloud_googleapis::spanner::v1::{ExecuteSqlRequest, PartialResultSet, ReadRequest, ResultSetMetadata};

use crate::row::Row;
use crate::session::SessionHandle;
use crate::transaction::CallOptions;

#[async_trait]
pub trait AsyncIterator {
    fn column_metadata(&self, column_name: &str) -> Option<(usize, Field)>;
    async fn next(&mut self) -> Result<Option<Row>, Status>;
}

#[async_trait]
pub trait Reader {
    async fn read(
        &self,
        session: &mut SessionHandle,
        option: Option<CallOptions>,
    ) -> Result<Response<Streaming<PartialResultSet>>, Status>;

    fn update_token(&mut self, resume_token: Vec<u8>);

    fn can_retry(&self) -> bool;
}

pub struct StatementReader {
    pub request: ExecuteSqlRequest,
}

#[async_trait]
impl Reader for StatementReader {
    async fn read(
        &self,
        session: &mut SessionHandle,
        option: Option<CallOptions>,
    ) -> Result<Response<Streaming<PartialResultSet>>, Status> {
        let option = option.unwrap_or_default();
        let client = &mut session.spanner_client;
        let result = client
            .execute_streaming_sql(self.request.clone(), option.cancel, option.retry)
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
}

#[async_trait]
impl Reader for TableReader {
    async fn read(
        &self,
        session: &mut SessionHandle,
        option: Option<CallOptions>,
    ) -> Result<Response<Streaming<PartialResultSet>>, Status> {
        let option = option.unwrap_or_default();
        let client = &mut session.spanner_client;
        let result = client
            .streaming_read(self.request.clone(), option.cancel, option.retry)
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

pub struct Chunk {
    fields: Arc<Vec<Field>>,
    index: Arc<HashMap<String, usize>>,
    rows: VecDeque<Value>,
    chunked_value: bool,
}

impl Chunk {
    fn next(&mut self) -> Option<Row> {
        if !self.rows.is_empty() {
            let column_length = self.fields.len();
            let target_record_is_chunked = self.rows.len() < column_length;
            let target_record_contains_chunked_value = self.chunked_value && self.rows.len() == column_length;

            if !target_record_is_chunked && !target_record_contains_chunked_value {
                // get column_length values
                let mut values = Vec::with_capacity(column_length);
                for _ in 0..column_length {
                    values.push(self.rows.pop_front().unwrap());
                }
                return Some(Row::new(Arc::clone(&self.index), Arc::clone(&self.fields), values));
            }
        }
        return None;
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
                    let merged = Chunk::merge(last_value_of_previous, first_value_of_next)?;
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

    fn add(
        &mut self,
        metadata: Option<ResultSetMetadata>,
        mut values: Vec<Value>,
        chunked_value: bool,
    ) -> Result<bool, Status> {
        // get metadata only once.
        if self.fields.is_empty() && metadata.is_some() {
            let metadata = metadata.unwrap();
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

        if self.chunked_value {
            //merge when the chunked value is found.
            let first = values.remove(0);
            let merged = Chunk::merge(self.rows.pop_back().unwrap(), first)?;
            self.rows.push_back(merged);
        }
        self.rows.extend(values);
        self.chunked_value = chunked_value;
        Ok(true)
    }
}

pub struct RowIterator<'a> {
    streaming: Streaming<PartialResultSet>,
    session: &'a mut SessionHandle,
    reader: Box<dyn Reader + Sync + Send>,
    chunk: Chunk,
    reader_option: Option<CallOptions>,
}

impl<'a> RowIterator<'a> {
    pub(crate) async fn new(
        session: &'a mut SessionHandle,
        reader: Box<dyn Reader + Sync + Send>,
        option: Option<CallOptions>,
    ) -> Result<RowIterator<'a>, Status> {
        let streaming = reader.read(session, option).await?.into_inner();
        let chunk = Chunk {
            fields: Arc::new(vec![]),
            index: Arc::new(HashMap::new()),
            rows: VecDeque::new(),
            chunked_value: false,
        };
        Ok(Self {
            streaming,
            session,
            reader,
            chunk,
            reader_option: None,
        })
    }

    pub fn set_call_options(&mut self, option: CallOptions) {
        self.reader_option = Some(option);
    }

    async fn try_recv(&mut self, option: Option<CallOptions>) -> Result<bool, Status> {
        // try getting records from server
        let maybe_result_set = match self.streaming.message().await {
            Ok(s) => s,
            Err(e) => {
                if !self.reader.can_retry() {
                    return Err(e.into());
                }
                log::debug!("streaming error: {}. resume reading by resume_token", e);
                let result = self.reader.read(&mut self.session, option).await?;
                self.streaming = result.into_inner();
                self.streaming.message().await?
            }
        };

        return match maybe_result_set {
            Some(result_set) => {
                if result_set.values.is_empty() {
                    return Ok(false);
                }
                //if resume_token changes set new resume_token
                if !result_set.resume_token.is_empty() {
                    self.reader.update_token(result_set.resume_token);
                }
                self.chunk
                    .add(result_set.metadata, result_set.values, result_set.chunked_value)
            }
            None => Ok(false),
        };
    }
}

#[async_trait]
impl<'a> AsyncIterator for RowIterator<'a> {
    fn column_metadata(&self, column_name: &str) -> Option<(usize, Field)> {
        for (i, val) in self.chunk.fields.iter().enumerate() {
            if val.name == column_name {
                return Some((i, val.clone()));
            }
        }
        None
    }

    /// next returns the next result.
    /// Its second return value is None if there are no more results.
    async fn next(&mut self) -> Result<Option<Row>, Status> {
        let row = self.chunk.next();
        if row.is_some() {
            return Ok(row);
        }
        // no data found or record chunked.
        if !self.try_recv(self.reader_option.clone()).await? {
            return Ok(None);
        }
        return self.next().await;
    }
}

#[cfg(test)]
mod tests {
    use crate::reader::Chunk;
    use crate::statement::ToKind;
    use google_cloud_googleapis::spanner::v1::struct_type::Field;
    use prost_types::value::Kind;
    use prost_types::Value;
    use std::collections::VecDeque;
    use std::sync::Arc;

    #[test]
    fn test_chunk_next_empty() {
        let mut chunk = Chunk {
            fields: Arc::new(vec![Field {
                name: "column1".to_string(),
                r#type: None,
            }]),
            index: Arc::new(Default::default()),
            rows: Default::default(),
            chunked_value: false,
        };
        assert!(chunk.next().is_none());
    }

    #[test]
    fn test_chunk_next_record_chunked_or_not() {
        let mut chunk = |values| Chunk {
            fields: Arc::new(vec![
                Field {
                    name: "column1".to_string(),
                    r#type: None,
                },
                Field {
                    name: "column2".to_string(),
                    r#type: None,
                },
            ]),
            index: Arc::new(Default::default()),
            rows: VecDeque::from(values),
            chunked_value: false,
        };
        assert!(chunk(vec![Value {
            kind: Some("value1".to_kind())
        }])
        .next()
        .is_none());
        assert_eq!(
            chunk(vec![
                Value {
                    kind: Some("value1".to_kind())
                },
                Value {
                    kind: Some("value2".to_kind())
                }
            ])
            .next()
            .unwrap()
            .column::<String>(0)
            .unwrap(),
            "value1".to_string()
        );
    }

    #[test]
    fn test_chunk_next_value_chunked_or_not() {
        let chunk = |chunked_value| Chunk {
            fields: Arc::new(vec![
                Field {
                    name: "column1".to_string(),
                    r#type: None,
                },
                Field {
                    name: "column2".to_string(),
                    r#type: None,
                },
            ]),
            index: Arc::new(Default::default()),
            rows: VecDeque::from(vec![
                Value {
                    kind: Some("value1".to_kind()),
                },
                Value {
                    kind: Some("value2".to_kind()),
                },
            ]),
            chunked_value,
        };
        assert!(chunk(true).next().is_none());
        assert_eq!(chunk(false).next().unwrap().column::<String>(0).unwrap(), "value1".to_string());
    }

    #[test]
    fn test_chunk_next_plural_record_one_column() {
        let chunk = |chunked_value| Chunk {
            fields: Arc::new(vec![Field {
                name: "column1".to_string(),
                r#type: None,
            }]),
            index: Arc::new(Default::default()),
            rows: VecDeque::from(vec![
                Value {
                    kind: Some("value1".to_kind()),
                },
                Value {
                    kind: Some("value2".to_kind()),
                },
                Value {
                    kind: Some("value3".to_kind()),
                },
            ]),
            chunked_value,
        };
        let mut incomplete = chunk(true);
        assert!(incomplete.next().is_some());
        assert!(incomplete.next().is_some());
        assert!(incomplete.next().is_none());
        let mut complete = chunk(false);
        assert!(complete.next().is_some());
        assert!(complete.next().is_some());
        assert!(complete.next().is_some());
        assert!(complete.next().is_none());
    }

    #[test]
    fn test_chunk_next_plural_record_multi_column() {
        let chunk = |chunked_value| Chunk {
            fields: Arc::new(vec![
                Field {
                    name: "column1".to_string(),
                    r#type: None,
                },
                Field {
                    name: "column2".to_string(),
                    r#type: None,
                },
            ]),
            index: Arc::new(Default::default()),
            rows: VecDeque::from(vec![
                Value {
                    kind: Some("value1".to_kind()),
                },
                Value {
                    kind: Some("value2".to_kind()),
                },
                Value {
                    kind: Some("value3".to_kind()),
                },
            ]),
            chunked_value,
        };
        let mut incomplete = chunk(true);
        assert_eq!(incomplete.next().unwrap().column::<String>(1).unwrap(), "value2".to_string());
        assert!(incomplete.next().is_none());
        let mut complete = chunk(false);
        assert_eq!(complete.next().unwrap().column::<String>(1).unwrap(), "value2".to_string());
        assert!(incomplete.next().is_none());
    }
}
