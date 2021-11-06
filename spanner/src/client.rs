use std::future::Future;

use prost_types::Timestamp;
use tonic::{Code, Status};

use google_cloud_gax::invoke::invoke_reuse;
use google_cloud_gax::invoke::AsTonicStatus;
use google_cloud_googleapis::spanner::v1::{
    commit_request, transaction_options, Mutation, TransactionOptions,
};

use crate::apiv1::conn_pool::ConnectionManager;
use crate::retry::{new_default_tx_retry, new_tx_retry_with_codes};
use crate::sessions::{ManagedSession, SessionConfig, SessionError, SessionManager};
use crate::statement::Statement;
use crate::transaction::{CallOptions, QueryOptions};
use crate::transaction_ro::{BatchReadOnlyTransaction, ReadOnlyTransaction};
use crate::transaction_rw::{commit, CommitOptions, ReadWriteTransaction};
use crate::value::TimestampBound;

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

#[derive(Clone)]
pub struct ApplyOptions {
    pub commit_options: CommitOptions,
}

impl Default for ApplyOptions {
    fn default() -> Self {
        ApplyOptions {
            commit_options: CommitOptions::default(),
        }
    }
}

/// Client is a client for reading and writing data to a Cloud Spanner database.
/// A client is safe to use concurrently, except for its Close method.
pub struct Client {
    sessions: SessionManager,
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
        config.session_config.max_opened = config.channel_config.num_channels * 100;
        config
    }
}

