use crate::apiv1::conn_pool::ConnectionManager;
use crate::apiv1::spanner_client::Client as SpannerClient;
use crate::reader;
use crate::reader::AsyncIterator;
use crate::session_pool::{
    ManagedSession, SessionConfig, SessionError, SessionHandle, SessionManager,
};
use crate::statement::Statement;
use crate::statement::ToKind;
use crate::transaction::{CallOptions, QueryOptions, Transaction};
use crate::transaction_ro::{BatchReadOnlyTransaction, ReadOnlyTransaction};
use crate::transaction_rw::{commit, CommitOptions, ReadWriteTransaction};
use crate::value::TimestampBound;
use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, TimeZone, Timelike, Utc, Weekday};
use futures_util::future::BoxFuture;
use google_cloud_gax::call_option::{Backoff, BackoffRetryer, CallSettings, Retryer};
use google_cloud_gax::invoke::AsTonicStatus;
use google_cloud_gax::{call_option, invoke as retryer};
use google_cloud_googleapis::spanner::v1::execute_sql_request::QueryMode;
use google_cloud_googleapis::spanner::v1::mutation::{Operation, Write};
use google_cloud_googleapis::spanner::v1::request_options::Priority;
use google_cloud_googleapis::spanner::v1::transaction_options::Mode::ReadOnly;
use google_cloud_googleapis::spanner::v1::{
    commit_request, request_options, result_set_stats, transaction_options, transaction_selector,
    ExecuteSqlRequest, KeySet, Mutation, RequestOptions, RollbackRequest,
    TransactionOptions as TxOptions, TransactionOptions, TransactionSelector,
};
use prost_types::value::Kind::StringValue;
use prost_types::{value, ListValue, Timestamp, Value};
use std::future::Future;
use std::net::Shutdown::Read;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard, OnceCell};
use tokio::time::{Duration, Instant, Interval};
use tonic::{Code, Status};

#[derive(Clone)]
pub struct PartitionedUpdateOption {
    pub begin_options: CallOptions,
    pub query_options: Option<QueryOptions>,
    pub transaction_retry_setting: CallSettings,
}

