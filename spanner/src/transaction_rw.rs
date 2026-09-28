use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration;

use prost_types::Struct;

use crate::session::ManagedSession;
use crate::statement::Statement;
use crate::transaction::{observe_precommit_token, CallOptions, PrecommitTokenSink, QueryOptions, Transaction};
use crate::value::Timestamp;
use google_cloud_gax::grpc::{Code, Status};
use google_cloud_gax::retry::{RetrySetting, TryAs};
use google_cloud_googleapis::spanner::v1::commit_request::Transaction::TransactionId;
use google_cloud_googleapis::spanner::v1::transaction_options::IsolationLevel;
use google_cloud_googleapis::spanner::v1::{
    commit_request, commit_response, execute_batch_dml_request, result_set_stats, transaction_options,
    transaction_selector, BeginTransactionRequest, CommitRequest, CommitResponse, ExecuteBatchDmlRequest,
    ExecuteSqlRequest, MultiplexedSessionPrecommitToken, Mutation, ResultSetStats, RollbackRequest, TransactionOptions,
    TransactionSelector,
};

/// A commit on a multiplexed session can succeed at the RPC level without having committed,
/// returning a fresh precommit token and asking to be retried with it. Bound how often that
/// exchange may repeat so a misbehaving backend cannot loop forever.
const MAX_PRECOMMIT_TOKEN_RETRIES: usize = 3;

#[derive(Clone, Default)]
pub struct CommitOptions {
    pub return_commit_stats: bool,
    pub call_options: CallOptions,
    pub max_commit_delay: Option<Duration>,
    /// The transaction tag to use for the CommitRequest.
    pub transaction_tag: Option<String>,
}

#[derive(Clone)]
pub struct CommitResult {
    pub timestamp: Option<Timestamp>,
    pub mutation_count: Option<u64>,
}

impl From<CommitResponse> for CommitResult {
    fn from(value: CommitResponse) -> Self {
        Self {
            timestamp: value.commit_timestamp.map(|v| v.into()),
            mutation_count: value.commit_stats.map(|s| s.mutation_count as u64),
        }
    }
}

/// ReadWriteTransaction provides a locking read-write transaction.
///
/// This type of transaction is the only way to write data into Cloud Spanner;
/// Client::apply, Client::apply_at_least_once, Client::partitioned_update use
/// transactions internally. These transactions rely on pessimistic locking and,
/// if necessary, two-phase commit. Locking read-write transactions may abort,
/// requiring the application to retry. However, the interface exposed by
/// Client:run_with_retry eliminates the need for applications to write
/// retry loops explicitly.
///
/// Locking transactions may be used to atomically read-modify-write data
/// anywhere in a database. This type of transaction is externally consistent.
///
/// Clients should attempt to minimize the amount of time a transaction is
/// active. Faster transactions commit with higher probability and cause less
/// contention. Cloud Spanner attempts to keep read locks active as long as the
/// transaction continues to do reads.  Long periods of inactivity at the client
/// may cause Cloud Spanner to release a transaction's locks and abort it.
///
/// Reads performed within a transaction acquire locks on the data being
/// read. Writes can only be done at commit time, after all reads have been
/// completed. Conceptually, a read-write transaction consists of zero or more
/// reads or SQL queries followed by a commit.
///
/// See Client::run_with_retry for an example.
///
/// Semantics
///
/// Cloud Spanner can commit the transaction if all read locks it acquired are
/// still valid at commit time, and it is able to acquire write locks for all
/// writes. Cloud Spanner can abort the transaction for any reason. If a commit
/// attempt returns ABORTED, Cloud Spanner guarantees that the transaction has
/// not modified any user data in Cloud Spanner.
///
/// Unless the transaction commits, Cloud Spanner makes no guarantees about how
/// long the transaction's locks were held for. It is an error to use Cloud
/// Spanner locks for any sort of mutual exclusion other than between Cloud
/// Spanner transactions themselves.
///
/// Aborted transactions
///
/// Application code does not need to retry explicitly; RunInTransaction will
/// automatically retry a transaction if an attempt results in an abort. The lock
/// priority of a transaction increases after each prior aborted transaction,
/// meaning that the next attempt has a slightly better chance of success than
/// before.
///
/// Under some circumstances (e.g., many transactions attempting to modify the
/// same row(s)), a transaction can abort many times in a short period before
/// successfully committing. Thus, it is not a good idea to cap the number of
/// retries a transaction can attempt; instead, it is better to limit the total
/// amount of wall time spent retrying.
pub struct ReadWriteTransaction {
    base_tx: Transaction,
    tx_id: Vec<u8>,
    wb: Vec<Mutation>,
}