#[derive(thiserror::Error, Debug)]
pub enum InitializeError {
    #[error(transparent)]
    TonicStatus(#[from] Status),

    #[error(transparent)]
    GRPCInitialize(#[from] crate::apiv1::conn_pool::Error),

    #[error("invalid config: {0}")]
    InvalidConfig(String),
}

#[derive(thiserror::Error, Debug)]
pub enum TxError {
    #[error(transparent)]
    TonicStatus(#[from] Status),

    #[error(transparent)]
    SessionError(#[from] SessionError),
}

impl AsTonicStatus for TxError {
    fn as_tonic_status(&self) -> Option<&Status> {
        match self {
            TxError::TonicStatus(s) => Some(s),
            _ => None,
        }
    }
}

impl Client {
    /// new creates a client to a database. A valid database name has
    /// the form projects/PROJECT_ID/instances/INSTANCE_ID/databases/DATABASE_ID.
    pub async fn new(
        database: impl Into<String>,
        options: Option<ClientConfig>,
    ) -> Result<Self, InitializeError> {
        let config = match options {
            Some(o) => o,
            None => Default::default(),
        };

        if config.session_config.max_opened > config.channel_config.num_channels * 100 {
            return Err(InitializeError::InvalidConfig(format!(
                "max session size is {} because max session size is 100 per gRPC connection",
                config.channel_config.num_channels * 100
            )));
        }

        let pool_size = config.channel_config.num_channels as usize;
        let emulator_host = match std::env::var("SPANNER_EMULATOR_HOST") {
            Ok(s) => Some(s),
            Err(_) => None,
        };
        let conn_pool = ConnectionManager::new(pool_size, emulator_host).await?;
        let session_manager =
            SessionManager::new(database, conn_pool, config.session_config).await?;
        session_manager.schedule_refresh();

        Ok(Client {
            sessions: session_manager,
        })
    }

    /// Close closes the client.
    pub async fn close(&mut self) {
        self.sessions.close().await;
    }

    /// single provides a read-only snapshot transaction optimized for the case
    /// where only a single read or query is needed.  This is more efficient than
    /// using read_only_transaction for a single read or query.
    pub async fn single(&self, tb: Option<TimestampBound>) -> Result<ReadOnlyTransaction, TxError> {
        let tb = match tb {
            Some(tb) => tb,
            None => TimestampBound::strong_read(),
        };
        let session = self.get_session().await?;
        let result = ReadOnlyTransaction::single(session, tb).await?;
        Ok(result)
    }

    /// read_only_transaction returns a ReadOnlyTransaction that can be used for
    /// multiple reads from the database.
    pub async fn read_only_transaction(
        &self,
        options: Option<ReadOnlyTransactionOption>,
    ) -> Result<ReadOnlyTransaction, TxError> {
        let opt = match options {
            Some(o) => o,
            None => ReadOnlyTransactionOption::default(),
        };
        let session = self.get_session().await?;
        let result =
            ReadOnlyTransaction::begin(session, opt.timestamp_bound, opt.call_options).await?;
        Ok(result)
    }

    /// batch_read_only_transaction returns a BatchReadOnlyTransaction that can be used
    /// for partitioned reads or queries from a snapshot of the database. This is
    /// useful in batch processing pipelines where one wants to divide the work of
    /// reading from the database across multiple machines.
    pub async fn batch_read_only_transaction(
        &self,
        options: Option<ReadOnlyTransactionOption>,
    ) -> Result<BatchReadOnlyTransaction, TxError> {
        let opt = match options {
            Some(o) => o,
            None => ReadOnlyTransactionOption::default(),
        };

        let session = self.get_session().await?;
        let result =
            BatchReadOnlyTransaction::begin(session, opt.timestamp_bound, opt.call_options).await?;
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
    pub async fn partitioned_update(
        &self,
        stmt: Statement,
        options: Option<PartitionedUpdateOption>,
    ) -> Result<i64, TxError> {
        let (bo, qo) = match options {
            Some(o) => (o.begin_options, o.query_options),
            None => {
                let o = PartitionedUpdateOption::default();
                (o.begin_options, o.query_options)
            }
        };

        let mut ro = new_tx_retry_with_codes(vec![Code::Aborted, Code::Internal]);
        let session = Some(self.get_session().await?);

        // reuse session
        return invoke_reuse(
            |session| async {
                let mut tx =
                    match ReadWriteTransaction::begin_partitioned_dml(session.unwrap(), bo.clone())
                        .await
                    {
                        Ok(tx) => tx,
                        Err(e) => return Err((TxError::TonicStatus(e.status), Some(e.session))),
                    };
                match tx.update(stmt.clone(), qo.clone()).await {
                    Ok(s) => Ok(s),
                    Err(e) => Err((TxError::TonicStatus(e), tx.take_session())),
                }
            },
            session,
            &mut ro,
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
    pub async fn apply_at_least_once(
        &self,
        ms: Vec<Mutation>,
        options: Option<ApplyOptions>,
    ) -> Result<Option<prost_types::Timestamp>, TxError> {
        let co = match options {
            Some(s) => s.commit_options,
            None => CommitOptions::default(),
        };
        let mut ro = new_default_tx_retry();
        let mut session = self.get_session().await?;

        return invoke_reuse(
            |session| async {
                let tx = commit_request::Transaction::SingleUseTransaction(TransactionOptions {
                    mode: Some(transaction_options::Mode::ReadWrite(
                        transaction_options::ReadWrite {},
                    )),
                });
                match commit(session, ms.clone(), tx, co.clone()).await {
                    Ok(s) => Ok(s.commit_timestamp),
                    Err(e) => Err((TxError::TonicStatus(e), session)),
                }
            },
            &mut session,
            &mut ro,
        )
        .await;
    }

    /// Apply applies a list of mutations atomically to the database.
    pub async fn apply(
        &self,
        ms: Vec<Mutation>,
        options: Option<ReadWriteTransactionOption>,
    ) -> Result<Option<Timestamp>, TxError> {
        let result: Result<(Option<Timestamp>, ()), TxError> = self
            .read_write_transaction_sync(
                |tx| {
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
    /// See https://godoc.org/cloud.google.com/go/spanner#ReadWriteTransaction for
    /// more details.
    /// ```
    pub async fn read_write_transaction<'a, T, E, F>(
        &self,
        f: impl Fn(ReadWriteTransaction) -> F,
        options: Option<ReadWriteTransactionOption>,
    ) -> Result<(Option<prost_types::Timestamp>, T), E>
    where
        E: AsTonicStatus + From<TxError> + From<tonic::Status>,
        F: Future<Output = (ReadWriteTransaction, Result<T, E>)>,
    {
        let (bo, co) = Client::split_read_write_transaction_option(options);

        let mut ro = new_default_tx_retry();
        let session = Some(self.get_session().await?);

        // must reuse session
        return invoke_reuse(
            |session| async {
                let tx = self
                    .create_read_write_transaction::<E>(session, bo.clone())
                    .await?;
                let (mut tx, result) = f(tx).await;
                tx.finish(result, Some(co.clone())).await
            },
            session,
            &mut ro,
        )
        .await;
    }

    pub async fn read_write_transaction_sync<T, E>(
        &self,
        f: impl Fn(&mut ReadWriteTransaction) -> Result<T, E>,
        options: Option<ReadWriteTransactionOption>,
    ) -> Result<(Option<prost_types::Timestamp>, T), E>
    where
        E: AsTonicStatus + From<TxError> + From<tonic::Status>,
    {
        let (bo, co) = Client::split_read_write_transaction_option(options);

        let mut ro = new_default_tx_retry();
        let session = Some(self.get_session().await?);

        // reuse session
        return invoke_reuse(
            |session| async {
                let mut tx = self
                    .create_read_write_transaction::<E>(session, bo.clone())
                    .await?;
                let result = f(&mut tx);
                tx.finish(result, Some(co.clone())).await
            },
            session,
            &mut ro,
        )
        .await;
    }

    async fn create_read_write_transaction<E>(
        &self,
        session: Option<ManagedSession>,
        bo: CallOptions,
    ) -> Result<ReadWriteTransaction, (E, Option<ManagedSession>)>
    where
        E: AsTonicStatus + From<TxError> + From<tonic::Status>,
    {
        return ReadWriteTransaction::begin(session.unwrap(), bo)
            .await
            .map_err(|e| (E::from(e.status), Some(e.session)));
    }

    async fn get_session(&self) -> Result<ManagedSession, TxError> {
        return self.sessions.get().await.map_err(TxError::SessionError);
    }

    fn split_read_write_transaction_option(
        options: Option<ReadWriteTransactionOption>,
    ) -> (CallOptions, CommitOptions) {
        match options {
            Some(s) => (s.begin_options, s.commit_options),
            None => {
                let s = ReadWriteTransactionOption::default();
                (s.begin_options, s.commit_options)
            }
        }
    }
}
