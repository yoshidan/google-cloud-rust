use arrow::array::{
    Array, ArrayAccessor, ArrayRef, AsArray, BinaryArray, Date32Array, Decimal128Array,
    Decimal256Array, Float64Array, Int64Array, ListArray, StringArray, Time64MicrosecondArray,
    TimestampMicrosecondArray,
};
use arrow::datatypes::{DataType, TimeUnit};

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
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Decimal128 {
    pub value: i128,
    pub precision: u8,
    pub scale: i8,
}

impl ToString for Decimal128 {
    fn to_string(&self) -> String {
        format_decimal_str(self.value.to_string().as_str(), self.precision as usize, self.scale)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Decimal256 {
    pub value: arrow::datatypes::i256,
    pub precision: u8,
    pub scale: i8,
}

impl ToString for Decimal256 {
    fn to_string(&self) -> String {
        format_decimal_str(self.value.to_string().as_str(), self.precision as usize, self.scale)
    }
}

fn format_decimal_str(value_str: &str, precision: usize, scale: i8) -> String {
    let (sign, rest) = match value_str.strip_prefix('-') {
        Some(stripped) => ("-", stripped),
        None => ("", value_str),
    };
    let bound = precision.min(rest.len()) + sign.len();
    let value_str = &value_str[0..bound];

    if scale == 0 {
        value_str.to_string()
    } else if scale < 0 {
        let padding = value_str.len() + scale.unsigned_abs() as usize;
        format!("{value_str:0<padding$}")
    } else if rest.len() > scale as usize {
        // Decimal separator is in the middle of the string
        let (whole, decimal) = value_str.split_at(value_str.len() - scale as usize);
        format!("{whole}.{decimal}")
    } else {
        // String has to be padded
        format!("{}0.{:0>width$}", sign, rest, width = scale as usize)
    }
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
            DataType::Decimal128(_, _) => Decimal128::decode(col, row_no).map(|v| v.to_string()),
            DataType::Decimal256(_, _) => Decimal256::decode(col, row_no).map(|v| v.to_string()),
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

impl ArrowDecodable<Decimal128> for Decimal128 {
    fn decode(col: &dyn Array, row_no: usize) -> Result<Decimal128, Error> {
        if col.is_null(row_no) {
            return Err(Error::InvalidNullable);
        }
        match col.data_type() {
            DataType::Decimal128(precision, scale) => {
                let value = downcast::<Decimal128Array>(col)?.value(row_no);
                Ok(Decimal128 {
                    value,
                    precision: *precision,
                    scale: *scale,
                })
            }
            _ => Err(Error::InvalidDataType(col.data_type().clone(), "Decimal128")),
        }
    }
}

impl ArrowDecodable<Decimal256> for Decimal256 {
    fn decode(col: &dyn Array, row_no: usize) -> Result<Decimal256, Error> {
        if col.is_null(row_no) {
            return Err(Error::InvalidNullable);
        }
        match col.data_type() {
            DataType::Decimal256(precision, scale) => {
                let value = downcast::<Decimal256Array>(col)?.value(row_no);
                Ok(Decimal256 {
                    value,
                    precision: *precision,
                    scale: *scale,
                })
            }
            _ => Err(Error::InvalidDataType(col.data_type().clone(), "Decimal256")),
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
    col
        .as_any()
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
