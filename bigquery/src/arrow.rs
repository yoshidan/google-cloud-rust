use arrow::array::{
    Array, ArrayAccessor, ArrayRef, AsArray, BinaryArray, Date32Array, Decimal128Array, Decimal256Array, Float64Array,
    Int64Array, ListArray, StringArray, Time64MicrosecondArray, TimestampMicrosecondArray,
};
use arrow::datatypes::{DataType, TimeUnit};

use bigdecimal::BigDecimal;
use std::ops::Add;
use std::str::FromStr;
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
pub trait ArrowDecodable<T> {
    fn decode(col: &dyn Array, row_no: usize) -> Result<T, Error>;
}

pub trait ArrowStructDecodable<T> {
    fn decode(fields: &[ArrayRef], row_no: usize) -> Result<T, Error>;
}

impl<S> ArrowDecodable<S> for S
where
    S: ArrowStructDecodable<S>,
{
    fn decode(col: &dyn Array, row_no: usize) -> Result<S, Error> {
        match col.data_type() {
            DataType::Struct(_) => S::decode(downcast::<arrow::array::StructArray>(col)?.columns(), row_no),
            _ => Err(Error::InvalidDataType(col.data_type().clone(), "struct")),
        }
    }
}

impl ArrowDecodable<bool> for bool {
    fn decode(col: &dyn Array, row_no: usize) -> Result<bool, Error> {
        if col.is_null(row_no) {
            return Err(Error::InvalidNullable);
        }
        match col.data_type() {
            DataType::Boolean => Ok(col.as_boolean().value(row_no)),
            _ => Err(Error::InvalidDataType(col.data_type().clone(), "bool")),
        }
    }
}

impl ArrowDecodable<i64> for i64 {
    fn decode(col: &dyn Array, row_no: usize) -> Result<i64, Error> {
        if col.is_null(row_no) {
            return Err(Error::InvalidNullable);
        }
        match col.data_type() {
            DataType::Int64 => Ok(downcast::<Int64Array>(col)?.value(row_no)),
            _ => Err(Error::InvalidDataType(col.data_type().clone(), "i64")),
        }
    }
}

impl ArrowDecodable<f64> for f64 {
    fn decode(col: &dyn Array, row_no: usize) -> Result<f64, Error> {
        if col.is_null(row_no) {
            return Err(Error::InvalidNullable);
        }
        match col.data_type() {
            DataType::Float64 => Ok(downcast::<Float64Array>(col)?.value(row_no)),
            _ => Err(Error::InvalidDataType(col.data_type().clone(), "f64")),
        }
    }
}

impl ArrowDecodable<Vec<u8>> for Vec<u8> {
    fn decode(col: &dyn Array, row_no: usize) -> Result<Vec<u8>, Error> {
        if col.is_null(row_no) {
            return Err(Error::InvalidNullable);
        }
        match col.data_type() {
            DataType::Binary => Ok(downcast::<BinaryArray>(col)?.value(row_no).into()),
            _ => Err(Error::InvalidDataType(col.data_type().clone(), "Vec<u8>")),
        }
    }
}

impl ArrowDecodable<String> for String {
    fn decode(col: &dyn Array, row_no: usize) -> Result<String, Error> {
        if col.is_null(row_no) {
            return Err(Error::InvalidNullable);
        }
        match col.data_type() {
            DataType::Decimal128(_, _) => Ok(downcast::<Decimal128Array>(col)?.value_as_string(row_no)),
            DataType::Decimal256(_, _) => Ok(downcast::<Decimal256Array>(col)?.value_as_string(row_no)),
            DataType::Date32 => Date::decode(col, row_no).map(|v| v.to_string()),
            DataType::Timestamp(_, _) => OffsetDateTime::decode(col, row_no).map(|v| v.to_string()),
            DataType::Time64(_) => Time::decode(col, row_no).map(|v| v.to_string()),
            DataType::Boolean => bool::decode(col, row_no).map(|v| v.to_string()),
            DataType::Float64 => f64::decode(col, row_no).map(|v| v.to_string()),
            DataType::Int64 => i64::decode(col, row_no).map(|v| v.to_string()),
            DataType::Utf8 => Ok(downcast::<StringArray>(col)?.value(row_no).to_string()),
            _ => Err(Error::InvalidDataType(col.data_type().clone(), "String")),
        }
    }
}

impl ArrowDecodable<BigDecimal> for BigDecimal {
    fn decode(col: &dyn Array, row_no: usize) -> Result<BigDecimal, Error> {
        if col.is_null(row_no) {
            return Err(Error::InvalidNullable);
        }
        match col.data_type() {
            DataType::Decimal128(precision, scale) => {
                let value = downcast::<Decimal128Array>(col)?.value_as_string(row_no);
                Ok(BigDecimal::from_str(&value)?)
            }
            DataType::Decimal256(precision, scale) => {
                let value = downcast::<Decimal256Array>(col)?.value_as_string(row_no);
                Ok(BigDecimal::from_str(&value)?)
            }
            _ => Err(Error::InvalidDataType(col.data_type().clone(), "Decimal128")),
        }
    }
}

impl ArrowDecodable<Time> for Time {
    fn decode(col: &dyn Array, row_no: usize) -> Result<Time, Error> {
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

impl ArrowDecodable<Date> for Date {
    fn decode(col: &dyn Array, row_no: usize) -> Result<Date, Error> {
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

impl ArrowDecodable<OffsetDateTime> for OffsetDateTime {
    fn decode(col: &dyn Array, row_no: usize) -> Result<OffsetDateTime, Error> {
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

impl<T> ArrowDecodable<Option<T>> for Option<T>
where
    T: ArrowDecodable<T>,
{
    fn decode(col: &dyn Array, row_no: usize) -> Result<Option<T>, Error> {
        if col.is_null(row_no) {
            return Ok(None);
        }
        Ok(Some(T::decode(col, row_no)?))
    }
}

impl<T> ArrowDecodable<Vec<T>> for Vec<T>
where
    T: ArrowDecodable<T>,
{
    fn decode(col: &dyn Array, row_no: usize) -> Result<Vec<T>, Error> {
        match col.data_type() {
            DataType::List(_) => {
                let list = downcast::<ListArray>(col)?;
                let col = list.value(row_no);
                let mut result: Vec<T> = Vec::with_capacity(col.len());
                for row_num in 0..col.len() {
                    result.push(T::decode(&col, row_num)?);
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
        assert!(!bool::decode(&array, 0).unwrap());
        assert!(bool::decode(&array, 1).unwrap());
        assert!(!bool::decode(&array, 2).unwrap());
        assert!(bool::decode(&array, 3).unwrap())
    }

    #[test]
    fn test_bool_option() {
        let v = vec![Some(true), None];
        let array = v.into_iter().collect::<BooleanArray>();
        assert!(Option::<bool>::decode(&array, 0).unwrap().unwrap());
        assert!(Option::<bool>::decode(&array, 1).unwrap().is_none());
    }
}
