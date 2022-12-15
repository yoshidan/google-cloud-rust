use google_cloud_gax::retry::{invoke_fn, TryAs};
use google_cloud_googleapis::spanner::v1::{commit_request, transaction_options, Mutation, TransactionOptions};

use crate::apiv1::conn_pool::{ConnectionManager, SPANNER};
use crate::session::{ManagedSession, SessionConfig, SessionError, SessionManager};
use crate::statement::Statement;
use crate::transaction::{CallOptions, QueryOptions};
use crate::transaction_ro::{BatchReadOnlyTransaction, ReadOnlyTransaction};
use crate::transaction_rw::{commit, CommitOptions, ReadWriteTransaction};
use crate::value::{Timestamp, TimestampBound};

use crate::retry::TransactionRetrySetting;

use google_cloud_auth::Project;
use google_cloud_gax::cancel::CancellationToken;
use google_cloud_gax::conn::Environment;
use google_cloud_gax::grpc::{Code, Status};
use google_cloud_gax::project::ProjectOptions;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct PartitionedUpdateOption {
    pub begin_options: CallOptions,
    pub query_options: Option<QueryOptions>,
}

#[derive(Clone)]
pub struct ReadOnlyTransactionOption {
    pub timestamp_bound: TimestampBound,
    pub call_options: CallOptions,
}

impl Default for ReadOnlyTransactionOption {
    fn default() -> Self {
        ReadOnlyTransactionOption {
            timestamp_bound: TimestampBound::strong_read(),
            call_options: CallOptions::default(),
        }
    }
}

#[derive(Clone, Default)]
pub struct ReadWriteTransactionOption {
    pub begin_options: CallOptions,
    pub commit_options: CommitOptions,
}

#[derive(Clone, Debug)]
pub struct ChannelConfig {
    /// num_channels is the number of gRPC channels.
    pub num_channels: usize,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        ChannelConfig { num_channels: 4 }
    }
}

/// ClientConfig has configurations for the client.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// SessionPoolConfig is the configuration for session pool.
    pub session_config: SessionConfig,
    /// ChannelConfig is the configuration for gRPC connection.
    pub channel_config: ChannelConfig,
    /// Overriding service endpoint
    pub endpoint: String,
    /// Runtime project
    pub project: ProjectOptions,
}

impl Default for ClientConfig {
    fn default() -> Self {
        let mut config = ClientConfig {
            channel_config: Default::default(),
            session_config: Default::default(),
            endpoint: SPANNER.to_string(),
            project: ProjectOptions::new("SPANNER_EMULATOR_HOST"),
        };
        config.session_config.min_opened = config.channel_config.num_channels * 4;
        config.session_config.max_opened = config.channel_config.num_channels * 100;
        config
    }
}

