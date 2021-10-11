use crate::apiv1::spanner_client::Client;
use crate::client::ReadWriteTransactionOption;
use crate::session_pool::{ManagedSession, SessionHandle, SessionManager};
use crate::statement::Statement;
use crate::transaction::{CallOptions, QueryOptions, Transaction};
use async_trait::async_trait;
use chrono::NaiveDateTime;
use gax::call_option::CallSettings;
use gax::invoke::AsTonicStatus;
use internal::spanner::v1::spanner_client::SpannerClient;
use internal::spanner::v1::transaction_options::Mode::ReadWrite;
use internal::spanner::v1::{
    commit_request, execute_batch_dml_request, execute_sql_request::QueryMode, request_options,
    result_set_stats, transaction_options, transaction_selector, BeginTransactionRequest,
    CommitRequest, CommitResponse, ExecuteBatchDmlRequest, ExecuteSqlRequest, Mutation,
    RequestOptions, ResultSet, ResultSetStats, RollbackRequest, Session, TransactionOptions,
    TransactionSelector,
};
use prost_types::Struct;
use std::future::Future;
use std::net::Shutdown::Read;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct CommitOptions {
    pub return_commit_stats: bool,
    pub call_options: CallOptions,
}

impl Default for CommitOptions {
    fn default() -> Self {
        CommitOptions {
            return_commit_stats: false,
            call_options: CallOptions::default(),
        }
    }
}

pub struct ReadWriteTransaction {
    base_tx: Transaction,
    tx_id: Vec<u8>,
    pub wb: Vec<Mutation>,
}

impl Deref for ReadWriteTransaction {
    type Target = Transaction;

    fn deref(&self) -> &Self::Target {
        return &self.base_tx;
    }
}

impl DerefMut for ReadWriteTransaction {
    fn deref_mut(&mut self) -> &mut Transaction {
        return &mut self.base_tx;
    }
}

impl ReadWriteTransaction {
    pub async fn begin(
        mut session: ManagedSession,
        options: CallOptions,
    ) -> Result<ReadWriteTransaction, tonic::Status> {
        return ReadWriteTransaction::begin_internal(
            session,
            transaction_options::Mode::ReadWrite(transaction_options::ReadWrite {}),
            options,
        )
        .await;
    }

    pub async fn begin_partitioned_dml(
        mut session: ManagedSession,
        options: CallOptions,
    ) -> Result<ReadWriteTransaction, tonic::Status> {
        return ReadWriteTransaction::begin_internal(
            session,
            transaction_options::Mode::PartitionedDml(transaction_options::PartitionedDml {}),
            options,
        )
        .await;
    }

    async fn begin_internal(
        mut session: ManagedSession,
        mode: transaction_options::Mode,
        options: CallOptions,
    ) -> Result<ReadWriteTransaction, tonic::Status> {
        let request = BeginTransactionRequest {
            session: session.session.name.to_string(),
            options: Some(TransactionOptions { mode: Some(mode) }),
            request_options: Transaction::create_request_options(options.priority),
        };
        let result = session
            .spanner_client
            .begin_transaction(request, options.call_setting)
            .await;
        match session.invalidate_if_needed(result).await {
            Ok(response) => {
                let tx = response.into_inner();
                Ok(ReadWriteTransaction {
                    base_tx: Transaction {
                        session,
                        sequence_number: AtomicI64::new(0),
                        transaction_selector: TransactionSelector {
                            selector: Some(transaction_selector::Selector::Id(tx.id.clone())),
                        },
                    },
                    tx_id: tx.id,
                    wb: vec![],
                })
            }
            Err(e) => Err(e),
        }
    }

    pub fn buffer_write(&mut self, ms: Vec<Mutation>) {
        self.wb.extend_from_slice(&ms)
    }