impl Deref for ReadWriteTransaction {
    type Target = Transaction;

    fn deref(&self) -> &Self::Target {
        &self.base_tx
    }
}

impl DerefMut for ReadWriteTransaction {
    fn deref_mut(&mut self) -> &mut Transaction {
        &mut self.base_tx
    }
}

pub struct BeginError {
    pub status: Status,
    pub session: ManagedSession,
}

impl ReadWriteTransaction {
    pub async fn begin(
        session: ManagedSession,
        options: CallOptions,
        transaction_tag: Option<String>,
        disable_route_to_leader: bool,
    ) -> Result<ReadWriteTransaction, BeginError> {
        ReadWriteTransaction::begin_internal(
            session,
            transaction_options::Mode::ReadWrite(transaction_options::ReadWrite::default()),
            options,
            transaction_tag,
            disable_route_to_leader,
        )
        .await
    }

    pub async fn begin_partitioned_dml(
        session: ManagedSession,
        options: CallOptions,
        transaction_tag: Option<String>,
        disable_route_to_leader: bool,
    ) -> Result<ReadWriteTransaction, BeginError> {
        ReadWriteTransaction::begin_internal(
            session,
            transaction_options::Mode::PartitionedDml(transaction_options::PartitionedDml {}),
            options,
            transaction_tag,
            disable_route_to_leader,
        )
        .await
    }

    async fn begin_internal(
        mut session: ManagedSession,
        mode: transaction_options::Mode,
        options: CallOptions,
        transaction_tag: Option<String>,
        disable_route_to_leader: bool,
    ) -> Result<ReadWriteTransaction, BeginError> {
        let request = BeginTransactionRequest {
            session: session.session.name.to_string(),
            options: Some(TransactionOptions {
                exclude_txn_from_change_streams: false,
                mode: Some(mode),
                isolation_level: IsolationLevel::Unspecified as i32,
            }),
            request_options: Transaction::create_request_options(options.priority, transaction_tag.clone()),
            mutation_key: None,
        };
        let result = session
            .spanner_client
            .begin_transaction(request, disable_route_to_leader, options.retry)
            .await;
        let response = match session.invalidate_if_needed(result).await {
            Ok(response) => response,
            Err(err) => {
                return Err(BeginError { status: err, session });
            }
        };
        let tx = response.into_inner();
        let precommit_token = PrecommitTokenSink::default();
        // BeginTransaction only returns a token when a mutation_key was supplied, which it
        // never is here -- see begin_with_mutation_key. Observe it anyway so the source is
        // handled uniformly with the others.
        observe_precommit_token(&precommit_token, tx.precommit_token);
        Ok(ReadWriteTransaction {
            base_tx: Transaction {
                session: Some(session),
                sequence_number: AtomicI64::new(0),
                transaction_selector: TransactionSelector {
                    selector: Some(transaction_selector::Selector::Id(tx.id.clone())),
                },
                transaction_tag,
                disable_route_to_leader,
                precommit_token,
            },
            tx_id: tx.id,
            wb: vec![],
        })
    }

    /// Re-issues BeginTransaction with a `mutation_key` so that Spanner hands back a
    /// precommit token.
    ///
    /// A read-write transaction on a multiplexed session must commit with a precommit
    /// token. Tokens normally ride on statement responses, but a transaction that only
    /// buffers mutations never runs a statement, so its only possible source is
    /// BeginTransaction -- and BeginTransaction only returns one when a `mutation_key` was
    /// supplied. Mutations are buffered after the transaction begins, so the key is not
    /// known until commit time and the transaction has to be begun a second time.
    ///
    /// The transaction being replaced has run no statements, which is exactly what "no
    /// token yet" means here, so nothing is lost by starting a new one. Any buffered
    /// mutation serves as the key; Spanner only uses it to route the transaction.
    async fn begin_with_mutation_key(&mut self, options: &CommitOptions) -> Result<(), Status> {
        let Some(mutation_key) = self.wb.first().cloned() else {
            return Ok(());
        };
        let request = BeginTransactionRequest {
            session: self.get_session_name(),
            options: Some(TransactionOptions {
                exclude_txn_from_change_streams: false,
                mode: Some(transaction_options::Mode::ReadWrite(transaction_options::ReadWrite::default())),
                isolation_level: IsolationLevel::Unspecified as i32,
            }),
            request_options: Transaction::create_request_options(
                options.call_options.priority,
                self.base_tx.transaction_tag.clone(),
            ),
            mutation_key: Some(mutation_key),
        };
        let disable_route_to_leader = self.disable_route_to_leader;
        let precommit_token = self.base_tx.precommit_token.clone();
        let session = self.as_mut_session();
        let result = session
            .spanner_client
            .begin_transaction(request, disable_route_to_leader, options.call_options.retry.clone())
            .await;
        let tx = session.invalidate_if_needed(result).await?.into_inner();
        observe_precommit_token(&precommit_token, tx.precommit_token);
        self.tx_id = tx.id.clone();
        self.base_tx.transaction_selector = TransactionSelector {
            selector: Some(transaction_selector::Selector::Id(tx.id)),
        };
        Ok(())
    }

