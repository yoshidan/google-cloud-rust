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
use google_cloud_gax::call_option::{Backoff, BackoffRetryer, CallSettings};
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
use tokio::time::{Instant, Interval};
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
            check_session_not_found: true,
        },
    }
}

pub struct Client {
    sessions: SessionManager,
}

pub struct ChannelConfig {
    pub num_channels: usize,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        ChannelConfig { num_channels: 4 }
    }
}

pub struct ClientConfig {
    pub session_config: SessionConfig,
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

    pub async fn close(&mut self) {
        self.sessions.close().await;
    }

    pub async fn single(&self) -> Result<ReadOnlyTransaction, TxError> {
        let session = self.get_session().await?;
        let result = ReadOnlyTransaction::single(session, TimestampBound::strong_read()).await?;
        Ok(result)
    }

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

    pub async fn apply(
        &self,
        ms: Vec<Mutation>,
        options: Option<ReadWriteTransactionOption>,
    ) -> Result<Option<Timestamp>, TxError> {
        let result: Result<(Option<Timestamp>, ()), TxError> = self
            .run_in_read_write_transaction(
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

    pub async fn read_write_transaction(
        &self,
        options: Option<CallOptions>,
    ) -> Result<ReadWriteTransaction, TxError> {
        let opt = match options {
            Some(o) => o,
            None => CallOptions::default(),
        };
        let session = self.get_session().await?;
        let result = ReadWriteTransaction::begin(session, opt).await?;
        Ok(result)
    }

    pub async fn run_with_retry<T, E, Fut>(
        mut f: impl FnMut() -> Fut,
        retry_setting: Option<CallSettings>,
    ) -> Result<T, E>
    where
        E: AsTonicStatus,
        Fut: Future<Output = Result<T, E>>,
    {
        let mut o = match retry_setting {
            Some(c) => c,
            None => default_transaction_retry_setting(),
        };
        return retryer::invoke(f, &mut o).await;
    }

    pub async fn run_in_read_write_transaction<T, E, F>(
        &self,
        mut f: impl Fn(Arc<Mutex<ReadWriteTransaction>>) -> F,
        options: Option<ReadWriteTransactionOption>,
    ) -> Result<(Option<prost_types::Timestamp>, T), E>
    where
        E: AsTonicStatus + From<TxError> + From<Status>,
        F: Future<Output = Result<T, E>>,
    {
        let (mut ro, bo, co) = Client::split_read_write_transaction_option(options);

        return retryer::invoke(
            || async {
                let mut tx = self.get_rw_transaction(bo.clone()).await?;

                let arc = Arc::new(Mutex::new(tx));
                let result = f(arc.clone()).await;

                let mut tx = arc.lock().await;
                return tx.finish(result, Some(co.clone())).await;
            },
            &mut ro,
        )
        .await;
    }

    async fn get_session(&self) -> Result<ManagedSession, SessionError> {
        return self.sessions.get().await;
    }

    async fn get_rw_transaction(
        &self,
        options: CallOptions,
    ) -> Result<ReadWriteTransaction, TxError> {
        let session = self.get_session().await?;
        let result = ReadWriteTransaction::begin(session, options).await?;
        Ok(result)
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
