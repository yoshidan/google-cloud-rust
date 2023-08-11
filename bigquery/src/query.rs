pub use backon::*;

use crate::{http, storage};

#[derive(Debug, Clone)]
pub struct QueryOption {
    /// Exponential back off retry setting
    pub(crate) retry: ExponentialBuilder,
    /// true: use storage api is page token is empty
    pub(crate) enable_storage_read: bool,
}

impl Default for QueryOption {
    fn default() -> Self {
        Self {
            enable_storage_read: false,
            retry: ExponentialBuilder::default().with_max_times(usize::MAX),
        }
    }
}

impl QueryOption {
    pub fn with_retry(mut self, builder: ExponentialBuilder) -> Self {
        self.retry = builder;
        self
    }
    pub fn with_enable_storage_read(mut self, value: bool) -> Self {
        self.enable_storage_read = value;
        self
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Http(#[from] http::query::Error),
    #[error(transparent)]
    Storage(#[from] storage::Error),
}

pub enum QueryResult<T: http::query::value::StructDecodable + storage::value::StructDecodable> {
    Http(http::query::Iterator<T>),
    Storage(storage::Iterator<T>),
}

pub struct Iterator<T: http::query::value::StructDecodable + storage::value::StructDecodable> {
    pub(crate) inner: QueryResult<T>,
    pub total_size: i64,
}

impl<T: http::query::value::StructDecodable + storage::value::StructDecodable> Iterator<T> {
    pub async fn next(&mut self) -> Result<Option<T>, Error> {
        Ok(match self.inner {
            QueryResult::Storage(ref mut v) => v.next().await?,
            QueryResult::Http(ref mut v) => v.next().await?,
        })
    }
}

pub mod row {
    use crate::http::tabledata::list::Tuple;
    use crate::{http, storage};
    use arrow::array::ArrayRef;

    #[derive(thiserror::Error, Debug)]
    pub enum Error {
        #[error(transparent)]
        Http(#[from] http::query::row::Error),
        #[error(transparent)]
        Storage(#[from] storage::row::Error),
    }

    pub enum RowType {
        Http(http::query::row::Row),
        Storage(storage::row::Row),
    }

    pub struct Row {
        inner: RowType,
    }

    impl Row {
        pub fn column<T: http::query::value::Decodable + storage::value::Decodable>(
            &self,
            index: usize,
        ) -> Result<T, Error> {
            Ok(match &self.inner {
                RowType::Http(row) => row.column(index)?,
                RowType::Storage(row) => row.column(index)?,
            })
        }
    }

    impl http::query::value::StructDecodable for Row {
        fn decode(value: Tuple) -> Result<Self, http::query::value::Error> {
            Ok(Self {
                inner: RowType::Http(http::query::row::Row::decode(value)?),
            })
        }
    }

    impl storage::value::StructDecodable for Row {
        fn decode_arrow(fields: &[ArrayRef], row_no: usize) -> Result<Self, storage::value::Error> {
            Ok(Self {
                inner: RowType::Storage(storage::row::Row::decode_arrow(fields, row_no)?),
            })
        }
    }
}

pub mod run {
    #[derive(thiserror::Error, Debug)]
    pub enum Error {
        #[error(transparent)]
        Http(#[from] crate::http::error::Error),
        #[error("Retry exceeded with job incomplete")]
        JobIncomplete,
    }
}
