use std::convert::Infallible;
use crate::http::tabledata::list::{Cell, Value};

#[derive(thiserror::Error)]
pub enum Error {
    NoDataFound(i32),
    Decode(String),
}

pub struct Row {
    inner: Vec<Cell>
}

impl Row {
    pub fn column<T: TryFrom<&Value, Error=String>>(&self, index:i32) -> Result<T, Error> {
        let cell : &Cell = self.inner.get(index).ok_or(Error::NoDataFound(index))?;
        T::try_from(&cell.v).map_err(Error::Decode)
    }
}

impl TryFrom<&Value> for &str {
    type Error = String;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        Ok(match value {
            Value::String(v) => v.as_str(),
            Value::Null => "",
            _ => "invalid value for &str"
        })
    }
}