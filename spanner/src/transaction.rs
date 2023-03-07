use std::ops::DerefMut;
use std::sync::atomic::AtomicI64;

use prost_types::Struct;

use google_cloud_gax::cancel::CancellationToken;
use google_cloud_gax::grpc::Status;
use google_cloud_gax::retry::RetrySetting;
use google_cloud_googleapis::spanner::v1::request_options::Priority;
use google_cloud_googleapis::spanner::v1::{
    execute_sql_request::QueryMode, execute_sql_request::QueryOptions as ExecuteQueryOptions, ExecuteSqlRequest,
    ReadRequest, RequestOptions, TransactionSelector,
};

use crate::key::{Key, KeySet};
use crate::reader::{AsyncIterator, RowIterator, StatementReader, TableReader};
use crate::row::Row;
use crate::session::ManagedSession;
use crate::statement::Statement;

#[derive(Clone, Default)]
pub struct CallOptions {
    /// Priority is the RPC priority to use for the read operation.
    pub priority: Option<Priority>,
    pub retry: Option<RetrySetting>,
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
        ReadOptions {
            index: "".to_string(),
            limit: 0,
            call_options: CallOptions::default(),
        }
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
        QueryOptions {
            mode: QueryMode::Normal,
            optimizer_options: None,
            call_options: CallOptions::default(),
        }
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
        priority.map(|s| RequestOptions {
            priority: s.into(),
            request_tag: "".to_string(),
            transaction_tag: "".to_string(),
        })
    }

    /// query executes a query against the database. It returns a RowIterator for
    /// retrieving the resulting rows.
    ///
    /// query returns only row data, without a query plan or execution statistics.
    pub async fn query(&mut self, statement: Statement) -> Result<RowIterator<'_>, Status> {
        self.query_with_option(statement, QueryOptions::default()).await
    }

    /// query executes a query against the database. It returns a RowIterator for
    /// retrieving the resulting rows.
    ///
    /// query returns only row data, without a query plan or execution statistics.
    pub async fn query_with_option(
        &mut self,
        statement: Statement,
        options: QueryOptions,
    ) -> Result<RowIterator<'_>, Status> {
        let request = ExecuteSqlRequest {
            session: self.session.as_ref().unwrap().session.name.to_string(),
            transaction: Some(self.transaction_selector.clone()),
            sql: statement.sql,
            params: Some(Struct {
                fields: statement.params,
            }),
            param_types: statement.param_types,
            resume_token: vec![],
            query_mode: options.mode.into(),
            partition_token: vec![],
            seqno: 0,
            query_options: options.optimizer_options,
            request_options: Transaction::create_request_options(options.call_options.priority),
        };
        let session = self.session.as_mut().unwrap().deref_mut();
        let reader = Box::new(StatementReader { request });
        RowIterator::new(session, reader, Some(options.call_options)).await
    }

    /// read returns a RowIterator for reading multiple rows from the database.
    /// ```
    /// use google_cloud_spanner::key::Key;
    /// use google_cloud_spanner::client::{Client, Error};
    /// use google_cloud_spanner::reader::AsyncIterator;
    ///
    /// #[tokio::main]
    /// async fn run(client: Client) -> Result<(), Error> {
    ///     let mut tx = client.single().await?;
    ///     let mut iter = tx.read("Guild", &["GuildID", "OwnerUserID"], vec![
    ///         Key::new(&"pk1"),
    ///         Key::new(&"pk2")
    ///     ]).await?;
    ///
    ///     while let Some(row) = iter.next().await? {
    ///         let guild_id = row.column_by_name::<String>("GuildID");
    ///         //do something
    ///     };
    ///     Ok(())
    /// }
    /// ```
    pub async fn read(
        &mut self,
        table: &str,
        columns: &[&str],
        key_set: impl Into<KeySet>,
    ) -> Result<RowIterator<'_>, Status> {
        self.read_with_option(table, columns, key_set, ReadOptions::default())
            .await
    }

    /// read returns a RowIterator for reading multiple rows from the database.
    pub async fn read_with_option(
        &mut self,
        table: &str,
        columns: &[&str],
        key_set: impl Into<KeySet>,
        options: ReadOptions,
    ) -> Result<RowIterator<'_>, Status> {
        let request = ReadRequest {
            session: self.get_session_name(),
            transaction: Some(self.transaction_selector.clone()),
            table: table.to_string(),
            index: options.index,
            columns: columns.iter().map(|x| x.to_string()).collect(),
            key_set: Some(key_set.into().inner),
            limit: options.limit,
            resume_token: vec![],
            partition_token: vec![],
            request_options: Transaction::create_request_options(options.call_options.priority),
        };

        let session = self.as_mut_session();
        let reader = Box::new(TableReader { request });
        RowIterator::new(session, reader, Some(options.call_options)).await
    }

    /// read returns a RowIterator for reading multiple rows from the database.
    /// ```
    /// use google_cloud_spanner::key::Key;
    /// use google_cloud_spanner::client::Client;
    /// use google_cloud_spanner::client::Error;
    ///
    /// async fn run(client: Client) -> Result<(), Error> {
    ///     let mut tx = client.single().await?;
    ///     let row = tx.read_row("Guild", &["GuildID", "OwnerUserID"], Key::new(&"guild1")).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn read_row(&mut self, table: &str, columns: &[&str], key: Key) -> Result<Option<Row>, Status> {
        self.read_row_with_option(table, columns, key, ReadOptions::default())
            .await
    }

    /// read returns a RowIterator for reading multiple rows from the database.
    pub async fn read_row_with_option(
        &mut self,
        table: &str,
        columns: &[&str],
        key: Key,
        options: ReadOptions,
    ) -> Result<Option<Row>, Status> {
        let call_options = options.call_options.clone();
        let mut reader = self
            .read_with_option(table, columns, KeySet::from(key), options)
            .await?;
        reader.set_call_options(call_options);
        reader.next().await
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
        self.session.take()
    }
}
