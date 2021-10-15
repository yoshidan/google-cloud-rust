use crate::apiv1::spanner_client::Client;
use crate::row::{Row, TryFromValue};
use crate::session_pool::SessionHandle;
use async_trait::async_trait;
use chrono::{FixedOffset, NaiveDate, NaiveDateTime, NaiveTime};
use google_cloud_gax::call_option::CallSettings;
use google_cloud_googleapis::spanner::v1::spanner_client::SpannerClient;
use google_cloud_googleapis::spanner::v1::struct_type::Field;
use google_cloud_googleapis::spanner::v1::{
    result_set_stats::RowCount, ExecuteSqlRequest, PartialResultSet, ReadRequest,
    ResultSetMetadata, Session,
};
use parking_lot::Mutex;
use prost::encoding::message::merge;
use prost_types::field_descriptor_proto::Type::Uint32;
use prost_types::value::Kind::ListValue;
use prost_types::value::Kind::StringValue;
use prost_types::{value, value::Kind, Type, Value};
use std::collections::{HashMap, VecDeque};
use std::convert::TryFrom;
use std::future::Future;
use std::num::ParseIntError;
use std::panic::resume_unwind;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;
use tonic::{Code, Response, Status, Streaming};

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
    pub call_setting: Option<CallSettings>,
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
    pub call_setting: Option<CallSettings>,
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

    /// merge tries to combine two protobuf Values if possible.
    fn merge(chunked: Value, first: Value) -> Value {
        return match chunked.kind.unwrap() {
            Kind::StringValue(last) => match first.kind.unwrap() {
                Kind::StringValue(first) => {
                    println!("prev={}, next={}", &last, &first);
                    let merged = last.to_owned() + &first;
                    println!("merged={}", &merged);
                    Value {
                        kind: Some(Kind::StringValue(merged)),
                    }
                }
                _ => panic!("mismatch"),
            },
            Kind::ListValue(mut last) => match first.kind.unwrap() {
                Kind::ListValue(first) => {
                    println!("list value chunk found");
                    let mut next_list = VecDeque::from(first.values);
                    let last_value_of_chunked_list = last.values.pop().unwrap();
                    let first_value_of_next_list = next_list.pop_front().unwrap();
                    let merged =
                        RowIterator::merge(last_value_of_chunked_list, first_value_of_next_list);
                    let mut merged_values = vec![];
                    for i in last.values {
                        merged_values.push(i)
                    }
                    merged_values.push(merged);
                    while !next_list.is_empty() {
                        merged_values.push(next_list.pop_front().unwrap());
                    }
                    last.values = merged_values;
                    Value {
                        kind: Some(Kind::ListValue(last)),
                    }
                }
                _ => panic!("mismatch"),
            },
            _ => panic!("unsupported"),
        };
    }

    fn values_to_rows(
        &mut self,
        mut values: VecDeque<Value>,
        chunked_value_found: bool,
    ) -> VecDeque<Row> {
        //チャンクが残ってた場合はマージ
        match self.chunked_value.clone() {
            Some(chunked) => {
                let merged = RowIterator::merge(chunked, values.pop_front().unwrap());
                values.push_front(merged);
                self.chunked_value = None;
            }
            None => {}
        }

        //未処理レコードの残を先頭に追加して処理対象にする
        while !self.chunked_record.is_empty() {
            values.push_front(self.chunked_record.pop().unwrap());
        }

        let column_count = self.fields.len();
        let chunked_record_found = values.len() % column_count > 0;
        let record_count = values.len() / column_count;

        println!(
            "datasize={}, column={}, records={}, chunked_record={}, chunked_value={}",
            values.len(),
            column_count,
            record_count,
            chunked_record_found,
            chunked_value_found
        );
        let mut rows: VecDeque<Row> = VecDeque::new();

        while !values.is_empty() {
            // レコードが不足してる場合 -> 最終行はチャンク行き
            // データ自体がチャンクしてる場合 -> 最終行はチャンク行き
            if (chunked_record_found && rows.len() == record_count)
                || (chunked_value_found && !chunked_record_found && rows.len() == record_count - 1)
            {
                self.chunked_record.push(values.pop_front().unwrap());
            } else {
                let mut record: Vec<Value> = vec![];
                for i in 0..column_count {
                    if values.is_empty() {
                        println!("illegal data {}, {}", rows.len(), i);
                        panic!("error");
                    }
                    record.push(values.pop_front().unwrap());
                }
                rows.push_back(Row::new(
                    Arc::clone(&self.index),
                    Arc::clone(&self.fields),
                    record,
                ));
            }
        }

        if chunked_value_found && !self.chunked_record.is_empty() {
            self.chunked_value = Some(self.chunked_record.pop().unwrap());
        }

        return rows;
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
        if !self.rows.is_empty() {
            return Ok(self.rows.pop_front());
        }

        let result_set_option = match self.streaming.message().await {
            Ok(s) => s,
            Err(e) => {
                println!("{}", "streaming message error");
                if !self.reader.can_retry() {
                    println!("{}", "resumes");
                    return Err(e);
                }

                let result = match self.reader.read(&mut self.session).await {
                    Ok(s) => s,
                    Err(e) => {
                        println!("{}", "streaming error");
                        return Err(e);
                    }
                };
                self.streaming = result.into_inner();
                self.streaming.message().await?
            }
        };

        let result_set = match result_set_option {
            Some(s) => s,
            None => {
                self.rows = VecDeque::new();
                return Ok(None);
            }
        };
        if result_set.values.is_empty() {
            return Ok(None);
        }

        //初回のみメタデータ設定
        if result_set.metadata.is_some() && self.fields.len() == 0 {
            // metadata can be found only first call
            self.fields = Arc::new(result_set.metadata.unwrap().row_type.unwrap().fields);
            let mut index = HashMap::new();
            for (i, f) in self.fields.iter().enumerate() {
                index.insert(f.name.clone(), i);
            }
            self.index = Arc::new(index);
        }
        if !result_set.resume_token.is_empty() {
            self.reader.update_token(result_set.resume_token);
        }
        self.rows =
            self.values_to_rows(VecDeque::from(result_set.values), result_set.chunked_value);
        return self.next().await;
    }
}