impl Default for PartitionedUpdateOption {
    fn default() -> Self {
        PartitionedUpdateOption {
            begin_options: CallOptions::default(),
            query_options: None,
            transaction_retry_setting: {
                let mut o = default_transaction_retry_setting();
                o.retryer.codes = vec![Code::Aborted, Code::Internal];
                o
            },
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
    pub transaction_retry_setting: CallSettings,
    pub begin_options: CallOptions,
    pub commit_options: CommitOptions,
}

impl Default for ReadWriteTransactionOption {
    fn default() -> Self {
        ReadWriteTransactionOption {
            transaction_retry_setting: default_transaction_retry_setting(),
            begin_options: CallOptions::default(),
            commit_options: CommitOptions::default(),
        }
    }
}

#[derive(Clone)]
pub struct ApplyOptions {
    pub transaction_retry_setting: CallSettings,
    pub commit_options: CommitOptions,
}

impl Default for ApplyOptions {
    fn default() -> Self {
        ApplyOptions {
            transaction_retry_setting: default_transaction_retry_setting(),
            commit_options: CommitOptions::default(),
        }
    }
}

fn default_transaction_retry_setting() -> CallSettings {
    CallSettings {
        retryer: BackoffRetryer {
            backoff: Backoff::default(),
            codes: vec![tonic::Code::Aborted],
        },
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
        return config;
    }
}

#[derive(thiserror::Error, Debug)]
pub enum InitializeError {
    #[error(transparent)]
    TonicStatus(#[from] Status),

    #[error(transparent)]
    TonicTransport(#[from] tonic::transport::Error),

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

        let conn_pool = ConnectionManager::new(config.channel_config.num_channels as usize).await?;
        let session_manager =
            SessionManager::new(database, conn_pool, config.session_config).await?;
        session_manager.schedule_refresh();

        return Ok(Client {
            sessions: session_manager,
        });
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
    ///
    /// Note: This transaction does not use the underlying session pool but creates a
    /// new session each time.
    pub async fn batch_read_only_transaction(
        &self,
        options: Option<ReadOnlyTransactionOption>,
    ) -> Result<BatchReadOnlyTransaction, TxError> {
        let opt = match options {
            Some(o) => o,
            None => ReadOnlyTransactionOption::default(),
        };
        //TODO don't use session pool use and create session each time
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
        let (mut ro, bo, qo) = match options {
            Some(o) => (
                o.transaction_retry_setting,
                o.begin_options,
                o.query_options,
            ),
            None => {
                let o = PartitionedUpdateOption::default();
                (
                    o.transaction_retry_setting,
                    o.begin_options,
                    o.query_options,
                )
            }
        };

        return retryer::invoke(
            || async {
                let mut session = self.get_session().await?;
                let mut tx =
                    ReadWriteTransaction::begin_partitioned_dml(session, bo.clone()).await?;
                let result = tx.update(stmt.clone(), qo.clone()).await?;
                Ok(result)
            },
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
        let (mut ro, co) = match options {
            Some(s) => (s.transaction_retry_setting, s.commit_options),
            None => {
                let s = ApplyOptions::default();
                (s.transaction_retry_setting, s.commit_options)
            }
        };
        return retryer::invoke(
            || async {
                let mut session = self.get_session().await?;
                let tx = commit_request::Transaction::SingleUseTransaction(TransactionOptions {
                    mode: Some(transaction_options::Mode::ReadWrite(
                        transaction_options::ReadWrite {},
                    )),
                });
                let commit_result = commit(&mut session, ms.clone(), tx, co.clone()).await?;
                return Ok(commit_result.commit_timestamp);
            },
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
            .read_write_transaction(
                |tx| {
                    let a = ms.to_vec();
                    async move {
                        let mut tx = tx.lock().await;
                        tx.buffer_write(a);
                        Ok(())
                    }
                },
                options,
            )
            .await;
        return Ok(result?.0);
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
    pub async fn read_write_transaction<T, E, F>(
        &self,
        mut f: impl Fn(Arc<Mutex<ReadWriteTransaction>>) -> F,
        options: Option<ReadWriteTransactionOption>,
    ) -> Result<(Option<prost_types::Timestamp>, T), E>
    where
        E: AsTonicStatus + From<TxError> + From<tonic::Status>,
        F: Future<Output = Result<T, E>>,
    {
        let (mut ro, bo, co) = Client::split_read_write_transaction_option(options);

        let backoff = &mut ro.retryer.backoff;
        let mut session = Some(self.get_session().await.map_err(TxError::SessionError)?);

        // reuse session
        loop {
            //run in transaction
            let tx = Arc::new(Mutex::new(
                ReadWriteTransaction::begin(session.take().unwrap(), bo.clone()).await?,
            ));
            let mut result = f(tx.clone()).await;
            let result = async {
                let mut locked = tx.lock().await;
                let result = locked.finish(result, Some(co.clone())).await;
                session = Some(locked.take_session());
                result
            }
            .await;

            if result.is_ok() {
                return result;
            }
            let err = result.err().unwrap();
            let status = match err.as_tonic_status() {
                Some(s) => s,
                None => return Err(err),
            };

            // continue immediate when the session not found
            if status.code() == Code::NotFound && status.message().contains("Session not found:") {
                continue;
            }
            // backoff retry
            if status.code() == Code::Aborted {
                tokio::time::sleep(backoff.duration()).await;
                continue;
            }
            return Err(err);
        }
    }

    async fn get_session(&self) -> Result<ManagedSession, SessionError> {
        return self.sessions.get().await;
    }

    fn split_read_write_transaction_option(
        options: Option<ReadWriteTransactionOption>,
    ) -> (CallSettings, CallOptions, CommitOptions) {
        match options {
            Some(s) => (
                s.transaction_retry_setting,
                s.begin_options,
                s.commit_options,
            ),
            None => {
                let s = ReadWriteTransactionOption::default();
                (
                    s.transaction_retry_setting,
                    s.begin_options,
                    s.commit_options,
                )
            }
        }
    }
}
