use std::ops::DerefMut;
use std::sync::atomic::AtomicI64;

use prost_types::Struct;
use tonic::Status;

use google_cloud_gax::call_option::BackoffRetrySettings;
use google_cloud_googleapis::spanner::v1::request_options::Priority;
use google_cloud_googleapis::spanner::v1::{
    execute_sql_request::QueryMode, execute_sql_request::QueryOptions as ExecuteQueryOptions,
    ExecuteSqlRequest, ReadRequest, RequestOptions, TransactionSelector,
};

use crate::key::KeySet;
use crate::reader::{RowIterator, StatementReader, TableReader};
use crate::sessions::ManagedSession;
use crate::statement::Statement;

#[derive(Clone)]
pub struct CallOptions {
    /// Priority is the RPC priority to use for the read operation.
    pub priority: Option<Priority>,
    pub call_setting: Option<BackoffRetrySettings>,
}

impl Default for CallOptions {
    fn default() -> Self {
        CallOptions {
            priority: None,
            call_setting: None,
        }
    }
}

#[derive(Clone)]
pub struct ReadOptions {
    /// The index to use for reading. If non-empty, you can only read columns
    /// that are part of the index key, part of the primary key, or stored in the
    /// index due to a STORING clause in the index definition.
    pub index: String,

    /// The maximum number of rows to read. A limit value less than 1 means no limit.
    pub limit: i64,

    pub call_options: CallOptions,
}

impl Default for ReadOptions {
    fn default() -> Self {
        return ReadOptions {
            index: "".to_string(),
            limit: 0,
            call_options: CallOptions::default(),
        };
    }
}

#[derive(Clone)]
pub struct QueryOptions {
    pub mode: QueryMode,
    pub optimizer_options: Option<ExecuteQueryOptions>,
    pub call_options: CallOptions,
}

impl Default for QueryOptions {
    fn default() -> Self {
        return QueryOptions {
            mode: QueryMode::Normal,
            optimizer_options: None,
            call_options: CallOptions::default(),
        };
    }
}

pub struct Transaction {
    pub(crate) session: Option<ManagedSession>,
    // for returning ownership of session on before destroy
    pub(crate) sequence_number: AtomicI64,
    pub(crate) transaction_selector: TransactionSelector,
}

impl Transaction {
    pub(crate) fn create_request_options(priority: Option<Priority>) -> Option<RequestOptions> {
        return match priority {
            None => None,
            Some(s) => Some(RequestOptions {
                priority: s.into(),
                request_tag: "".to_string(),
                transaction_tag: "".to_string(),
            }),
        };
    }

    /// query executes a query against the database. It returns a RowIterator for
    /// retrieving the resulting rows.
    ///
    /// query returns only row data, without a query plan or execution statistics.
    pub async fn query(
        &mut self,
        statement: Statement,
        options: Option<QueryOptions>,
    ) -> Result<RowIterator<'_>, Status> {
        let opt = match options {
            Some(o) => o,
            None => QueryOptions::default(),
        };

        let request = ExecuteSqlRequest {
            session: self.session.as_ref().unwrap().session.name.to_string(),
            transaction: Some(self.transaction_selector.clone()),
            sql: statement.sql,
            params: Some(Struct {
                fields: statement.params,
            }),
            param_types: statement.param_types,
            resume_token: vec![],
            query_mode: opt.mode.into(),
            partition_token: vec![],
            seqno: 0,
            query_options: opt.optimizer_options,
            request_options: Transaction::create_request_options(opt.call_options.priority),
        };
        let session = self.session.as_mut().unwrap().deref_mut();
        return RowIterator::new(
            session,
            Box::new(StatementReader {
                request,
                call_setting: opt.call_options.call_setting,
            }),
        )
        .await;
    }

    /// read returns a RowIterator for reading multiple rows from the database.
    pub async fn read<T, C, K>(
        &mut self,
        table: T,
        columns: Vec<C>,
        key_set: K,
        options: Option<ReadOptions>,
    ) -> Result<RowIterator<'_>, Status>
    where
        T: Into<String>,
        C: Into<String>,
        K: Into<KeySet>,
    {
        let opt = match options {
            Some(o) => o,
            None => ReadOptions::default(),
        };

        let request = ReadRequest {
            session: self.get_session_name(),
            transaction: Some(self.transaction_selector.clone()),
            table: table.into(),
            index: opt.index.into(),
            columns: columns.into_iter().map(|x| x.into()).collect(),
            key_set: Some(key_set.into().inner),
            limit: opt.limit,
            resume_token: vec![],
            partition_token: vec![],
            request_options: Transaction::create_request_options(opt.call_options.priority),
        };

        let session = self.as_mut_session();
        return RowIterator::new(
            session,
            Box::new(TableReader {
                request,
                call_setting: opt.call_options.call_setting,
            }),
        )
        .await;
    }

    pub(crate) fn get_session_name(&self) -> String {
        return self.session.as_ref().unwrap().session.name.to_string();
    }

    pub(crate) fn as_mut_session(&mut self) -> &mut ManagedSession {
        return self.session.as_mut().unwrap();
    }

    /// returns the owner ship of session.
    /// must drop destroy after this method.
    pub(crate) fn take_session(&mut self) -> Option<ManagedSession> {
        return self.session.take();
    }
}
