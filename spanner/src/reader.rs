use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use prost::Message;
use prost_types::{value::Kind, Value};

use google_cloud_gax::grpc::{Code, Response, Status, Streaming};
use google_cloud_googleapis::spanner::v1::struct_type::Field;
use google_cloud_googleapis::spanner::v1::{
    ExecuteSqlRequest, PartialResultSet, ReadRequest, ResultSetMetadata, ResultSetStats,
};

use crate::retry::StreamingRetry;
use crate::row::Row;
use crate::session::SessionHandle;
use crate::transaction::CallOptions;

pub trait Reader: Send + Sync {
    fn read(
        &self,
        session: &mut SessionHandle,
        option: Option<CallOptions>,
        disable_route_to_leader: bool,
    ) -> impl std::future::Future<Output = Result<Response<Streaming<PartialResultSet>>, Status>> + Send;

    fn update_token(&mut self, resume_token: Vec<u8>);

    fn can_resume(&self) -> bool;
}

pub struct StatementReader {
    pub enable_resume: bool,
    pub request: ExecuteSqlRequest,
}

impl Reader for StatementReader {
    async fn read(
        &self,
        session: &mut SessionHandle,
        option: Option<CallOptions>,
        disable_route_to_leader: bool,
    ) -> Result<Response<Streaming<PartialResultSet>>, Status> {
        let option = option.unwrap_or_default();
        let client = &mut session.spanner_client;
        let result = client
            .execute_streaming_sql(self.request.clone(), disable_route_to_leader, option.retry)
            .await;
        session.invalidate_if_needed(result).await
    }

    fn update_token(&mut self, resume_token: Vec<u8>) {
        self.request.resume_token = resume_token;
    }

    fn can_resume(&self) -> bool {
        self.enable_resume && !self.request.resume_token.is_empty()
    }
}

pub struct TableReader {
    pub request: ReadRequest,
}

impl Reader for TableReader {
    async fn read(
        &self,
        session: &mut SessionHandle,
        option: Option<CallOptions>,
        disable_route_to_leader: bool,
    ) -> Result<Response<Streaming<PartialResultSet>>, Status> {
        let option = option.unwrap_or_default();
        let client = &mut session.spanner_client;
        let result = client
            .streaming_read(self.request.clone(), disable_route_to_leader, option.retry)
            .await;
        session.invalidate_if_needed(result).await
    }

    fn update_token(&mut self, resume_token: Vec<u8>) {
        self.request.resume_token = resume_token;
    }

    fn can_resume(&self) -> bool {
        !self.request.resume_token.is_empty()
    }
}

pub struct ResultSet {
    fields: Arc<Vec<Field>>,
    index: Arc<HashMap<String, usize>>,
    rows: VecDeque<Value>,
    chunked_value: bool,
}

const DEFAULT_MAX_BYTES_BETWEEN_RESUME_TOKENS: usize = 128 * 1024 * 1024;

#[derive(Debug)]
struct ResumablePartialResultSetBuffer {
    pending: VecDeque<PartialResultSet>,
    last_delivered_token: Vec<u8>,
    observed_token: Vec<u8>,
    bytes_between_tokens: usize,
    max_bytes_between_tokens: usize,
    unretryable: bool,
}

impl ResumablePartialResultSetBuffer {
    fn new(max_bytes_between_tokens: usize) -> Self {
        Self {
            pending: VecDeque::new(),
            last_delivered_token: Vec::new(),
            observed_token: Vec::new(),
            bytes_between_tokens: 0,
            max_bytes_between_tokens,
            unretryable: false,
        }
    }

    fn push(&mut self, result_set: PartialResultSet) {
        if !result_set.resume_token.is_empty() && result_set.resume_token != self.observed_token {
            self.observed_token = result_set.resume_token.clone();
        }

        if !self.unretryable && self.observed_token == self.last_delivered_token {
            self.bytes_between_tokens = self.bytes_between_tokens.saturating_add(result_set.encoded_len());
            if self.bytes_between_tokens >= self.max_bytes_between_tokens {
                self.unretryable = true;
            }
        }

        self.pending.push_back(result_set);
    }

