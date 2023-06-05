use arrow::array::{Array, AsArray, Int16Array, Int8Array};
use arrow::datatypes::{ArrowPrimitiveType, DataType};
use arrow::datatypes::DataType::Boolean;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid data type actual={0}, expected={1}")]
    InvalidDataType(DataType, &'static str),
    #[error("invalid downcast dataType={0}")]
    InvalidDowncast(DataType),
    #[error("invalid non nullable")]
    InvalidNullable
}

pub trait ArrowDecodable<T> {
    fn decode(col: &dyn Array, row_no: usize) -> Result<T, Error>;
}

impl ArrowDecodable<bool> for bool {
    fn decode(col: &dyn Array, row_no: usize) -> Result<bool, Error> {
        if col.is_null(row_no){
            return Err(Error::InvalidNullable)
        }
        match col.data_type() {
            Boolean => Ok(col.as_boolean().value(row_no)),
            _ => Err(Error::InvalidDataType(col.data_type().clone(), "bool"))
        }
    }
}

impl ArrowDecodable<i8> for i8 {
    fn decode(col: &dyn Array, row_no: usize) -> Result<i8, Error> {
        if col.is_null(row_no) {
            return Err(Error::InvalidNullable)
        }
        match col.data_type() {
            DataType::Int8 => Ok(downcast::<Int8Array>(col)?.value(row_no)),
            _ => Err(Error::InvalidDataType(col.data_type().clone(), "i16"))
        }
    }
}

impl <T> ArrowDecodable<Option<T>> for Option<T> where T: ArrowDecodable<T>{
    fn decode(col: &dyn Array, row_no: usize) -> Result<Option<T>, Error> {
        if col.is_null(row_no){
            return Ok(None)
        }
        Ok(Some(T::decode(col, row_no)?))
    }
}

fn downcast<T: 'static>(col: &dyn Array) -> Result<&T, Error> {
    Ok(col.as_any().downcast_ref::<T>().ok_or(Error::InvalidDowncast(col.data_type().clone()))?)
}

#[cfg(test)]
mod test {
    use std::sync::Arc;
    use arrow::array::{BooleanArray};
    use crate::arrow::ArrowDecodable;

    #[test]
    fn test_bool() {
        let v = vec![Some(false), Some(true), Some(false), Some(true)];
        let array = v.into_iter().collect::<BooleanArray>();
        assert!(!bool::decode(&array, 0).unwrap());
        assert!(bool::decode(&array, 1).unwrap());
        assert!(!bool::decode(&array, 2).unwrap());
        assert!(bool::decode(&array, 3).unwrap())
    }
}