    pub fn buffer_write(&mut self, ms: Vec<Mutation>) {
        self.wb.extend_from_slice(&ms)
    }

    pub async fn update(&mut self, stmt: Statement) -> Result<i64, Status> {
        self.update_with_option(stmt, QueryOptions::default()).await
    }

    pub async fn update_with_option(&mut self, stmt: Statement, options: QueryOptions) -> Result<i64, Status> {
        let request = ExecuteSqlRequest {
            session: self.get_session_name(),
            transaction: Some(self.transaction_selector.clone()),
            sql: stmt.sql.to_string(),
            data_boost_enabled: false,
            params: Some(prost_types::Struct { fields: stmt.params }),
            param_types: stmt.param_types,
            resume_token: vec![],
            query_mode: options.mode.into(),
            partition_token: vec![],
            seqno: self.sequence_number.fetch_add(1, Ordering::Relaxed),
            query_options: options.optimizer_options,
            request_options: Transaction::create_request_options(
                options.call_options.priority,
                self.base_tx.transaction_tag.clone(),
            ),
            directed_read_options: None,
            last_statement: false,
        };
        let disable_route_to_leader = self.disable_route_to_leader;
        let precommit_token = self.base_tx.precommit_token.clone();
        let session = self.as_mut_session();
        let result = session
            .spanner_client
            .execute_sql(request, disable_route_to_leader, options.call_options.retry)
            .await;
        let response = session.invalidate_if_needed(result).await?.into_inner();
        observe_precommit_token(&precommit_token, response.precommit_token);
        Ok(extract_row_count(response.stats))
    }

    pub async fn batch_update(&mut self, stmt: Vec<Statement>) -> Result<Vec<i64>, Status> {
        self.batch_update_with_option(stmt, QueryOptions::default()).await
    }

    pub async fn batch_update_with_option(
        &mut self,
        stmt: Vec<Statement>,
        options: QueryOptions,
    ) -> Result<Vec<i64>, Status> {
        let request = ExecuteBatchDmlRequest {
            session: self.get_session_name(),
            transaction: Some(self.transaction_selector.clone()),
            seqno: self.sequence_number.fetch_add(1, Ordering::Relaxed),
            request_options: Transaction::create_request_options(
                options.call_options.priority,
                self.base_tx.transaction_tag.clone(),
            ),
            statements: stmt
                .into_iter()
                .map(|x| execute_batch_dml_request::Statement {
                    sql: x.sql,
                    params: Some(Struct { fields: x.params }),
                    param_types: x.param_types,
                })
                .collect(),
            last_statements: false,
        };

        let disable_route_to_leader = self.disable_route_to_leader;
        let precommit_token = self.base_tx.precommit_token.clone();
        let session = self.as_mut_session();
        let result = session
            .spanner_client
            .execute_batch_dml(request, disable_route_to_leader, options.call_options.retry)
            .await;
        let response = session.invalidate_if_needed(result).await?.into_inner();
        observe_precommit_token(&precommit_token, response.precommit_token);
        Ok(response
            .result_sets
            .into_iter()
            .map(|x| extract_row_count(x.stats))
            .collect())
    }

    pub async fn end<S, E>(
        &mut self,
        result: Result<S, E>,
        options: Option<CommitOptions>,
    ) -> Result<(CommitResult, S), E>
    where
        E: TryAs<Status> + From<Status>,
    {
        let opt = options.unwrap_or_default();
        match result {
            Ok(success) => {
                let cr = self.commit(opt).await?;
                Ok((cr.into(), success))
            }
            Err(err) => {
                if let Some(status) = err.try_as() {
                    // can't rollback. should retry
                    if status.code() == Code::Aborted {
                        return Err(err);
                    }
                }
                let _ = self.rollback(opt.call_options.retry).await;
                Err(err)
            }
        }
    }