    fn pop_ready(&mut self, end_of_stream: bool) -> Option<PartialResultSet> {
        if self.pending.is_empty() {
            return None;
        }

        if self.unretryable || end_of_stream {
            return self.pending.pop_front();
        }

        if self.observed_token != self.last_delivered_token {
            let result_set = self.pending.pop_front();
            if let Some(ref rs) = result_set {
                if !rs.resume_token.is_empty() && rs.resume_token == self.observed_token {
                    self.last_delivered_token = self.observed_token.clone();
                    self.bytes_between_tokens = 0;
                }
            }
            return result_set;
        }

        None
    }

    fn on_resumption(&mut self) {
        self.pending.clear();
        self.observed_token = self.last_delivered_token.clone();
        self.bytes_between_tokens = 0;
        self.unretryable = false;
    }
}

impl ResultSet {
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
        None
    }

    /// Merge tries to combine two protobuf Values if possible.
    fn merge(previous_last: Value, current_first: Value) -> Result<Value, Status> {
        match previous_last.kind.unwrap() {
            Kind::StringValue(last) => match current_first.kind.unwrap() {
                Kind::StringValue(first) => {
                    tracing::trace!("previous_last={}, current_first={}", &last, first);
                    Ok(Value {
                        kind: Some(Kind::StringValue(last + &first)),
                    })
                }
                _ => Err(Status::new(
                    Code::Internal,
                    "chunks kind mismatch: current_first must be StringKind",
                )),
            },
            Kind::ListValue(mut last) => match current_first.kind.unwrap() {
                Kind::ListValue(mut first) => {
                    let first_value_of_current = first.values.remove(0);
                    let merged = match last.values.pop() {
                        Some(last_value_of_previous) => {
                            ResultSet::merge(last_value_of_previous, first_value_of_current)?
                        }
                        // last record can be empty
                        None => first_value_of_current,
                    };
                    last.values.push(merged);
                    last.values.extend(first.values);
                    Ok(Value {
                        kind: Some(Kind::ListValue(last)),
                    })
                }
                _ => Err(Status::new(
                    Code::Internal,
                    "chunks kind mismatch: current_first must be ListValue",
                )),
            },
            _ => Err(Status::new(
                Code::Internal,
                "previous_last kind mismatch: only StringValue and ListValue can be chunked",
            )),
        }
    }

    fn add(
        &mut self,
        metadata: Option<ResultSetMetadata>,
        mut values: Vec<Value>,
        chunked_value: bool,
    ) -> Result<bool, Status> {
        // get metadata only once.
        if self.fields.is_empty() {
            if let Some(metadata) = metadata {
                self.fields = metadata
                    .row_type
                    .map(|e| Arc::new(e.fields))
                    .ok_or_else(|| Status::new(Code::Internal, "no field metadata found"))?;
                // create index for Row::column_by_name("column_name")
                let mut index = HashMap::new();
                for (i, f) in self.fields.iter().enumerate() {
                    index.insert(f.name.clone(), i);
                }
                self.index = Arc::new(index);
            }
        }

        if self.chunked_value {
            tracing::trace!("now chunked value found previous={}, current={}", self.rows.len(), values.len());
            //merge when the chunked value is found.
            let merged = ResultSet::merge(self.rows.pop_back().unwrap(), values.remove(0))?;
            self.rows.push_back(merged);
        }
        self.rows.extend(values);
        self.chunked_value = chunked_value;
        Ok(true)
    }

    fn is_row_boundary(&self) -> bool {
        if self.fields.is_empty() {
            return self.rows.is_empty() && !self.chunked_value;
        }
        if self.chunked_value {
            return false;
        }
        let columns = self.fields.len();
        if columns == 0 {
            return self.rows.is_empty();
        }
        self.rows.len().is_multiple_of(columns)
    }
}

