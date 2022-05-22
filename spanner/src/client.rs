use google_cloud_gax::retry::{invoke_fn, TryAs};
use google_cloud_googleapis::spanner::v1::{commit_request, transaction_options, Mutation, TransactionOptions};

use crate::apiv1::conn_pool::ConnectionManager;
use crate::session::{ManagedSession, SessionConfig, SessionError, SessionManager};
use crate::statement::Statement;
use crate::transaction::{CallOptions, QueryOptions};
use crate::transaction_ro::{BatchReadOnlyTransaction, ReadOnlyTransaction};
use crate::transaction_rw::{commit, CommitOptions, ReadWriteTransaction};
use crate::value::{Timestamp, TimestampBound};

use crate::retry::TransactionRetrySetting;
use google_cloud_gax::cancel::CancellationToken;
use google_cloud_gax::conn::Environment;
use google_cloud_gax::grpc::{Code, Status};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

#[derive(Clone)]
pub struct PartitionedUpdateOption {
    pub begin_options: CallOptions,
    pub query_options: Option<QueryOptions>,
}

impl Default for PartitionedUpdateOption {
    fn default() -> Self {
        PartitionedUpdateOption {
            begin_options: CallOptions::default(),
            query_options: None,
        }
    }
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

#[derive(Clone)]
pub struct ReadWriteTransactionOption {
    pub begin_options: CallOptions,
    pub commit_options: CommitOptions,
}

impl Default for ReadWriteTransactionOption {
    fn default() -> Self {
        ReadWriteTransactionOption {
            begin_options: CallOptions::default(),
            commit_options: CommitOptions::default(),
        }
    }
}

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
pub struct ClientConfig {
    /// SessionPoolConfig is the configuration for session pool.
    pub session_config: SessionConfig,
    /// ChannelConfig is the configuration for gRPC connection.
    pub channel_config: ChannelConfig,
}

impl Default for ClientConfig {
    fn default() -> Self {
        let mut config = ClientConfig {
            channel_config: Default::default(),
            session_config: Default::default(),
        };
        config.session_config.min_opened = config.channel_config.num_channels * 4;
        config.session_config.max_opened = config.channel_config.num_channels * 100;
        config
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
    fn try_as(&self) -> Result<&Status, ()> {
        match self {
            TxError::GRPC(s) => Ok(s),
            _ => Err(()),
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

impl TryAs<Status> for RunInTxError {
    fn try_as(&self) -> Result<&Status, ()> {
        match self {
            RunInTxError::GRPC(e) => Ok(e),
            _ => Err(()),
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
        return Client::new_with_config(database, Default::default()).await;
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

        let environment = match std::env::var("SPANNER_EMULATOR_HOST") {
            Ok(host) => Environment::Emulator(host),
            Err(_) => Environment::GoogleCloud(google_cloud_auth::project().await?),
        };
        let pool_size = config.channel_config.num_channels as usize;
        let conn_pool = ConnectionManager::new(pool_size, &environment).await?;
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
    pub async fn single(&self) -> Result<ReadOnlyTransaction, TxError> {
        return self.single_with_timestamp_bound(TimestampBound::strong_read()).await;
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
    pub async fn read_only_transaction(&self) -> Result<ReadOnlyTransaction, TxError> {
        return self
            .read_only_transaction_with_option(ReadOnlyTransactionOption::default())
            .await;
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
        return self
            .batch_read_only_transaction_with_option(ReadOnlyTransactionOption::default())
            .await;
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
        return self
            .partitioned_update_with_option(stmt, PartitionedUpdateOption::default())
            .await;
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
        return invoke_fn(
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
        .await;
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
        return self.apply_at_least_once_with_option(ms, CommitOptions::default()).await;
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

        return invoke_fn(
            options.call_options.cancel.clone(),
            Some(ro),
            |session| async {
                let tx = commit_request::Transaction::SingleUseTransaction(TransactionOptions {
                    mode: Some(transaction_options::Mode::ReadWrite(transaction_options::ReadWrite {})),
                });
                match commit(session, ms.clone(), tx, options.clone()).await {
                    Ok(s) => Ok(match s.commit_timestamp {
                        Some(s) => Some(s.into()),
                        None => None,
                    }),
                    Err(e) => Err((TxError::GRPC(e), session)),
                }
            },
            &mut session,
        )
        .await;
    }

    /// Apply applies a list of mutations atomically to the database.
    pub async fn apply(&self, ms: Vec<Mutation>) -> Result<Option<Timestamp>, TxError> {
        return self.apply_with_option(ms, ReadWriteTransactionOption::default()).await;
    }

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
    pub async fn read_write_transaction<'a, T, E, F>(&self, f: F) -> Result<(Option<Timestamp>, T), E>
    where
        E: TryAs<Status> + From<SessionError> + From<Status>,
        F: for<'tx> Fn(
            &'tx mut ReadWriteTransaction,
            Option<CancellationToken>,
        ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'tx>>,
    {
        return self
            .read_write_transaction_with_option(f, ReadWriteTransactionOption::default())
            .await;
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
        return invoke_fn(
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
        .await;
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
        return invoke_fn(
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
        .await;
    }

    async fn create_read_write_transaction<E>(
        &self,
        session: Option<ManagedSession>,
        bo: CallOptions,
    ) -> Result<ReadWriteTransaction, (E, Option<ManagedSession>)>
    where
        E: TryAs<Status> + From<SessionError> + From<Status>,
    {
        return ReadWriteTransaction::begin(session.unwrap(), bo)
            .await
            .map_err(|e| (E::from(e.status), Some(e.session)));
    }

    async fn get_session(&self) -> Result<ManagedSession, SessionError> {
        return self.sessions.get().await;
    }

    fn split_read_write_transaction_option(options: ReadWriteTransactionOption) -> (CallOptions, CommitOptions) {
        (options.begin_options, options.commit_options)
    }
}
