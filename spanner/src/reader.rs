use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use async_trait::async_trait;
use prost_types::{value::Kind, Value, field};
use tonic::{Response, Status, Streaming, Code};

use google_cloud_gax::call_option::BackoffRetrySettings;
use google_cloud_googleapis::spanner::v1::struct_type::Field;
use google_cloud_googleapis::spanner::v1::{ExecuteSqlRequest, PartialResultSet, ReadRequest};

use crate::row::Row;
use crate::session_pool::SessionHandle;

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
        return !self.request.resume_token.is_empty();
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
        return !self.request.resume_token.is_empty();
    }
}

pub struct RowIterator<'a> {
    streaming: Streaming<PartialResultSet>,
    session: &'a mut SessionHandle,
    reader: Box<dyn Reader + Sync + Send>,
    fields: Arc<Vec<Field>>,
    index: Arc<HashMap<String, usize>>,
    rows: VecDeque<Row>,
    chunked_value: Option<Value>,
    chunked_record: Vec<Value>,
}

impl<'a> RowIterator<'a> {
    pub(crate) async fn new(
        session: &'a mut SessionHandle,
        reader: Box<dyn Reader + Sync + Send>,
    ) -> Result<RowIterator<'a>, Status> {
        let streaming = reader.read(session).await?.into_inner();
        return Ok(RowIterator {
            streaming,
            session,
            fields: Arc::new(vec![]),
            index: Arc::new(HashMap::new()),
            reader,
            rows: VecDeque::new(),
            chunked_value: None,
            chunked_record: vec![],
        });
    }

    /// Merge tries to combine two protobuf Values if possible.
    fn merge(previous_last: Value, current_first: Value) -> Result<Value, Status> {
        return match previous_last.kind.unwrap() {
            Kind::StringValue(last) => match current_first.kind.unwrap() {
                Kind::StringValue(first) => {
                    log::trace!("previous_last={}, current_first={}", &last, &first);
                    let merged = last + &first;
                    Ok(Value {
                        kind: Some(Kind::StringValue(merged)),
                    })
                }
                _ => return Err(Status::new(Code::Internal, "chunks kind mismatch: current_first must be StringKind")),
            },
            Kind::ListValue(mut last) => match current_first.kind.unwrap() {
                Kind::ListValue(first) => {
                    let mut next_list = VecDeque::from(first.values);
                    let last_value_of_previous = last.values.pop().unwrap();
                    let first_value_of_next = next_list.pop_front().unwrap();
                    let merged_value =
                        RowIterator::merge(last_value_of_previous, first_value_of_next)?;

                    let mut merged_values = vec![];

                    for i in last.values {
                        merged_values.push(i)
                    }
                    merged_values.push(merged_value);

                    while let Some(value) = next_list.pop_front() {
                        merged_values.push(value);
                    }

                    last.values = merged_values;
                    Ok(Value {
                        kind: Some(Kind::ListValue(last)),
                    })
                }
                _ => return Err(Status::new(Code::Internal, "chunks kind mismatch: current_first must be ListValue")),
            },
            _ => return Err(Status::new(Code::Internal, "previous_last kind mismatch: only StringValue and ListValue can be chunked")),
        };
    }

    /// Format PartialResultSet::values and process it into a format that expresses one line of RDB.
    ///
    /// The PartialResultSet::values returned from the server does not represent one line of data,
    /// and may contain multiple lines or the data may be cut off in the middle of the line.
    fn values_to_rows(
        &mut self,
        mut values: VecDeque<Value>,
        chunked_value_found: bool,
    ) -> Result<VecDeque<Row>, Status> {

        //未マージのデータをサーバから返却されたデータの先頭とマージする。
        if let Some(chunked_value) =  self.chunked_value.take() {
            let merged = RowIterator::merge(chunked_value, values.pop_front().unwrap())?;
            values.push_front(merged);
        }

        //チャンクされたレコードの残りは処理対象のデータの先頭に突っ込む
        while let Some(value) = self.chunked_record.pop() {
            values.push_front(value);
        }

        let column_count = self.fields.len();
        let chunked_record_found = values.len() % column_count > 0;
        let expected_total_record_count = values.len() / column_count;

        if chunked_value_found || chunked_record_found {
            println!(
                "datasize={}, column={}, records={}, chunked_record={}, chunked_value={}",
                values.len(),
                column_count,
                expected_total_record_count,
                chunked_record_found,
                chunked_value_found
            );
        }

        let mut rows: VecDeque<Row> = VecDeque::new();
        while !values.is_empty() {
            // レコードが全カラム分のデータを含んでいない場合は、次のfetchで後続のカラムが取得できるのでその行をChunkedRecordとして扱う。
            if (chunked_record_found && rows.len() == expected_total_record_count)
                // レコードではなく、そもそも各カラムのデータがChunkの場合にもその行のデータをChunkedRecordとして扱う。
                || (chunked_value_found && !chunked_record_found && rows.len() == expected_total_record_count - 1)
            {
                self.chunked_record.push(values.pop_front().unwrap());
            } else {
                // 1行のデータを作る。
                let mut row: Vec<Value> = vec![];
                for  _ in 0..column_count {
                    row.push(values.pop_front().unwrap());
                }
                rows.push_back(Row::new(
                    Arc::clone(&self.index),
                    Arc::clone(&self.fields),
                    row,
                ));
            }
        }

        // カラムがチャンクしてる場合は次のループでマージが必要となるので、マージ対象として保持。
        if chunked_value_found && !self.chunked_record.is_empty() {
            self.chunked_value = Some(self.chunked_record.pop().unwrap());
        }

        return Ok(rows);
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
        return None;
    }

    /// next returns the next result.
    /// Its second return value is None if there are no more results.
    async fn next(&mut self) -> Result<Option<Row>, tonic::Status> {

        // get next data
        if !self.rows.is_empty() {
            return Ok(self.rows.pop_front());
        }

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

        let result_set = match result_set_option {
            Some(s) => s,
            None => return Ok(None)
        };

        if result_set.values.is_empty() {
            return Ok(None);
        }

        // get metadata only once.
        if self.fields.is_empty() && result_set.metadata.is_some(){
            let metadata = result_set.metadata.unwrap();
            self.fields = match metadata.row_type {
                Some(row_type) => Arc::new(row_type.fields),
                None => return Err(Status::new(Code::Internal, "no field metadata found {}"))
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

        self.rows =
            self.values_to_rows(VecDeque::from(result_set.values), result_set.chunked_value)?;
        return self.next().await;
    }
}
