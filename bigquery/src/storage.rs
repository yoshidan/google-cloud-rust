use std::collections::VecDeque;
use std::io::{BufReader, Cursor};

use arrow::error::ArrowError;
use arrow::ipc::reader::StreamReader;

use google_cloud_gax::grpc::{Status, Streaming};
use google_cloud_gax::retry::RetrySetting;
use google_cloud_googleapis::cloud::bigquery::storage::v1::read_rows_response::{Rows, Schema};
use google_cloud_googleapis::cloud::bigquery::storage::v1::{
    ArrowSchema, ReadRowsRequest, ReadRowsResponse, ReadSession,
};

use crate::grpc::apiv1::bigquery_client::StreamingReadClient;

use crate::storage::value::StructDecodable;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    GRPC(#[from] Status),
    #[error(transparent)]
    ArrowNative(#[from] ArrowError),
    #[error(transparent)]
    Value(#[from] value::Error),
    #[error("data format must be arrow")]
    InvalidDataFormat,
    #[error("schema format must be arrow")]
    InvalidSchemaFormat,
    #[error("no schema found in first response")]
    NoSchemaFound,
}

pub struct Iterator<T>
where
    T: StructDecodable,
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
    T: StructDecodable,
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
    T: StructDecodable,
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
                    chunk.push_back(T::decode_arrow(row.columns(), row_no)?)
                }
            }
            Ok(chunk)
        }
        _ => Err(Error::InvalidDataFormat),
    }
}

pub mod row {
    use arrow::array::ArrayRef;

    use crate::storage::value::{Decodable, StructDecodable};

    #[derive(thiserror::Error, Debug)]
    pub enum Error {
        #[error("UnexpectedColumnIndex: {0}")]
        UnexpectedColumnIndex(usize),
        #[error(transparent)]
        ArrowError(#[from] super::value::Error),
    }

    pub struct Row {
        fields: Vec<ArrayRef>,
        row_no: usize,
    }

    impl StructDecodable for Row {
        fn decode_arrow(fields: &[ArrayRef], row_no: usize) -> Result<Row, super::value::Error> {
            Ok(Self {
                fields: fields.to_vec(),
                row_no,
            })
        }
    }

    impl Row {
        pub fn column<T: Decodable>(&self, index: usize) -> Result<T, Error> {
            let column = self.fields.get(index).ok_or(Error::UnexpectedColumnIndex(index))?;
            Ok(T::decode_arrow(column, self.row_no)?)
        }
    }
}

pub mod value {
    use std::ops::Add;

    use arrow::array::{
        Array, ArrayRef, AsArray, BinaryArray, Date32Array, Decimal128Array, Decimal256Array, Float64Array, Int64Array,
        ListArray, StringArray, Time64MicrosecondArray, TimestampMicrosecondArray,
    };
    use arrow::datatypes::{DataType, TimeUnit};
    use bigdecimal::BigDecimal;
    use time::macros::date;
    use time::{Date, Duration, OffsetDateTime, Time};