impl ClientConfig {
    pub fn project(&mut self, project: Project) {
        if let ProjectOptions::Project(_) = self.project {
            self.project = ProjectOptions::Project(Some(project))
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum InitializationError {
    #[error(transparent)]
    FailedToCreateSessionPool(#[from] Status),

    #[error(transparent)]
    FailedToCreateChannelPool(#[from] google_cloud_gax::conn::Error),

    #[error(transparent)]
    Auth(#[from] google_cloud_auth::error::Error),

    #[error("invalid config: {0}")]
    InvalidConfig(String),
}

#[derive(thiserror::Error, Debug)]
pub enum TxError {
    #[error(transparent)]
    GRPC(#[from] Status),

    #[error(transparent)]
    InvalidSession(#[from] SessionError),
}

impl TryAs<Status> for TxError {
    fn try_as(&self) -> Option<&Status> {
        match self {
            TxError::GRPC(s) => Some(s),
            _ => None,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum RunInTxError {
    #[error(transparent)]
    GRPC(#[from] Status),

    #[error(transparent)]
    InvalidSession(#[from] SessionError),

    #[error(transparent)]
    ParseError(#[from] crate::row::Error),

    #[error(transparent)]
    Any(#[from] anyhow::Error),
}

impl From<TxError> for RunInTxError {
    fn from(err: TxError) -> Self {
        match err {
            TxError::GRPC(err) => RunInTxError::GRPC(err),
            TxError::InvalidSession(err) => RunInTxError::InvalidSession(err),
        }
    }
}

impl TryAs<Status> for RunInTxError {
    fn try_as(&self) -> Option<&Status> {
        match self {
            RunInTxError::GRPC(e) => Some(e),
            _ => None,
        }
    }
}

/// Client is a client for reading and writing data to a Cloud Spanner database.
/// A client is safe to use concurrently, except for its Close method.
pub struct Client {
    sessions: Arc<SessionManager>,
}

impl Clone for Client {
    fn clone(&self) -> Self {
        Client {
            sessions: Arc::clone(&self.sessions),
        }
    }
}

impl Client {
    /// new creates a client to a database. A valid database name has
    /// the form projects/PROJECT_ID/instances/INSTANCE_ID/databases/DATABASE_ID.
    pub async fn new(database: impl Into<String>) -> Result<Self, InitializationError> {
        Client::new_with_config(database, Default::default()).await
    }

    /// new creates a client to a database. A valid database name has
    /// the form projects/PROJECT_ID/instances/INSTANCE_ID/databases/DATABASE_ID.
    pub async fn new_with_config(
        database: impl Into<String>,
        config: ClientConfig,
    ) -> Result<Self, InitializationError> {
        if config.session_config.max_opened > config.channel_config.num_channels * 100 {
            return Err(InitializationError::InvalidConfig(format!(
                "max session size is {} because max session size is 100 per gRPC connection",
                config.channel_config.num_channels * 100
            )));
        }

        let environment = Environment::from_project(config.project).await?;
        let pool_size = config.channel_config.num_channels as usize;
        let conn_pool = ConnectionManager::new(pool_size, &environment, config.endpoint.as_str()).await?;
        let session_manager = SessionManager::new(database, conn_pool, config.session_config).await?;

        Ok(Client {
            sessions: Arc::new(session_manager),
        })
    }

    /// Close closes the client.
    pub async fn close(&self) {
        self.sessions.close().await;
    }

    /// single provides a read-only snapshot transaction optimized for the case
    /// where only a single read or query is needed.  This is more efficient than
    /// using read_only_transaction for a single read or query.
    /// ```
    /// use google_cloud_spanner::key::Key;
    /// use google_cloud_spanner::statement::ToKind;
    /// use google_cloud_spanner::client::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), anyhow::Error> {
    ///     const DATABASE: &str = "projects/local-project/instances/test-instance/databases/local-database";
    ///     let client = Client::new(DATABASE).await?;
    ///
    ///     let mut tx = client.single().await?;
    ///     let iter1 = tx.read("Guild",&["GuildID", "OwnerUserID"], vec![
    ///         Key::new(&"pk1"),
    ///         Key::new(&"pk2")
    ///     ]).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn single(&self) -> Result<ReadOnlyTransaction, TxError> {
        self.single_with_timestamp_bound(TimestampBound::strong_read()).await
    }

    /// single provides a read-only snapshot transaction optimized for the case
    /// where only a single read or query is needed.  This is more efficient than
    /// using read_only_transaction for a single read or query.
    pub async fn single_with_timestamp_bound(&self, tb: TimestampBound) -> Result<ReadOnlyTransaction, TxError> {
        let session = self.get_session().await?;
        let result = ReadOnlyTransaction::single(session, tb).await?;
        Ok(result)
    }

    /// read_only_transaction returns a ReadOnlyTransaction that can be used for
    /// multiple reads from the database.
    ///
    /// ```ignore
    /// use google_cloud_spanner::statement::Statement;
    /// use google_cloud_spanner::key::Key;
    ///
    /// let tx = client.read_only_transaction().await?;
    ///
    /// let mut stmt = Statement::new("SELECT * , \
    ///             ARRAY (SELECT AS STRUCT * FROM UserItem WHERE UserId = @Param1 ) AS UserItem, \
    ///             ARRAY (SELECT AS STRUCT * FROM UserCharacter WHERE UserId = @Param1 ) AS UserCharacter  \
    ///             FROM User \
    ///             WHERE UserId = @Param1");
    ///
    /// stmt.add_param("Param1", user_id);
    /// let mut reader = tx.query(stmt).await?;
    /// while let Some(row) = reader.next().await? {
    ///     let user_id= row.column_by_name::<String>("UserId")?;
    ///     let user_items= row.column_by_name::<Vec<model::UserItem>>("UserItem")?;
    ///     let user_characters = row.column_by_name::<Vec<model::UserCharacter>>("UserCharacter")?;
    ///     data.push(user_id);
    /// }
    ///
    /// let mut reader2 = tx.read("User", &["UserId"], vec![
    ///     Key::new(&"user-1"),
    ///     Key::new(&"user-2")
    /// ]).await?;
    pub async fn read_only_transaction(&self) -> Result<ReadOnlyTransaction, TxError> {
        self.read_only_transaction_with_option(ReadOnlyTransactionOption::default())
            .await
    }

    /// read_only_transaction returns a ReadOnlyTransaction that can be used for
    /// multiple reads from the database.
    pub async fn read_only_transaction_with_option(
        &self,
        options: ReadOnlyTransactionOption,
    ) -> Result<ReadOnlyTransaction, TxError> {
        let session = self.get_session().await?;
        let result = ReadOnlyTransaction::begin(session, options.timestamp_bound, options.call_options).await?;
        Ok(result)
    }

    /// batch_read_only_transaction returns a BatchReadOnlyTransaction that can be used
    /// for partitioned reads or queries from a snapshot of the database. This is
    /// useful in batch processing pipelines where one wants to divide the work of
    /// reading from the database across multiple machines.
    pub async fn batch_read_only_transaction(&self) -> Result<BatchReadOnlyTransaction, TxError> {
        self.batch_read_only_transaction_with_option(ReadOnlyTransactionOption::default())
            .await
    }

    /// batch_read_only_transaction returns a BatchReadOnlyTransaction that can be used
    /// for partitioned reads or queries from a snapshot of the database. This is
    /// useful in batch processing pipelines where one wants to divide the work of
    /// reading from the database across multiple machines.
    pub async fn batch_read_only_transaction_with_option(
        &self,
        options: ReadOnlyTransactionOption,
    ) -> Result<BatchReadOnlyTransaction, TxError> {
        let session = self.get_session().await?;
        let result = BatchReadOnlyTransaction::begin(session, options.timestamp_bound, options.call_options).await?;
        Ok(result)
    }

    /// partitioned_update executes a DML statement in parallel across the database,
    /// using separate, internal transactions that commit independently. The DML
    /// statement must be fully partitionable: it must be expressible as the union
    /// of many statements each of which accesses only a single row of the table. The
    /// statement should also be idempotent, because it may be applied more than once.
    ///
    /// PartitionedUpdate returns an estimated count of the number of rows affected.
    /// The actual number of affected rows may be greater than the estimate.
    pub async fn partitioned_update(&self, stmt: Statement) -> Result<i64, TxError> {
        self.partitioned_update_with_option(stmt, PartitionedUpdateOption::default())
            .await
    }

    /// partitioned_update executes a DML statement in parallel across the database,
    /// using separate, internal transactions that commit independently. The DML
    /// statement must be fully partitionable: it must be expressible as the union
    /// of many statements each of which accesses only a single row of the table. The
    /// statement should also be idempotent, because it may be applied more than once.
    ///
    /// PartitionedUpdate returns an estimated count of the number of rows affected.
    /// The actual number of affected rows may be greater than the estimate.
    pub async fn partitioned_update_with_option(
        &self,
        stmt: Statement,
        options: PartitionedUpdateOption,
    ) -> Result<i64, TxError> {
        let ro = TransactionRetrySetting::new(vec![Code::Aborted, Code::Internal]);
        let session = Some(self.get_session().await?);

        // reuse session
        invoke_fn(
            options.begin_options.cancel.clone(),
            Some(ro),
            |session| async {
                let mut tx =
                    match ReadWriteTransaction::begin_partitioned_dml(session.unwrap(), options.begin_options.clone())
                        .await
                    {
                        Ok(tx) => tx,
                        Err(e) => return Err((TxError::GRPC(e.status), Some(e.session))),
                    };
                let qo = match options.query_options.clone() {
                    Some(o) => o,
                    None => QueryOptions::default(),
                };
                tx.update_with_option(stmt.clone(), qo)
                    .await
                    .map_err(|e| (TxError::GRPC(e), tx.take_session()))
            },
            session,
        )
        .await
    }

    /// apply_at_least_once may attempt to apply mutations more than once; if
    /// the mutations are not idempotent, this may lead to a failure being reported
    /// when the mutation was applied more than once. For example, an insert may
    /// fail with ALREADY_EXISTS even though the row did not exist before Apply was
    /// called. For this reason, most users of the library will prefer not to use
    /// this option.  However, apply_at_least_once requires only a single RPC, whereas
    /// apply's default replay protection may require an additional RPC.  So this
    /// method may be appropriate for latency sensitive and/or high throughput blind
    /// writing.
    pub async fn apply_at_least_once(&self, ms: Vec<Mutation>) -> Result<Option<Timestamp>, TxError> {
        self.apply_at_least_once_with_option(ms, CommitOptions::default()).await
    }

    /// apply_at_least_once may attempt to apply mutations more than once; if
    /// the mutations are not idempotent, this may lead to a failure being reported
    /// when the mutation was applied more than once. For example, an insert may
    /// fail with ALREADY_EXISTS even though the row did not exist before Apply was
    /// called. For this reason, most users of the library will prefer not to use
    /// this option.  However, apply_at_least_once requires only a single RPC, whereas
    /// apply's default replay protection may require an additional RPC.  So this
    /// method may be appropriate for latency sensitive and/or high throughput blind
    /// writing.
    pub async fn apply_at_least_once_with_option(
        &self,
        ms: Vec<Mutation>,
        options: CommitOptions,
    ) -> Result<Option<Timestamp>, TxError> {
        let ro = TransactionRetrySetting::default();
        let mut session = self.get_session().await?;

        invoke_fn(
            options.call_options.cancel.clone(),
            Some(ro),
            |session| async {
                let tx = commit_request::Transaction::SingleUseTransaction(TransactionOptions {
                    mode: Some(transaction_options::Mode::ReadWrite(transaction_options::ReadWrite::default())),
                });
                match commit(session, ms.clone(), tx, options.clone()).await {
                    Ok(s) => Ok(s.commit_timestamp.map(|s| s.into())),
                    Err(e) => Err((TxError::GRPC(e), session)),
                }
            },
            &mut session,
        )
        .await
    }

    /// Apply applies a list of mutations atomically to the database.
    /// ```
    /// use google_cloud_spanner::mutation::insert;
    /// use google_cloud_spanner::mutation::delete;
    /// use google_cloud_spanner::key::all_keys;
    /// use google_cloud_spanner::statement::ToKind;
    /// use google_cloud_spanner::client::Client;
    /// use google_cloud_spanner::value::CommitTimestamp;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), anyhow::Error> {
    ///     const DATABASE: &str = "projects/local-project/instances/test-instance/databases/local-database";
    ///     let client = Client::new(DATABASE).await?;
    ///     let m1 = delete("Guild", all_keys());
    ///     let m2 = insert("Guild", &["GuildID", "OwnerUserID", "UpdatedAt"], &[&"1", &"2", &CommitTimestamp::new()]);
    ///     let commit_timestamp = client.apply(vec![m1,m2]).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn apply(&self, ms: Vec<Mutation>) -> Result<Option<Timestamp>, TxError> {
        self.apply_with_option(ms, ReadWriteTransactionOption::default()).await
    }

    /// Apply applies a list of mutations atomically to the database.
    pub async fn apply_with_option(
        &self,
        ms: Vec<Mutation>,
        options: ReadWriteTransactionOption,
    ) -> Result<Option<Timestamp>, TxError> {
        let result: Result<(Option<Timestamp>, ()), TxError> = self
            .read_write_transaction_sync_with_option(
                |tx, _cancel| {
                    tx.buffer_write(ms.to_vec());
                    Ok(())
                },
                options,
            )
            .await;
        Ok(result?.0)
    }

    /// ReadWriteTransaction executes a read-write transaction, with retries as
    /// necessary.
    ///
    /// The function f will be called one or more times. It must not maintain
    /// any state between calls.
    ///
    /// If the transaction cannot be committed or if f returns an ABORTED error,
    /// ReadWriteTransaction will call f again. It will continue to call f until the
    /// transaction can be committed or the Context times out or is cancelled.  If f
    /// returns an error other than ABORTED, ReadWriteTransaction will abort the
    /// transaction and return the error.
    ///
    /// To limit the number of retries, set a deadline on the Context rather than
    /// using a fixed limit on the number of attempts. ReadWriteTransaction will
    /// retry as needed until that deadline is met.
    ///
    /// See <https://godoc.org/cloud.google.com/go/spanner#ReadWriteTransaction> for
    /// more details.
    /// ```
    /// use google_cloud_spanner::mutation::update;
    /// use google_cloud_spanner::key::{Key, all_keys};
    /// use google_cloud_spanner::value::Timestamp;
    /// use google_cloud_spanner::client::RunInTxError;
    /// use google_cloud_spanner::client::Client;
    /// use google_cloud_spanner::reader::AsyncIterator;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), anyhow::Error> {
    ///     const DATABASE: &str = "projects/local-project/instances/test-instance/databases/local-database";
    ///     let client = Client::new(DATABASE).await?;
    ///     let tx_result: Result<(Option<Timestamp>,()), RunInTxError> = client.read_write_transaction(|tx, _| {
    ///         Box::pin(async move {
    ///             // The transaction function will be called again if the error code
    ///             // of this error is Aborted. The backend may automatically abort
    ///             // any read/write transaction if it detects a deadlock or other problems.
    ///             let key = all_keys();
    ///             let mut reader = tx.read("UserItem", &["UserId", "ItemId", "Quantity"], key).await?;
    ///             let mut ms = vec![];
    ///             while let Some(row) = reader.next().await? {
    ///                 let user_id = row.column_by_name::<String>("UserId")?;
    ///                 let item_id = row.column_by_name::<i64>("ItemId")?;
    ///                 let quantity = row.column_by_name::<i64>("Quantity")? + 1;
    ///                 let m = update("UserItem", &["Quantity"], &[&user_id, &item_id, &quantity]);
    ///                 ms.push(m);
    ///             }
    ///             // The buffered mutation will be committed.  If the commit
    ///             // fails with an Aborted error, this function will be called again
    ///             tx.buffer_write(ms);
    ///             Ok(())
    ///         })
    ///     }).await;
    ///     Ok(())
    /// }
    pub async fn read_write_transaction<'a, T, E, F>(&self, f: F) -> Result<(Option<Timestamp>, T), E>
    where
        E: TryAs<Status> + From<SessionError> + From<Status>,
        F: for<'tx> Fn(
            &'tx mut ReadWriteTransaction,
            Option<CancellationToken>,
        ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'tx>>,
    {
        self.read_write_transaction_with_option(f, ReadWriteTransactionOption::default())
            .await
    }

    /// ReadWriteTransaction executes a read-write transaction, with retries as
    /// necessary.
    ///
    /// The function f will be called one or more times. It must not maintain
    /// any state between calls.
    ///
    /// If the transaction cannot be committed or if f returns an ABORTED error,
    /// ReadWriteTransaction will call f again. It will continue to call f until the
    /// transaction can be committed or the Context times out or is cancelled.  If f
    /// returns an error other than ABORTED, ReadWriteTransaction will abort the
    /// transaction and return the error.
    ///
    /// To limit the number of retries, set a deadline on the Context rather than
    /// using a fixed limit on the number of attempts. ReadWriteTransaction will
    /// retry as needed until that deadline is met.
    ///
    /// See <https://godoc.org/cloud.google.com/go/spanner#ReadWriteTransaction> for
    /// more details.
    pub async fn read_write_transaction_with_option<'a, T, E, F>(
        &'a self,
        f: F,
        options: ReadWriteTransactionOption,
    ) -> Result<(Option<Timestamp>, T), E>
    where
        E: TryAs<Status> + From<SessionError> + From<Status>,
        F: for<'tx> Fn(
            &'tx mut ReadWriteTransaction,
            Option<CancellationToken>,
        ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'tx>>,
    {
        let (bo, co) = Client::split_read_write_transaction_option(options);

        let ro = TransactionRetrySetting::default();
        let session = Some(self.get_session().await?);
        let cancel = bo.cancel.clone();
        // must reuse session
        invoke_fn(
            cancel.clone(),
            Some(ro),
            |session| async {
                let cancel = cancel.clone().map(|v| v.child_token());
                let mut tx = self.create_read_write_transaction::<E>(session, bo.clone()).await?;
                let result = f(&mut tx, cancel).await;
                tx.finish(result, Some(co.clone())).await
            },
            session,
        )
        .await
    }

    /// begin_read_write_transaction creates new ReadWriteTransaction.
    /// ```
    /// use google_cloud_spanner::mutation::update;
    /// use google_cloud_spanner::key::{Key, all_keys};
    /// use google_cloud_spanner::value::Timestamp;
    /// use google_cloud_spanner::client::RunInTxError;
    /// use google_cloud_spanner::client::Client;
    /// use google_cloud_spanner::reader::AsyncIterator;
    /// use google_cloud_spanner::transaction_rw::ReadWriteTransaction;
    /// use google_cloud_googleapis::spanner::v1::execute_batch_dml_request::Statement;
    /// use google_cloud_spanner::retry::TransactionRetry;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), anyhow::Error> {
    ///     const DATABASE: &str = "projects/local-project/instances/test-instance/databases/local-database";
    ///     let client = Client::new(DATABASE).await?;
    ///     let retry = &mut TransactionRetry::new();
    ///     loop {
    ///         let tx = &mut client.begin_read_write_transaction().await?;
    ///
    ///         let result = run_in_transaction(tx).await;
    ///
    ///         // try to commit or rollback transaction.
    ///         match tx.end(result, None).await {
    ///             Ok((_commit_timestamp, success)) => return Ok(success),
    ///             Err(err) => retry.next(err).await? // check retry
    ///         }
    ///     }
    ///     Ok(())
    /// }
    ///
    /// async fn run_in_transaction(tx: &mut ReadWriteTransaction) -> Result<(), RunInTxError> {
    ///     let key = all_keys();
    ///     let mut reader = tx.read("UserItem", &["UserId", "ItemId", "Quantity"], key).await?;
    ///     let mut ms = vec![];
    ///     while let Some(row) = reader.next().await? {
    ///         let user_id = row.column_by_name::<String>("UserId")?;
    ///         let item_id = row.column_by_name::<i64>("ItemId")?;
    ///         let quantity = row.column_by_name::<i64>("Quantity")? + 1;
    ///         let m = update("UserItem", &["UserId", "ItemId", "Quantity"], &[&user_id, &item_id, &quantity]);
    ///         ms.push(m);
    ///     }
    ///     tx.buffer_write(ms);
    ///     Ok(())
    /// }
    /// ```
    pub async fn begin_read_write_transaction(&self) -> Result<ReadWriteTransaction, TxError> {
        let session = self.get_session().await?;
        ReadWriteTransaction::begin(session, ReadWriteTransactionOption::default().begin_options)
            .await
            .map_err(|e| e.status.into())
    }

    /// Get open session count.
    pub fn session_count(&self) -> usize {
        self.sessions.num_opened()
    }

    async fn read_write_transaction_sync_with_option<T, E>(
        &self,
        f: impl Fn(&mut ReadWriteTransaction, Option<CancellationToken>) -> Result<T, E>,
        options: ReadWriteTransactionOption,
    ) -> Result<(Option<Timestamp>, T), E>
    where
        E: TryAs<Status> + From<SessionError> + From<Status>,
    {
        let (bo, co) = Client::split_read_write_transaction_option(options);

        let ro = TransactionRetrySetting::default();
        let session = Some(self.get_session().await?);

        // reuse session
        let cancel = bo.cancel.clone();
        invoke_fn(
            cancel.clone(),
            Some(ro),
            |session| async {
                let cancel = cancel.clone().map(|v| v.child_token());
                let mut tx = self.create_read_write_transaction::<E>(session, bo.clone()).await?;
                let result = f(&mut tx, cancel);
                tx.finish(result, Some(co.clone())).await
            },
            session,
        )
        .await
    }

    async fn create_read_write_transaction<E>(
        &self,
        session: Option<ManagedSession>,
        bo: CallOptions,
    ) -> Result<ReadWriteTransaction, (E, Option<ManagedSession>)>
    where
        E: TryAs<Status> + From<SessionError> + From<Status>,
    {
        ReadWriteTransaction::begin(session.unwrap(), bo)
            .await
            .map_err(|e| (E::from(e.status), Some(e.session)))
    }

    async fn get_session(&self) -> Result<ManagedSession, SessionError> {
        self.sessions.get().await
    }

    fn split_read_write_transaction_option(options: ReadWriteTransactionOption) -> (CallOptions, CommitOptions) {
        (options.begin_options, options.commit_options)
    }
}
