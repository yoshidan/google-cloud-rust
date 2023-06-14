use arrow::array::{
    Array, ArrayRef, AsArray, BinaryArray, Date32Array, Decimal128Array, Decimal256Array, Float64Array, Int64Array,
    ListArray, StringArray, Time64MicrosecondArray, TimestampMicrosecondArray,
};
use arrow::datatypes::{DataType, TimeUnit};

use bigdecimal::BigDecimal;
use std::ops::Add;
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
pub trait ArrowDecodable : Sized {
    fn decode_arrow(col: &dyn Array, row_no: usize) -> Result<Self, Error>;
}

pub trait ArrowStructDecodable : Sized {
    fn decode_arrow(fields: &[ArrayRef], row_no: usize) -> Result<Self, Error>;
}

impl <S> ArrowDecodable for S
where
    S: ArrowStructDecodable,
{
    fn decode_arrow(col: &dyn Array, row_no: usize) -> Result<S, Error> {
        match col.data_type() {
            DataType::Struct(_) => S::decode_arrow(downcast::<arrow::array::StructArray>(col)?.columns(), row_no),
            _ => Err(Error::InvalidDataType(col.data_type().clone(), "struct")),
        }
    }
}

impl ArrowDecodable for bool {
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

impl ArrowDecodable for i64 {
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

impl ArrowDecodable for f64 {
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

impl ArrowDecodable for Vec<u8> {
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

impl ArrowDecodable for String {
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

impl ArrowDecodable for BigDecimal {
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

impl ArrowDecodable for Time {
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

impl ArrowDecodable for Date {
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

impl ArrowDecodable for OffsetDateTime {
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

impl<T> ArrowDecodable for Option<T>
where
    T: ArrowDecodable,
{
    fn decode_arrow(col: &dyn Array, row_no: usize) -> Result<Option<T>, Error> {
        if col.is_null(row_no) {
            return Ok(None);
        }
        Ok(Some(T::decode_arrow(col, row_no)?))
    }
}

impl<T> ArrowDecodable for Vec<T>
where
    T: ArrowDecodable,
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
    use crate::arrow::ArrowDecodable;
    use arrow::array::BooleanArray;

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