    #[derive(thiserror::Error, Debug)]
    pub enum Error {
        #[error("invalid data type actual={0}, expected={1}")]
        InvalidDataType(DataType, &'static str),
        #[error("invalid downcast dataType={0}")]
        InvalidDowncast(DataType),
        #[error("invalid non nullable")]
        InvalidNullable,
        #[error(transparent)]
        InvalidTime(#[from] time::error::ComponentRange),
        #[error(transparent)]
        InvalidDecimal(#[from] bigdecimal::ParseBigDecimalError),
    }

    /// https://cloud.google.com/bigquery/docs/reference/storage#arrow_schema_details
    pub trait Decodable: Sized {
        fn decode_arrow(col: &dyn Array, row_no: usize) -> Result<Self, Error>;
    }

    pub trait StructDecodable: Sized {
        fn decode_arrow(fields: &[ArrayRef], row_no: usize) -> Result<Self, Error>;
    }

    impl<S> Decodable for S
    where
        S: StructDecodable,
    {
        fn decode_arrow(col: &dyn Array, row_no: usize) -> Result<S, Error> {
            match col.data_type() {
                DataType::Struct(_) => S::decode_arrow(downcast::<arrow::array::StructArray>(col)?.columns(), row_no),
                _ => Err(Error::InvalidDataType(col.data_type().clone(), "struct")),
            }
        }
    }

    impl Decodable for bool {
        fn decode_arrow(col: &dyn Array, row_no: usize) -> Result<Self, Error> {
            if col.is_null(row_no) {
                return Err(Error::InvalidNullable);
            }
            match col.data_type() {
                DataType::Boolean => Ok(col.as_boolean().value(row_no)),
                _ => Err(Error::InvalidDataType(col.data_type().clone(), "bool")),
            }
        }
    }

    impl Decodable for i64 {
        fn decode_arrow(col: &dyn Array, row_no: usize) -> Result<Self, Error> {
            if col.is_null(row_no) {
                return Err(Error::InvalidNullable);
            }
            match col.data_type() {
                DataType::Int64 => Ok(downcast::<Int64Array>(col)?.value(row_no)),
                _ => Err(Error::InvalidDataType(col.data_type().clone(), "i64")),
            }
        }
    }

    impl Decodable for f64 {
        fn decode_arrow(col: &dyn Array, row_no: usize) -> Result<Self, Error> {
            if col.is_null(row_no) {
                return Err(Error::InvalidNullable);
            }
            match col.data_type() {
                DataType::Float64 => Ok(downcast::<Float64Array>(col)?.value(row_no)),
                _ => Err(Error::InvalidDataType(col.data_type().clone(), "f64")),
            }
        }
    }

    impl Decodable for Vec<u8> {
        fn decode_arrow(col: &dyn Array, row_no: usize) -> Result<Self, Error> {
            if col.is_null(row_no) {
                return Err(Error::InvalidNullable);
            }
            match col.data_type() {
                DataType::Binary => Ok(downcast::<BinaryArray>(col)?.value(row_no).into()),
                _ => Err(Error::InvalidDataType(col.data_type().clone(), "Vec<u8>")),
            }
        }
    }

    impl Decodable for String {
        fn decode_arrow(col: &dyn Array, row_no: usize) -> Result<Self, Error> {
            if col.is_null(row_no) {
                return Err(Error::InvalidNullable);
            }
            match col.data_type() {
                DataType::Decimal128(_, _) => BigDecimal::decode_arrow(col, row_no).map(|v| v.to_string()),
                DataType::Decimal256(_, _) => BigDecimal::decode_arrow(col, row_no).map(|v| v.to_string()),
                DataType::Date32 => Date::decode_arrow(col, row_no).map(|v| v.to_string()),
                DataType::Timestamp(_, _) => OffsetDateTime::decode_arrow(col, row_no).map(|v| v.to_string()),
                DataType::Time64(_) => Time::decode_arrow(col, row_no).map(|v| v.to_string()),
                DataType::Boolean => bool::decode_arrow(col, row_no).map(|v| v.to_string()),
                DataType::Float64 => f64::decode_arrow(col, row_no).map(|v| v.to_string()),
                DataType::Int64 => i64::decode_arrow(col, row_no).map(|v| v.to_string()),
                DataType::Utf8 => Ok(downcast::<StringArray>(col)?.value(row_no).to_string()),
                _ => Err(Error::InvalidDataType(col.data_type().clone(), "String")),
            }
        }
    }

    impl Decodable for BigDecimal {
        fn decode_arrow(col: &dyn Array, row_no: usize) -> Result<Self, Error> {
            if col.is_null(row_no) {
                return Err(Error::InvalidNullable);
            }
            match col.data_type() {
                DataType::Decimal128(_, _) => {
                    let decimal = downcast::<Decimal128Array>(col)?;
                    let value = decimal.value(row_no);
                    let bigint = num_bigint::BigInt::from_signed_bytes_le(&value.to_le_bytes());
                    Ok(BigDecimal::from((bigint, decimal.scale() as i64)))
                }
                DataType::Decimal256(_, _) => {
                    let decimal = downcast::<Decimal256Array>(col)?;
                    let value = decimal.value(row_no);
                    let bigint = num_bigint::BigInt::from_signed_bytes_le(&value.to_le_bytes());
                    Ok(BigDecimal::from((bigint, decimal.scale() as i64)))
                }
                _ => Err(Error::InvalidDataType(col.data_type().clone(), "Decimal128")),
            }
        }
    }

    impl Decodable for Time {
        fn decode_arrow(col: &dyn Array, row_no: usize) -> Result<Self, Error> {
            if col.is_null(row_no) {
                return Err(Error::InvalidNullable);
            }
            match col.data_type() {
                DataType::Time64(tu) => match tu {
                    TimeUnit::Microsecond => {
                        let micros = downcast::<Time64MicrosecondArray>(col)?.value(row_no);
                        Ok(Time::from_hms_micro(0, 0, 0, micros as u32)?)
                    }
                    _ => Err(Error::InvalidDataType(col.data_type().clone(), "Time")),
                },
                _ => Err(Error::InvalidDataType(col.data_type().clone(), "Time")),
            }
        }
    }

    impl Decodable for Date {
        fn decode_arrow(col: &dyn Array, row_no: usize) -> Result<Self, Error> {
            if col.is_null(row_no) {
                return Err(Error::InvalidNullable);
            }
            match col.data_type() {
                DataType::Date32 => {
                    let days_from_epoch = downcast::<Date32Array>(col)?.value(row_no);
                    const UNIX_EPOCH: Date = date!(1970 - 01 - 01);
                    Ok(UNIX_EPOCH.add(Duration::days(days_from_epoch as i64)))
                }
                _ => Err(Error::InvalidDataType(col.data_type().clone(), "DaysFromEpoch")),
            }
        }
    }

    impl Decodable for OffsetDateTime {
        fn decode_arrow(col: &dyn Array, row_no: usize) -> Result<Self, Error> {
            if col.is_null(row_no) {
                return Err(Error::InvalidNullable);
            }
            match col.data_type() {
                DataType::Timestamp(tu, _zone) => match tu {
                    TimeUnit::Microsecond => {
                        let micros = downcast::<TimestampMicrosecondArray>(col)?.value(row_no);
                        Ok(OffsetDateTime::from_unix_timestamp_nanos(micros as i128 * 1000)?)
                    }
                    _ => Err(Error::InvalidDataType(col.data_type().clone(), "Days")),
                },
                _ => Err(Error::InvalidDataType(col.data_type().clone(), "Days")),
            }
        }
    }

    impl<T> Decodable for Option<T>
    where
        T: Decodable,
    {
        fn decode_arrow(col: &dyn Array, row_no: usize) -> Result<Option<T>, Error> {
            if col.is_null(row_no) {
                return Ok(None);
            }
            Ok(Some(T::decode_arrow(col, row_no)?))
        }
    }

    impl<T> Decodable for Vec<T>
    where
        T: Decodable,
    {
        fn decode_arrow(col: &dyn Array, row_no: usize) -> Result<Vec<T>, Error> {
            match col.data_type() {
                DataType::List(_) => {
                    let list = downcast::<ListArray>(col)?;
                    let col = list.value(row_no);
                    let mut result: Vec<T> = Vec::with_capacity(col.len());
                    for row_num in 0..col.len() {
                        result.push(T::decode_arrow(&col, row_num)?);
                    }
                    Ok(result)
                }
                _ => Err(Error::InvalidDataType(col.data_type().clone(), "Days")),
            }
        }
    }

    fn downcast<T: 'static>(col: &dyn Array) -> Result<&T, Error> {
        col.as_any()
            .downcast_ref::<T>()
            .ok_or(Error::InvalidDowncast(col.data_type().clone()))
    }

    #[cfg(test)]
    mod test {
        use arrow::array::BooleanArray;

        use crate::storage::value::Decodable;

        #[test]
        fn test_bool() {
            let v = vec![Some(false), Some(true), Some(false), Some(true)];
            let array = v.into_iter().collect::<BooleanArray>();
            assert!(!bool::decode_arrow(&array, 0).unwrap());
            assert!(bool::decode_arrow(&array, 1).unwrap());
            assert!(!bool::decode_arrow(&array, 2).unwrap());
            assert!(bool::decode_arrow(&array, 3).unwrap())
        }

        #[test]
        fn test_bool_option() {
            let v = vec![Some(true), None];
            let array = v.into_iter().collect::<BooleanArray>();
            assert!(Option::<bool>::decode_arrow(&array, 0).unwrap().unwrap());
            assert!(Option::<bool>::decode_arrow(&array, 1).unwrap().is_none());
        }
    }
}
