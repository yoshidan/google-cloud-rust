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

pub struct ResultSet {
    fields: Arc<Vec<Field>>,
    index: Arc<HashMap<String, usize>>,
    rows: VecDeque<Value>,
    chunked_value: bool,
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
}

pub struct RowIterator<'a> {
    streaming: Streaming<PartialResultSet>,
    session: &'a mut SessionHandle,
    reader: Box<dyn Reader + Sync + Send>,
    rs: ResultSet,
    reader_option: Option<CallOptions>,
}

impl<'a> RowIterator<'a> {
    pub(crate) async fn new(
        session: &'a mut SessionHandle,
        reader: Box<dyn Reader + Sync + Send>,
        option: Option<CallOptions>,
    ) -> Result<RowIterator<'a>, Status> {
        let streaming = reader.read(session, option).await?.into_inner();
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
                    return Err(e);
                }
                tracing::debug!("streaming error: {}. resume reading by resume_token", e);
                let result = self.reader.read(self.session, option).await?;
                self.streaming = result.into_inner();
                self.streaming.message().await?
            }
        };

        match maybe_result_set {
            Some(result_set) => {
                if result_set.values.is_empty() {
                    return Ok(false);
                }
                //if resume_token changes set new resume_token
                if !result_set.resume_token.is_empty() {
                    self.reader.update_token(result_set.resume_token);
                }
                self.rs
                    .add(result_set.metadata, result_set.values, result_set.chunked_value)
            }
            None => Ok(false),
        }
    }
}

#[async_trait]
impl<'a> AsyncIterator for RowIterator<'a> {
    fn column_metadata(&self, column_name: &str) -> Option<(usize, Field)> {
        for (i, val) in self.rs.fields.iter().enumerate() {
            if val.name == column_name {
                return Some((i, val.clone()));
            }
        }
        None
    }

    /// next returns the next result.
    /// Its second return value is None if there are no more results.
    async fn next(&mut self) -> Result<Option<Row>, Status> {
        let row = self.rs.next();
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
    use crate::reader::ResultSet;
    use crate::row::{Row, TryFromValue};
    use crate::statement::ToKind;
    use google_cloud_googleapis::spanner::v1::struct_type::Field;
    use google_cloud_googleapis::spanner::v1::{ResultSetMetadata, StructType};
    use prost_types::value::Kind;
    use prost_types::Value;
    use std::collections::VecDeque;
    use std::sync::Arc;

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
}