    pub async fn update(
        &mut self,
        stmt: Statement,
        options: Option<QueryOptions>,
    ) -> Result<i64, tonic::Status> {
        let opt = match options {
            Some(o) => o,
            None => QueryOptions::default(),
        };

        let request = ExecuteSqlRequest {
            session: self.base_tx.session.session.name.to_string(),
            transaction: Some(self.transaction_selector.clone()),
            sql: stmt.sql.to_string(),
            params: Some(prost_types::Struct {
                fields: stmt.params,
            }),
            param_types: stmt.param_types,
            resume_token: vec![],
            query_mode: opt.mode.into(),
            partition_token: vec![],
            seqno: self.sequence_number.fetch_add(1, Ordering::Relaxed),
            query_options: opt.optimizer_options,
            request_options: Transaction::create_request_options(opt.call_options.priority),
        };

        let result = self
            .base_tx
            .session
            .spanner_client
            .execute_sql(request, opt.call_options.call_setting)
            .await;
        let response = self.session.invalidate_if_needed(result).await;
        match response {
            Ok(r) => Ok(extract_row_count(r.into_inner().stats)),
            Err(s) => Err(s),
        }
    }

    pub async fn batch_update(
        &mut self,
        stmt: Vec<Statement>,
        options: Option<QueryOptions>,
    ) -> Result<Vec<i64>, tonic::Status> {
        let opt = match options {
            Some(o) => o,
            None => QueryOptions::default(),
        };

        let request = ExecuteBatchDmlRequest {
            session: self.base_tx.session.session.name.to_string(),
            transaction: Some(self.transaction_selector.clone()),
            seqno: self.sequence_number.fetch_add(1, Ordering::Relaxed),
            request_options: Transaction::create_request_options(opt.call_options.priority),
            statements: stmt
                .into_iter()
                .map(|x| execute_batch_dml_request::Statement {
                    sql: x.sql,
                    params: Some(Struct { fields: x.params }),
                    param_types: x.param_types,
                })
                .collect(),
        };

        let result = self
            .base_tx
            .session
            .spanner_client
            .execute_batch_dml(request, opt.call_options.call_setting)
            .await;
        let response = self.session.invalidate_if_needed(result).await;
        match response {
            Ok(r) => Ok(r
                .into_inner()
                .result_sets
                .into_iter()
                .map(|x| extract_row_count(x.stats))
                .collect()),
            Err(s) => Err(s),
        }
    }

    pub async fn finish<T, E>(
        &mut self,
        result: Result<T, E>,
        options: Option<CommitOptions>,
    ) -> Result<(Option<prost_types::Timestamp>, T), E>
    where
        E: AsTonicStatus + From<tonic::Status>,
    {
        let opt = match options {
            Some(o) => o,
            None => CommitOptions::default(),
        };

        return match result {
            Ok(s) => match self.commit(opt).await {
                Ok(c) => Ok((c.commit_timestamp, s)),
                Err(e) => Err(E::from(e)),
            },
            Err(err) => {
                let status = match err.as_tonic_status() {
                    Some(status) => status,
                    None => {
                        self.rollback(opt.call_options.call_setting).await;
                        return Err(err);
                    }
                };
                match status.code() {
                    tonic::Code::Aborted => Err(err),
                    tonic::Code::NotFound => Err(err),
                    _ => {
                        self.rollback(opt.call_options.call_setting).await;
                        return Err(err);
                    }
                }
            }
        };
    }

    pub async fn commit(
        &mut self,
        options: CommitOptions,
    ) -> Result<CommitResponse, tonic::Status> {
        let session = &mut self.base_tx.session;
        return commit(
            session,
            self.wb.to_vec(),
            commit_request::Transaction::TransactionId(self.tx_id.clone()),
            options,
        )
        .await;
    }

    pub async fn rollback(&mut self, setting: Option<CallSettings>) -> Result<(), tonic::Status> {
        let request = RollbackRequest {
            session: self.base_tx.session.session.name.to_string(),
            transaction_id: self.tx_id.clone(),
        };
        let result = self
            .base_tx
            .session
            .spanner_client
            .rollback(request, setting)
            .await;
        let response = self.base_tx.session.invalidate_if_needed(result).await;
        match response {
            Ok(r) => Ok(r.into_inner()),
            Err(e) => Err(e),
        }
    }
}

pub async fn commit(
    session: &mut ManagedSession,
    ms: Vec<Mutation>,
    tx: commit_request::Transaction,
    commit_options: CommitOptions,
) -> Result<CommitResponse, tonic::Status> {
    let request = CommitRequest {
        session: session.session.name.to_string(),
        mutations: ms,
        transaction: Some(tx),
        request_options: Transaction::create_request_options(commit_options.call_options.priority),
        return_commit_stats: commit_options.return_commit_stats,
    };
    let result = session
        .spanner_client
        .commit(request, commit_options.call_options.call_setting)
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
