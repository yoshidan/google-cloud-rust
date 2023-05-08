use std::convert::Infallible;
use crate::http::tabledata::list::{Cell, Tuple, Value};

#[derive(thiserror::Error,Debug)]
pub enum Error {
    #[error("not data found")]
    NoDataFound,
    #[error("invalid type")]
    Decode(String),
}

pub struct Row {
    inner: Vec<Cell>
}

impl Row {
    pub fn column<'a, T: TryFrom<&'a Value, Error=String>>(&'a self, index:usize) -> Result<T, Error> {
        let cell : &Cell = self.inner.get(index).ok_or(Error::NoDataFound)?;
        T::try_from(&cell.v).map_err(Error::Decode)
    }
}

impl TryFrom<Tuple> for Row {
    type Error = String;

    fn try_from(value: Tuple) -> Result<Self, Self::Error> {
        Ok(Self {
            inner: value.f
        })
    }
}

impl <'a> TryFrom<&'a Value> for &'a str {
    type Error = String;

    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        Ok(match value {
            Value::String(v) => v.as_str(),
            Value::Null => "",
            _ => "invalid value for &str"
        })
    }
}