    pub(crate) async fn finish<T, E>(
        &mut self,
        result: Result<T, E>,
        options: Option<CommitOptions>,
    ) -> Result<(CommitResult, T), (E, Option<ManagedSession>)>
    where
        E: TryAs<Status> + From<Status>,
    {
        let opt = options.unwrap_or_default();

        match result {
            Ok(s) => match self.commit(opt).await {
                Ok(c) => Ok((c.into(), s)),
                // Retry the transaction using the same session on ABORT error.
                // Cloud Spanner will create the new transaction with the previous
                // one's wound-wait priority.
                Err(e) => Err((E::from(e), self.take_session())),
            },

            // Rollback the transaction unless the error occurred during the
            // commit. Executing a rollback after a commit has failed will
            // otherwise cause an error. Note that transient errors, such as
            // UNAVAILABLE, are already handled in the gRPC layer and do not show
            // up here. Context errors (deadline exceeded / canceled) during
            // commits are also not rolled back.
            Err(err) => {
                let status = match err.try_as() {
                    Some(status) => status,
                    None => {
                        let _ = self.rollback(opt.call_options.retry).await;
                        return Err((err, self.take_session()));
                    }
                };
                match status.code() {
                    Code::Aborted => Err((err, self.take_session())),
                    _ => {
                        let _ = self.rollback(opt.call_options.retry).await;
                        Err((err, self.take_session()))
                    }
                }
            }
        }
    }

    pub(crate) async fn commit(&mut self, options: CommitOptions) -> Result<CommitResponse, Status> {
        // On a multiplexed session a mutation-only transaction has no statement response to
        // take a precommit token from, and Spanner will not commit without one.
        if self.uses_multiplexed_session() && !self.wb.is_empty() && self.base_tx.precommit_token.lock().is_none() {
            self.begin_with_mutation_key(&options).await?;
        }
        let tx_id = self.tx_id.clone();
        let mutations = self.wb.to_vec();
        let disable_route_to_leader = self.disable_route_to_leader;
        let precommit_token = self.base_tx.precommit_token.lock().clone();
        let session = self.as_mut_session();
        commit(
            session,
            mutations,
            TransactionId(tx_id),
            options,
            disable_route_to_leader,
            precommit_token,
        )
        .await
    }

    pub(crate) async fn rollback(&mut self, retry: Option<RetrySetting>) -> Result<(), Status> {
        let request = RollbackRequest {
            transaction_id: self.tx_id.clone(),
            session: self.get_session_name(),
        };
        let disable_route_to_leader = self.disable_route_to_leader;
        let session = self.as_mut_session();
        let result = session
            .spanner_client
            .rollback(request, disable_route_to_leader, retry)
            .await;
        session.invalidate_if_needed(result).await?.into_inner();
        Ok(())
    }
}

pub(crate) async fn commit(
    session: &mut ManagedSession,
    ms: Vec<Mutation>,
    tx: commit_request::Transaction,
    commit_options: CommitOptions,
    disable_route_to_leader: bool,
    precommit_token: Option<MultiplexedSessionPrecommitToken>,
) -> Result<CommitResponse, Status> {
    // Only a multiplexed session can answer a commit with "not committed, try again with
    // this token", so the regular path stays a single request and never has to keep a copy
    // of the mutations around.
    let attempts = if session.session.multiplexed {
        MAX_PRECOMMIT_TOKEN_RETRIES
    } else {
        1
    };
    let mut precommit_token = precommit_token;
    let mut mutations = ms;

    for attempt in 1..=attempts {
        let request = CommitRequest {
            session: session.session.name.to_string(),
            mutations: if attempt == attempts {
                std::mem::take(&mut mutations)
            } else {
                mutations.clone()
            },
            transaction: Some(tx.clone()),
            request_options: Transaction::create_request_options(
                commit_options.call_options.priority,
                commit_options.transaction_tag.clone(),
            ),
            return_commit_stats: commit_options.return_commit_stats,
            max_commit_delay: commit_options.max_commit_delay.map(|d| d.try_into().unwrap()),
            precommit_token: precommit_token.take(),
        };
        let result = session
            .spanner_client
            .commit(request, disable_route_to_leader, commit_options.call_options.retry.clone())
            .await;
        let response = session.invalidate_if_needed(result).await?.into_inner();
        match response.multiplexed_session_retry {
            Some(commit_response::MultiplexedSessionRetry::PrecommitToken(token)) => {
                tracing::debug!("commit was not applied, retrying with the returned precommit token");
                precommit_token = Some(token);
            }
            None => return Ok(response),
        }
    }

    Err(Status::new(
        Code::Aborted,
        "commit did not complete after retrying with the precommit tokens returned by Spanner",
    ))
}

fn extract_row_count(rs: Option<ResultSetStats>) -> i64 {
    match rs {
        Some(o) => match o.row_count {
            Some(o) => match o {
                result_set_stats::RowCount::RowCountExact(v) => v,
                result_set_stats::RowCount::RowCountLowerBound(v) => v,
            },
            None => 0,
        },
        None => 0,
    }
}
