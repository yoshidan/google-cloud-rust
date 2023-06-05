use crate::types;
use crate::types::Numeric;
use arrow::array::{
    Array, AsArray, BinaryArray, Date32Array, Date64Array, Decimal128Array, Float32Array, Float64Array, Int16Array,
    Int32Array, Int64Array, Int8Array, StringArray, Time32MillisecondArray, Time32SecondArray, Time64MicrosecondArray,
    Time64NanosecondArray, TimestampMicrosecondArray, UInt16Array, UInt32Array, UInt64Array, UInt8Array,
};
use arrow::datatypes::DataType::Boolean;
use arrow::datatypes::{ArrowPrimitiveType, DataType, TimeUnit};
use std::ops::Add;
use time::macros::date;
use time::{Date, Duration, Month, OffsetDateTime, Time};

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
}

/// https://cloud.google.com/bigquery/docs/reference/storage#arrow_schema_details
pub trait ArrowDecodable<T> {
    fn decode(col: &dyn Array, row_no: usize) -> Result<T, Error>;
}

impl ArrowDecodable<bool> for bool {
    fn decode(col: &dyn Array, row_no: usize) -> Result<bool, Error> {
        if col.is_null(row_no) {
            return Err(Error::InvalidNullable);
        }
        match col.data_type() {
            Boolean => Ok(col.as_boolean().value(row_no)),
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
            DataType::Utf8 => Ok(downcast::<StringArray>(col)?.value(row_no).to_string()),
            _ => Err(Error::InvalidDataType(col.data_type().clone(), "String")),
        }
    }
}

impl ArrowDecodable<Numeric> for Numeric {
    fn decode(col: &dyn Array, row_no: usize) -> Result<Numeric, Error> {
        if col.is_null(row_no) {
            return Err(Error::InvalidNullable);
        }
        match col.data_type() {
            DataType::Decimal128(precision, scale) => Ok(Numeric {
                precision: precision.clone(),
                scale: scale.clone(),
            }),
            DataType::Decimal256(precision, scale) => Ok(Numeric {
                precision: precision.clone(),
                scale: scale.clone(),
            }),
            _ => Err(Error::InvalidDataType(col.data_type().clone(), "String")),
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

fn downcast<T: 'static>(col: &dyn Array) -> Result<&T, Error> {
    Ok(col
        .as_any()
        .downcast_ref::<T>()
        .ok_or(Error::InvalidDowncast(col.data_type().clone()))?)
}

#[cfg(test)]
mod test {
    use crate::arrow::ArrowDecodable;
    use arrow::array::BooleanArray;
    use std::sync::Arc;

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
