use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use google_cloud_googleapis::spanner::v1::MultiplexedSessionPrecommitToken;
use parking_lot::Mutex;
use std::time::Duration;

use prost_types::Struct;

use crate::session::ManagedSession;
use crate::statement::Statement;
use crate::transaction::{CallOptions, QueryOptions, Transaction};
use crate::value::Timestamp;
use google_cloud_gax::grpc::{Code, Status};
use google_cloud_gax::retry::{RetrySetting, TryAs};
use google_cloud_googleapis::spanner::v1::commit_request::Transaction::TransactionId;
use google_cloud_googleapis::spanner::v1::transaction_options::IsolationLevel;
use google_cloud_googleapis::spanner::v1::{
    commit_request, execute_batch_dml_request, result_set_stats, transaction_options, transaction_selector,
    BeginTransactionRequest, CommitRequest, CommitResponse, ExecuteBatchDmlRequest, ExecuteSqlRequest, Mutation,
    ResultSetStats, RollbackRequest, TransactionOptions, TransactionSelector,
};

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

/// Update the precommit-token slot with `new_token` only if its `seq_num` is
/// strictly greater than the currently-stored token's `seq_num`. The proto
/// requires the client to send the precommit token with the highest `seq_num`
/// of the transaction attempt.
pub(crate) fn update_precommit_token(
    slot: &Mutex<Option<MultiplexedSessionPrecommitToken>>,
    new_token: &MultiplexedSessionPrecommitToken,
) {
    let mut guard = slot.lock();
    let should_replace = match guard.as_ref() {
        None => true,
        Some(cur) => new_token.seq_num > cur.seq_num,
    };
    if should_replace {
        *guard = Some(new_token.clone());
    }
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
        let is_read_write = matches!(mode, transaction_options::Mode::ReadWrite(_));
        let tx_options = TransactionOptions {
            exclude_txn_from_change_streams: false,
            mode: Some(mode),
            isolation_level: IsolationLevel::Unspecified as i32,
        };

        // Multiplexed sessions use inline begin for read-write transactions:
        // the transaction is started implicitly with the first RPC (query or
        // DML), not via a separate BeginTransaction call. This matches the
        // Java client's behaviour. (Read-only transactions on multiplexed
        // sessions still use explicit BeginTransaction in this crate; see
        // ReadOnlyTransaction::begin.) PartitionedDml is not supported by
        // inline begin and must always go through BeginTransaction.
        if session.session.multiplexed && is_read_write {
            let pending_tx_id = Arc::new(Mutex::new(None));
            let pending_token = Arc::new(Mutex::new(None));
            return Ok(ReadWriteTransaction {
                base_tx: Transaction {
                    session: Some(session),
                    sequence_number: AtomicI64::new(0),
                    transaction_selector: TransactionSelector {
                        selector: Some(transaction_selector::Selector::Begin(tx_options)),
                    },
                    transaction_tag,
                    disable_route_to_leader,
                    pending_inline_tx_id: Some(pending_tx_id),
                    pending_precommit_token: Some(pending_token),
                },
                tx_id: vec![],
                wb: vec![],
            });
        }

        let request = BeginTransactionRequest {
            session: session.session.name.to_string(),
            options: Some(tx_options),
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
        Ok(ReadWriteTransaction {
            base_tx: Transaction {
                session: Some(session),
                sequence_number: AtomicI64::new(0),
                transaction_selector: TransactionSelector {
                    selector: Some(transaction_selector::Selector::Id(tx.id.clone())),
                },
                transaction_tag,
                disable_route_to_leader,
                pending_inline_tx_id: None,
                pending_precommit_token: None,
            },
            tx_id: tx.id,
            wb: vec![],
        })
    }

    /// If inline begin captured a transaction ID, upgrade the selector to
    /// `Id(tx_id)` and mirror the id into `self.tx_id` (used by commit and
    /// rollback). Idempotent.
    fn resolve_inline_begin(&mut self) {
        self.base_tx.resolve_inline_begin_selector();
        if !self.tx_id.is_empty() {
            return;
        }
        if let Some(ref slot) = self.base_tx.pending_inline_tx_id {
            if let Some(tx_id) = slot.lock().clone() {
                self.tx_id = tx_id;
            }
        }
    }

    pub fn buffer_write(&mut self, ms: Vec<Mutation>) {
        self.wb.extend_from_slice(&ms)
    }

    pub async fn update(&mut self, stmt: Statement) -> Result<i64, Status> {
        self.update_with_option(stmt, QueryOptions::default()).await
    }

    pub async fn update_with_option(&mut self, stmt: Statement, options: QueryOptions) -> Result<i64, Status> {
        self.resolve_inline_begin();
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
        let session = self.as_mut_session();
        let result = session
            .spanner_client
            .execute_sql(request, disable_route_to_leader, options.call_options.retry)
            .await;
        let response = session.invalidate_if_needed(result).await?;
        let result_set = response.into_inner();
        // When this is the first operation in an inline-begin transaction,
        // the server returns the new transaction ID in the response metadata.
        // Capture it so commit() can use Id(tx_id) instead of Begin.
        if let Some(ref slot) = self.base_tx.pending_inline_tx_id {
            if let Some(ref meta) = result_set.metadata {
                if let Some(ref txn) = meta.transaction {
                    if !txn.id.is_empty() {
                        let mut guard = slot.lock();
                        if guard.is_none() {
                            *guard = Some(txn.id.clone());
                        }
                    }
                }
            }
        }
        // Capture the precommit token from DML responses (multiplexed sessions).
        if let Some(ref slot) = self.base_tx.pending_precommit_token {
            if let Some(token) = result_set.precommit_token.as_ref() {
                update_precommit_token(slot, token);
            }
        }
        Ok(extract_row_count(result_set.stats))
    }

    pub async fn batch_update(&mut self, stmt: Vec<Statement>) -> Result<Vec<i64>, Status> {
        self.batch_update_with_option(stmt, QueryOptions::default()).await
    }

    pub async fn batch_update_with_option(
        &mut self,
        stmt: Vec<Statement>,
        options: QueryOptions,
    ) -> Result<Vec<i64>, Status> {
        self.resolve_inline_begin();
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
        let session = self.as_mut_session();
        let result = session
            .spanner_client
            .execute_batch_dml(request, disable_route_to_leader, options.call_options.retry)
            .await;
        let response = session.invalidate_if_needed(result).await?.into_inner();
        // When this is the first operation in an inline-begin transaction,
        // the server returns the new transaction ID in the metadata of the
        // first ResultSet (only result_sets[0] carries valid metadata).
        if let Some(ref slot) = self.base_tx.pending_inline_tx_id {
            if let Some(first) = response.result_sets.first() {
                if let Some(ref meta) = first.metadata {
                    if let Some(ref txn) = meta.transaction {
                        if !txn.id.is_empty() {
                            let mut guard = slot.lock();
                            if guard.is_none() {
                                *guard = Some(txn.id.clone());
                            }
                        }
                    }
                }
            }
        }
        // Capture the response-level precommit token (multiplexed sessions).
        if let Some(ref slot) = self.base_tx.pending_precommit_token {
            if let Some(token) = response.precommit_token.as_ref() {
                update_precommit_token(slot, token);
            }
        }
        Ok(response.result_sets.into_iter().map(|x| extract_row_count(x.stats)).collect())
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
        self.resolve_inline_begin();
        // Mutations-only commit on a multiplexed session: no read/DML fired,
        // so there is no server-side transaction yet. The Spanner protocol
        // requires an explicit BeginTransaction with `mutation_key` set to one
        // of the buffered mutations in this case.
        if self.tx_id.is_empty() && self.base_tx.pending_inline_tx_id.is_some() && !self.wb.is_empty() {
            self.begin_for_mutations_only(&options).await?;
        }
        // No server-side transaction was ever started (no read/DML and no
        // mutations to commit). There is nothing to commit; return a synthetic
        // empty response rather than shipping `TransactionId(empty)`, which
        // the server would reject.
        if self.tx_id.is_empty() {
            return Ok(CommitResponse::default());
        }
        let tx_id = self.tx_id.clone();
        let mutations = self.wb.to_vec();
        let disable_route_to_leader = self.disable_route_to_leader;
        // Collect the precommit token required by Spanner Omni for inline-begin
        // transactions on multiplexed sessions.
        let precommit_token = self
            .base_tx
            .pending_precommit_token
            .as_ref()
            .and_then(|slot| slot.lock().clone());
        let session = self.as_mut_session();
        commit(session, mutations, TransactionId(tx_id), options, disable_route_to_leader, precommit_token).await
    }

    /// Issue an explicit BeginTransaction with `mutation_key` for the
    /// multiplexed-session mutations-only case. The returned tx_id is mirrored
    /// into the inline-begin slot, the selector, and `self.tx_id`.
    async fn begin_for_mutations_only(&mut self, options: &CommitOptions) -> Result<(), Status> {
        let tx_options = match self.base_tx.transaction_selector.selector {
            Some(transaction_selector::Selector::Begin(ref opts)) => opts.clone(),
            _ => TransactionOptions {
                exclude_txn_from_change_streams: false,
                mode: Some(transaction_options::Mode::ReadWrite(transaction_options::ReadWrite::default())),
                isolation_level: IsolationLevel::Unspecified as i32,
            },
        };
        // Spanner v1 proto for BeginTransactionRequest.mutation_key advises:
        // "Clients should randomly select one of the mutations from the
        // mutation set and send it as a part of this request." A stable
        // first-element pick would defeat the server's partition lookup
        // load-spreading. Use clock nanos for a cheap pseudo-random index
        // (no new crate dependency; spec only requires load distribution,
        // not cryptographic randomness).
        let mutation_key = if self.wb.is_empty() {
            None
        } else {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.subsec_nanos() as usize)
                .unwrap_or(0);
            self.wb.get(nanos % self.wb.len()).cloned()
        };
        let request = BeginTransactionRequest {
            session: self.get_session_name(),
            options: Some(tx_options),
            request_options: Transaction::create_request_options(
                options.call_options.priority,
                self.base_tx.transaction_tag.clone(),
            ),
            mutation_key,
        };
        let disable_route_to_leader = self.disable_route_to_leader;
        let retry = options.call_options.retry.clone();
        let session = self.as_mut_session();
        let result = session.spanner_client.begin_transaction(request, disable_route_to_leader, retry).await;
        let response = session.invalidate_if_needed(result).await?;
        let tx = response.into_inner();
        if tx.id.is_empty() {
            return Err(Status::new(
                Code::Internal,
                "BeginTransaction returned empty transaction id for mutations-only commit",
            ));
        }
        // The server returns a precommit token on the BeginTransaction response
        // when `mutation_key` is set on a multiplexed session; it must be sent
        // back on the Commit request or the server rejects the commit.
        if let Some(ref slot) = self.base_tx.pending_precommit_token {
            if let Some(token) = tx.precommit_token.as_ref() {
                update_precommit_token(slot, token);
            }
        }
        if let Some(ref slot) = self.base_tx.pending_inline_tx_id {
            *slot.lock() = Some(tx.id.clone());
        }
        self.base_tx.transaction_selector = TransactionSelector {
            selector: Some(transaction_selector::Selector::Id(tx.id.clone())),
        };
        self.tx_id = tx.id;
        Ok(())
    }

    pub(crate) async fn rollback(&mut self, retry: Option<RetrySetting>) -> Result<(), Status> {
        // Mirror any inline-begin tx_id captured by a prior read/DML so that
        // rollback targets the right server-side transaction.
        self.resolve_inline_begin();
        // If no server-side transaction was ever started (no read/DML and no
        // mutations-only begin), there is nothing to roll back.
        if self.tx_id.is_empty() {
            return Ok(());
        }
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
    let request = CommitRequest {
        session: session.session.name.to_string(),
        mutations: ms,
        transaction: Some(tx),
        request_options: Transaction::create_request_options(
            commit_options.call_options.priority,
            commit_options.transaction_tag.clone(),
        ),
        return_commit_stats: commit_options.return_commit_stats,
        max_commit_delay: commit_options.max_commit_delay.map(|d| d.try_into().unwrap()),
        precommit_token,
    };
    let result = session
        .spanner_client
        .commit(request, disable_route_to_leader, commit_options.call_options.retry)
        .await;
    let response = session.invalidate_if_needed(result).await;
    match response {
        Ok(r) => Ok(r.into_inner()),
        Err(s) => Err(s),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn token(bytes: &[u8], seq_num: i32) -> MultiplexedSessionPrecommitToken {
        MultiplexedSessionPrecommitToken {
            precommit_token: bytes.to_vec(),
            seq_num,
        }
    }

    #[test]
    fn update_precommit_token_stores_into_empty_slot() {
        let slot = Mutex::new(None);
        update_precommit_token(&slot, &token(b"a", 5));
        assert_eq!(slot.lock().as_ref().unwrap().seq_num, 5);
        assert_eq!(slot.lock().as_ref().unwrap().precommit_token, b"a");
    }

    #[test]
    fn update_precommit_token_replaces_when_seq_num_strictly_greater() {
        let slot = Mutex::new(Some(token(b"a", 3)));
        update_precommit_token(&slot, &token(b"b", 5));
        assert_eq!(slot.lock().as_ref().unwrap().seq_num, 5);
        assert_eq!(slot.lock().as_ref().unwrap().precommit_token, b"b");
    }

    #[test]
    fn update_precommit_token_keeps_existing_when_seq_num_lower() {
        let slot = Mutex::new(Some(token(b"a", 5)));
        update_precommit_token(&slot, &token(b"b", 3));
        assert_eq!(slot.lock().as_ref().unwrap().seq_num, 5);
        assert_eq!(slot.lock().as_ref().unwrap().precommit_token, b"a");
    }

    #[test]
    fn update_precommit_token_keeps_existing_when_seq_num_equal() {
        let slot = Mutex::new(Some(token(b"a", 5)));
        update_precommit_token(&slot, &token(b"b", 5));
        assert_eq!(slot.lock().as_ref().unwrap().precommit_token, b"a");
    }
}