pub struct RowIterator<'a, T>
where
    T: Reader,
{
    streaming: Streaming<PartialResultSet>,
    session: &'a mut SessionHandle,
    reader: T,
    rs: ResultSet,
    reader_option: Option<CallOptions>,
    disable_route_to_leader: bool,
    stats: Option<ResultSetStats>,
    prs_buffer: ResumablePartialResultSetBuffer,
    resumable: bool,
    end_of_stream: bool,
    stream_retry: StreamingRetry,
}

impl<'a, T> RowIterator<'a, T>
where
    T: Reader,
{
    pub(crate) async fn new(
        session: &'a mut SessionHandle,
        reader: T,
        option: Option<CallOptions>,
        disable_route_to_leader: bool,
    ) -> Result<RowIterator<'a, T>, Status> {
        let streaming = reader
            .read(session, option, disable_route_to_leader)
            .await?
            .into_inner();
        let rs = ResultSet {
            fields: Arc::new(vec![]),
            index: Arc::new(HashMap::new()),
            rows: VecDeque::new(),
            chunked_value: false,
        };
        Ok(Self {
            streaming,
            session,
            reader,
            rs,
            reader_option: None,
            disable_route_to_leader,
            stats: None,
            prs_buffer: ResumablePartialResultSetBuffer::new(DEFAULT_MAX_BYTES_BETWEEN_RESUME_TOKENS),
            resumable: true,
            end_of_stream: false,
            stream_retry: StreamingRetry::new(),
        })
    }

    pub fn set_call_options(&mut self, option: CallOptions) {
        self.reader_option = Some(option);
    }

    async fn try_recv(&mut self, option: Option<CallOptions>) -> Result<bool, Status> {
        loop {
            if let Some(result_set) = self.prs_buffer.pop_ready(self.end_of_stream) {
                if result_set.values.is_empty() {
                    return Ok(false);
                }
                let resume_token_present = !result_set.resume_token.is_empty();
                //if resume_token changes set new resume_token
                if resume_token_present {
                    self.reader.update_token(result_set.resume_token.clone());
                }
                // Capture stats if present (only sent with the last response)
                if result_set.stats.is_some() {
                    self.stats = result_set.stats;
                }
                let added = self
                    .rs
                    .add(result_set.metadata, result_set.values, result_set.chunked_value)?;
                if resume_token_present && !self.rs.is_row_boundary() {
                    return Err(Status::new(Code::FailedPrecondition, "resume token is not on a row boundary"));
                }
                return Ok(added);
            }

            if self.end_of_stream {
                return Ok(false);
            }

            let received = match self.streaming.message().await {
                Ok(s) => s,
                Err(e) => {
                    if !self.reader.can_resume() || !self.resumable {
                        return Err(e);
                    }
                    tracing::debug!("streaming error: {}. resume reading by resume_token", e);
                    self.stream_retry.next(e).await?;
                    let call_option = option.clone();
                    let result = self
                        .reader
                        .read(self.session, call_option, self.disable_route_to_leader)
                        .await?;
                    self.streaming = result.into_inner();
                    self.prs_buffer.on_resumption();
                    continue;
                }
            };

            match received {
                Some(result_set) => {
                    if result_set.last {
                        self.end_of_stream = true;
                    }
                    self.prs_buffer.push(result_set);
                    if self.prs_buffer.unretryable {
                        self.resumable = false;
                    }
                }
                None => {
                    self.end_of_stream = true;
                }
            }
        }
    }

    /// Return metadata for all columns
    pub fn columns_metadata(&self) -> &Arc<Vec<Field>> {
        &self.rs.fields
    }

    pub fn column_metadata(&self, column_name: &str) -> Option<(usize, Field)> {
        for (i, val) in self.rs.fields.iter().enumerate() {
            if val.name == column_name {
                return Some((i, val.clone()));
            }
        }
        None
    }

    /// Returns query execution statistics if available.
    /// Stats are only available after all rows have been consumed and only when
    /// the query was executed with a QueryMode that includes stats (Profile, WithStats, or WithPlanAndStats).
    pub fn stats(&self) -> Option<&ResultSetStats> {
        self.stats.as_ref()
    }

    /// next returns the next result.
    /// Its second return value is None if there are no more results.
    pub async fn next(&mut self) -> Result<Option<Row>, Status> {
        loop {
            let row = self.rs.next();
            if row.is_some() {
                return Ok(row);
            }
            // no data found or record chunked.
            if !self.try_recv(self.reader_option.clone()).await? {
                return Ok(None);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::sync::Arc;

    use prost_types::value::Kind;
    use prost_types::Value;

    use google_cloud_googleapis::spanner::v1::struct_type::Field;
    use google_cloud_googleapis::spanner::v1::{PartialResultSet, ResultSetMetadata, StructType};

    use crate::reader::ResultSet;
    use crate::row::{Row, TryFromValue};
    use crate::statement::ToKind;

    fn empty_rs() -> ResultSet {
        ResultSet {
            fields: Arc::new(vec![]),
            index: Arc::new(Default::default()),
            rows: Default::default(),
            chunked_value: false,
        }
    }

    fn field(name: &str) -> Field {
        Field {
            name: name.to_string(),
            r#type: None,
        }
    }

    fn value(to_kind: impl ToKind) -> Value {
        Value {
            kind: Some(to_kind.to_kind()),
        }
    }

    fn prs(values: Vec<Value>, resume_token: &str, chunked_value: bool) -> PartialResultSet {
        PartialResultSet {
            metadata: None,
            values,
            chunked_value,
            resume_token: resume_token.as_bytes().to_vec(),
            stats: None,
            precommit_token: None,
            last: false,
        }
    }

    fn assert_one_column(rs: &ResultSet) {
        assert_eq!(rs.fields.len(), 1);
        assert_eq!(rs.fields[0].name, "column1".to_string());
        assert_eq!(*rs.index.get("column1").unwrap(), 0);
    }

    fn assert_multi_column(rs: &ResultSet) {
        assert_eq!(rs.fields.len(), 2);
        assert_eq!(rs.fields[0].name, "column1".to_string());
        assert_eq!(rs.fields[1].name, "column2".to_string());
        assert_eq!(*rs.index.get("column1").unwrap(), 0);
        assert_eq!(*rs.index.get("column2").unwrap(), 1);
    }

    fn assert_some_one_column<T: TryFromValue + std::cmp::PartialEq + std::fmt::Debug>(row: Option<Row>, v: T) {
        assert!(row.is_some());
        assert_eq!(v, row.unwrap().column::<T>(0).unwrap());
    }

    fn assert_some_multi_column<
        T1: TryFromValue + std::cmp::PartialEq + std::fmt::Debug,
        T2: TryFromValue + std::cmp::PartialEq + std::fmt::Debug,
    >(
        row: Option<Row>,
        v1: T1,
        v2: T2,
    ) {
        assert!(row.is_some());
        let v = row.unwrap();
        assert_eq!(v1, v.column::<T1>(0).unwrap());
        assert_eq!(v2, v.column::<T2>(1).unwrap());
    }

    #[test]
    fn test_rs_next_empty() {
        let mut rs = ResultSet {
            fields: Arc::new(vec![field("column1")]),
            index: Arc::new(Default::default()),
            rows: Default::default(),
            chunked_value: false,
        };
        assert!(rs.next().is_none());
    }

    #[test]
    fn test_rs_next_record_chunked_or_not() {
        let rs = |values| ResultSet {
            fields: Arc::new(vec![field("column1"), field("column2")]),
            index: Arc::new(Default::default()),
            rows: VecDeque::from(values),
            chunked_value: false,
        };
        let mut rs1 = rs(vec![value("value1")]);
        assert!(rs1.next().is_none());
        let mut rs2 = rs(vec![value("value1"), value("value2")]);
        assert_eq!(rs2.next().unwrap().column::<String>(0).unwrap(), "value1".to_string());
    }

    #[test]
    fn test_rs_next_value_chunked_or_not() {
        let rs = |chunked_value| ResultSet {
            fields: Arc::new(vec![field("column1"), field("column2")]),
            index: Arc::new(Default::default()),
            rows: VecDeque::from(vec![value("value1"), value("value2")]),
            chunked_value,
        };
        assert!(rs(true).next().is_none());
        assert_eq!(rs(false).next().unwrap().column::<String>(0).unwrap(), "value1".to_string());
    }

    #[test]
    fn test_rs_next_plural_record_one_column() {
        let rs = |chunked_value| ResultSet {
            fields: Arc::new(vec![field("column1")]),
            index: Arc::new(Default::default()),
            rows: VecDeque::from(vec![value("value1"), value("value2"), value("value3")]),
            chunked_value,
        };
        let mut incomplete = rs(true);
        assert!(incomplete.next().is_some());
        assert!(incomplete.next().is_some());
        assert!(incomplete.next().is_none());
        let mut complete = rs(false);
        assert!(complete.next().is_some());
        assert!(complete.next().is_some());
        assert!(complete.next().is_some());
        assert!(complete.next().is_none());
    }

    #[test]
    fn test_rs_next_plural_record_multi_column() {
        let rs = |chunked_value| ResultSet {
            fields: Arc::new(vec![field("column1"), field("column2")]),
            index: Arc::new(Default::default()),
            rows: VecDeque::from(vec![value("value1"), value("value2"), value("value3")]),
            chunked_value,
        };
        let mut incomplete = rs(true);
        assert_eq!(incomplete.next().unwrap().column::<String>(1).unwrap(), "value2".to_string());
        assert!(incomplete.next().is_none());
        let mut complete = rs(false);
        assert_eq!(complete.next().unwrap().column::<String>(1).unwrap(), "value2".to_string());
        assert!(incomplete.next().is_none());
    }

    #[test]
    fn test_rs_merge_string_value() {
        let result = ResultSet::merge(value("val"), value("ue1"));
        assert!(result.is_ok());
        let kind = result.unwrap().kind.unwrap();
        match kind {
            Kind::StringValue(v) => assert_eq!(v, "value1".to_string()),
            _ => unreachable!("must be string value"),
        }
    }

    #[test]
    fn test_rs_merge_list_value() {
        let previous_last = value(vec!["value1-1", "value1-2", "val"]);
        let current_first = value(vec!["ue1-3", "value2-1", "valu"]);
        let result = ResultSet::merge(previous_last, current_first);
        assert!(result.is_ok());
        let kind = result.unwrap().kind.unwrap();
        match kind {
            Kind::ListValue(v) => {
                assert_eq!(v.values.len(), 5);
                match v.values[0].kind.as_ref().unwrap() {
                    Kind::StringValue(v) => assert_eq!(*v, "value1-1".to_string()),
                    _ => unreachable!("must be string value"),
                };
                match v.values[1].kind.as_ref().unwrap() {
                    Kind::StringValue(v) => assert_eq!(*v, "value1-2".to_string()),
                    _ => unreachable!("must be string value"),
                };
                match v.values[2].kind.as_ref().unwrap() {
                    Kind::StringValue(v) => assert_eq!(*v, "value1-3".to_string()),
                    _ => unreachable!("must be string value"),
                };
                match v.values[3].kind.as_ref().unwrap() {
                    Kind::StringValue(v) => assert_eq!(*v, "value2-1".to_string()),
                    _ => unreachable!("must be string value"),
                }
                match v.values[4].kind.as_ref().unwrap() {
                    Kind::StringValue(v) => assert_eq!(*v, "valu".to_string()),
                    _ => unreachable!("must be string value"),
                }
            }
            _ => unreachable!("must be string value"),
        }
    }

    #[test]
    fn test_rs_add_one_column_no_chunked_value() {
        let mut rs = empty_rs();
        let metadata = Some(ResultSetMetadata {
            row_type: Some(StructType {
                fields: vec![field("column1")],
            }),
            transaction: None,
            undeclared_parameters: None,
        });
        let values = vec![value("value1"), value("value2"), value("value3")];
        assert!(rs.add(metadata, values, false).unwrap());
        assert_eq!(rs.rows.len(), 3);
        assert_one_column(&rs);
        assert!(!rs.chunked_value);

        assert_some_one_column(rs.next(), "value1".to_string());
        assert_some_one_column(rs.next(), "value2".to_string());
        assert_some_one_column(rs.next(), "value3".to_string());
        assert!(rs.next().is_none());
    }

    #[test]
    fn test_rs_add_multi_column_no_chunked_value() {
        let mut rs = empty_rs();
        let metadata = Some(ResultSetMetadata {
            row_type: Some(StructType {
                fields: vec![field("column1"), field("column2")],
            }),
            transaction: None,
            undeclared_parameters: None,
        });
        let values = vec![value("value1"), value("value2"), value("value3")];
        assert!(rs.add(metadata, values, false).unwrap());
        assert_eq!(rs.rows.len(), 3);
        assert_multi_column(&rs);
        assert!(!rs.chunked_value);

        assert_some_multi_column(rs.next(), "value1".to_string(), "value2".to_string());
        assert!(rs.next().is_none());
    }

    #[test]
    fn test_rs_add_multi_column_no_chunked_value_just() {
        let mut rs = empty_rs();
        let metadata = Some(ResultSetMetadata {
            row_type: Some(StructType {
                fields: vec![field("column1"), field("column2")],
            }),
            transaction: None,
            undeclared_parameters: None,
        });
        let values = vec![value("value1"), value("value2"), value("value3"), value("value4")];
        assert!(rs.add(metadata, values, false).unwrap());
        assert_eq!(rs.rows.len(), 4);
        assert_multi_column(&rs);
        assert!(!rs.chunked_value);

        assert_some_multi_column(rs.next(), "value1".to_string(), "value2".to_string());
        assert_some_multi_column(rs.next(), "value3".to_string(), "value4".to_string());
        assert!(rs.next().is_none());
    }

    #[test]
    fn test_rs_add_one_column_chunked_value() {
        let mut rs = empty_rs();
        let metadata = Some(ResultSetMetadata {
            row_type: Some(StructType {
                fields: vec![field("column1")],
            }),
            transaction: None,
            undeclared_parameters: None,
        });
        let values = vec![value("value1"), value("value2"), value("val")];
        assert!(rs.add(metadata.clone(), values, true).unwrap());
        assert_eq!(rs.rows.len(), 3);
        assert_one_column(&rs);
        assert!(rs.chunked_value);

        assert_some_one_column(rs.next(), "value1".to_string());
        assert_some_one_column(rs.next(), "value2".to_string());
        assert!(rs.next().is_none());

        // add next stream data
        assert!(rs.add(metadata, vec![value("ue3")], false).unwrap());
        assert!(!rs.chunked_value);
        assert_eq!(rs.rows.len(), 1);
        assert_some_one_column(rs.next(), "value3".to_string());
        assert!(rs.next().is_none());
    }

    #[test]
    fn test_rs_add_multi_column_chunked_value() {
        let mut rs = empty_rs();
        let metadata = Some(ResultSetMetadata {
            row_type: Some(StructType {
                fields: vec![field("column1"), field("column2")],
            }),
            transaction: None,
            undeclared_parameters: None,
        });
        let values = vec![value("value1"), value("value2"), value("val")];
        assert!(rs.add(metadata.clone(), values, true).unwrap());
        assert_eq!(rs.rows.len(), 3);
        assert_multi_column(&rs);
        assert!(rs.chunked_value);

        assert_some_multi_column(rs.next(), "value1".to_string(), "value2".to_string());
        assert!(rs.next().is_none());

        // add next stream data
        assert!(rs.add(metadata.clone(), vec![value("ue3")], false).unwrap());
        assert!(!rs.chunked_value);
        assert_eq!(rs.rows.len(), 1);
        assert!(rs.next().is_none());

        // add next stream data
        assert!(rs.add(metadata, vec![value("value4")], false).unwrap());
        assert!(!rs.chunked_value);
        assert_eq!(rs.rows.len(), 2);
        assert_some_multi_column(rs.next(), "value3".to_string(), "value4".to_string());
    }

    #[test]
    fn test_rs_add_multi_column_no_chunked_value_list_value() {
        let mut rs = empty_rs();
        let metadata = Some(ResultSetMetadata {
            row_type: Some(StructType {
                fields: vec![field("column1"), field("column2")],
            }),
            transaction: None,
            undeclared_parameters: None,
        });
        let values = vec![value(vec!["value1-1", "value1-2"])];
        assert!(rs.add(metadata.clone(), values, false).unwrap());
        assert_eq!(rs.rows.len(), 1);
        assert_multi_column(&rs);
        assert!(!rs.chunked_value);
        assert!(rs.next().is_none());
        assert!(rs.add(metadata, vec![value(vec!["value2-1"])], false).unwrap());
        assert!(!rs.chunked_value);
        assert_eq!(rs.rows.len(), 2);
        assert_some_multi_column(
            rs.next(),
            vec!["value1-1".to_string(), "value1-2".to_string()],
            vec!["value2-1".to_string()],
        );
        assert!(rs.next().is_none());
    }

    #[test]
    fn test_rs_add_multi_column_chunked_value_list_value() {
        let mut rs = empty_rs();
        let metadata = Some(ResultSetMetadata {
            row_type: Some(StructType {
                fields: vec![field("column1"), field("column2")],
            }),
            transaction: None,
            undeclared_parameters: None,
        });
        let values = vec![value(vec!["value1-1", "value1-2"]), value(vec!["value2-"])];
        assert!(rs.add(metadata.clone(), values, true).unwrap());
        assert_eq!(rs.rows.len(), 2);
        assert_multi_column(&rs);
        assert!(rs.chunked_value);
        assert!(rs.next().is_none());

        // add next stream data
        assert!(rs.add(metadata.clone(), vec![value(vec!["1", "valu"])], true).unwrap());
        assert!(rs.chunked_value);
        assert_eq!(rs.rows.len(), 2);
        assert!(rs.next().is_none());

        // add next stream data
        assert!(rs.add(metadata, vec![value(vec!["e2-2"])], false).unwrap());
        assert!(!rs.chunked_value);
        assert_eq!(rs.rows.len(), 2);
        assert_some_multi_column(
            rs.next(),
            vec!["value1-1".to_string(), "value1-2".to_string()],
            vec!["value2-1".to_string(), "value2-2".to_string()],
        );
        assert!(rs.next().is_none());
    }

    #[test]
    fn test_rs_add_multi_column_chunked_value_list_and_string_value() {
        let mut rs = empty_rs();
        let metadata = Some(ResultSetMetadata {
            row_type: Some(StructType {
                fields: vec![field("column1"), field("column2")],
            }),
            transaction: None,
            undeclared_parameters: None,
        });
        let values = vec![value(vec!["value1-1", "value1-2"]), value("va")];
        assert!(rs.add(metadata.clone(), values, true).unwrap());
        assert_eq!(rs.rows.len(), 2);
        assert_multi_column(&rs);
        assert!(rs.chunked_value);
        assert!(rs.next().is_none());

        // add next stream data
        assert!(rs
            .add(metadata.clone(), vec![value("lueA"), value(vec!["valu"])], true)
            .unwrap());
        assert!(rs.chunked_value);
        assert_eq!(rs.rows.len(), 3);
        assert_some_multi_column(
            rs.next(),
            vec!["value1-1".to_string(), "value1-2".to_string()],
            "valueA".to_string(),
        );
        assert!(rs.next().is_none());

        // add next stream data
        assert!(rs
            .add(metadata.clone(), vec![value(vec!["e2-1", "value2-2"])], false)
            .unwrap());
        assert!(!rs.chunked_value);
        assert_eq!(rs.rows.len(), 1);
        assert!(rs.next().is_none());

        // add next stream data
        assert!(rs.add(metadata.clone(), vec![value("value")], true).unwrap());
        assert!(rs.chunked_value);
        assert_eq!(rs.rows.len(), 2);
        assert!(rs.next().is_none());

        // add next stream data
        assert!(rs.add(metadata, vec![value("B")], false).unwrap());
        assert!(!rs.chunked_value);
        assert_eq!(rs.rows.len(), 2);
        assert_some_multi_column(
            rs.next(),
            vec!["value2-1".to_string(), "value2-2".to_string()],
            "valueB".to_string(),
        );
        assert!(rs.next().is_none());
    }

    #[test]
    fn test_prs_buffer_waits_for_resume_token() {
        let mut buffer = super::ResumablePartialResultSetBuffer::new(1024);
        buffer.push(prs(vec![value("value-1")], "", false));
        assert!(buffer.pop_ready(false).is_none());

        buffer.push(prs(vec![value("value-2")], "token-1", false));
        assert!(buffer.pop_ready(false).is_some());
        assert!(buffer.pop_ready(false).is_some());
        assert!(buffer.pop_ready(false).is_none());
    }

    #[test]
    fn test_prs_buffer_flushes_on_end_of_stream() {
        let mut buffer = super::ResumablePartialResultSetBuffer::new(1024);
        buffer.push(prs(vec![value("value-1")], "", false));
        assert!(buffer.pop_ready(false).is_none());
        assert!(buffer.pop_ready(true).is_some());
        assert!(buffer.pop_ready(true).is_none());
    }

    #[test]
    fn test_prs_buffer_becomes_unretryable_after_limit() {
        let mut buffer = super::ResumablePartialResultSetBuffer::new(1);
        buffer.push(prs(vec![value("value-1")], "", false));
        assert!(buffer.unretryable);
        assert!(buffer.pop_ready(false).is_some());
    }

    #[test]
    fn test_prs_buffer_on_resumption_discards_pending() {
        let mut buffer = super::ResumablePartialResultSetBuffer::new(1024);
        buffer.push(prs(vec![value("value-1")], "", false));
        buffer.on_resumption();
        buffer.push(prs(vec![value("value-2")], "token-1", false));
        assert!(buffer.pop_ready(false).is_some());
        assert!(buffer.pop_ready(false).is_none());
    }

    #[test]
    fn test_rs_is_row_boundary_empty() {
        let rs = empty_rs();
        assert!(rs.is_row_boundary());
    }

    #[test]
    fn test_rs_is_row_boundary_chunked() {
        let rs = ResultSet {
            fields: Arc::new(vec![field("column1")]),
            index: Arc::new(Default::default()),
            rows: VecDeque::from(vec![value("value1")]),
            chunked_value: true,
        };
        assert!(!rs.is_row_boundary());
    }

    #[test]
    fn test_rs_is_row_boundary_multiple_columns() {
        let rs_complete = ResultSet {
            fields: Arc::new(vec![field("column1"), field("column2")]),
            index: Arc::new(Default::default()),
            rows: VecDeque::from(vec![value("value1"), value("value2")]),
            chunked_value: false,
        };
        assert!(rs_complete.is_row_boundary());

        let rs_partial = ResultSet {
            fields: Arc::new(vec![field("column1"), field("column2")]),
            index: Arc::new(Default::default()),
            rows: VecDeque::from(vec![value("value1")]),
            chunked_value: false,
        };
        assert!(!rs_partial.is_row_boundary());
    }
}
