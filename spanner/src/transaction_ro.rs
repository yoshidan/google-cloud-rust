use crate::reader::{AsyncIterator, Reader, RowIterator, StatementReader, TableReader};
use crate::session_pool::{ManagedSession, SessionHandle, SessionManager};
use crate::statement::Statement;
use crate::transaction::{CallOptions, QueryOptions, ReadOptions, Transaction};
use crate::value::TimestampBound;
use async_trait::async_trait;
use chrono::NaiveDateTime;
use google_cloud_googleapis::spanner::v1::{
    commit_request, execute_sql_request::QueryMode, request_options, result_set_stats,
    transaction_options, transaction_selector, BeginTransactionRequest, CommitRequest,
    CommitResponse, ExecuteSqlRequest, KeySet, Mutation, PartitionOptions, PartitionQueryRequest,
    PartitionReadRequest, PartitionResponse, ReadRequest, RequestOptions, RollbackRequest, Session,
    TransactionOptions, TransactionSelector,
};
use prost_types::Struct;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicI64, Ordering};
use tonic::{Response, Status};

/// ReadOnlyTransaction provides a snapshot transaction with guaranteed
/// consistency across reads, but does not allow writes.  Read-only transactions
/// can be configured to read at timestamps in the past.
//
/// Read-only transactions do not take locks. Instead, they work by choosing a
/// Cloud Spanner timestamp, then executing all reads at that timestamp. Since
/// they do not acquire locks, they do not block concurrent read-write
/// transactions.
//
/// Unlike locking read-write transactions, read-only transactions never abort.
/// They can fail if the chosen read timestamp is garbage collected; however, the
/// default garbage collection policy is generous enough that most applications
/// do not need to worry about this in practice. See the documentation of
/// TimestampBound for more details.
pub struct ReadOnlyTransaction {
    base_tx: Transaction,
    rts: Option<NaiveDateTime>,
}

impl Deref for ReadOnlyTransaction {
    type Target = Transaction;

    fn deref(&self) -> &Self::Target {
        return &self.base_tx;
    }
}

impl DerefMut for ReadOnlyTransaction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return &mut self.base_tx;
    }
}

impl ReadOnlyTransaction {
    pub async fn single(
        session: ManagedSession,
        tb: TimestampBound,
    ) -> Result<ReadOnlyTransaction, Status> {
        return Ok(ReadOnlyTransaction {
            base_tx: Transaction {
                session: Some(session),
                sequence_number: AtomicI64::new(0),
                transaction_selector: TransactionSelector {
                    selector: Some(transaction_selector::Selector::SingleUse(
                        TransactionOptions {
                            mode: Some(transaction_options::Mode::ReadOnly(tb.into())),
                        },
                    )),
                },
            },
            rts: None,
        });
    }

    /// begin starts a snapshot read-only Transaction on Cloud Spanner.
    pub async fn begin(
        mut session: ManagedSession,
        tb: TimestampBound,
        options: CallOptions,
    ) -> Result<ReadOnlyTransaction, Status> {
        let request = BeginTransactionRequest {
            session: session.session.name.to_string(),
            options: Some(TransactionOptions {
                mode: Some(transaction_options::Mode::ReadOnly(tb.into())),
            }),
            request_options: Transaction::create_request_options(options.priority),
        };

        let result = session
            .spanner_client
            .begin_transaction(request, options.call_setting)
            .await;
        return match session.invalidate_if_needed(result).await {
            Ok(response) => {
                let tx = response.into_inner();
                let rts = tx.read_timestamp.unwrap();
                Ok(ReadOnlyTransaction {
                    base_tx: Transaction {
                        session: Some(session),
                        sequence_number: AtomicI64::new(0),
                        transaction_selector: TransactionSelector {
                            selector: Some(transaction_selector::Selector::Id(tx.id.clone())),
                        },
                    },
                    rts: Some(NaiveDateTime::from_timestamp(rts.seconds, rts.nanos as u32)),
                })
            }
            Err(e) => Err(e),
        };
    }
}

pub struct Partition<T: Reader> {
    pub reader: T,
}

/// BatchReadOnlyTransaction is a ReadOnlyTransaction that allows for exporting
/// arbitrarily large amounts of data from Cloud Spanner databases.
/// BatchReadOnlyTransaction partitions a read/query request. Read/query request
/// can then be executed independently over each partition while observing the
/// same snapshot of the database.
pub struct BatchReadOnlyTransaction {
    base_tx: ReadOnlyTransaction,
}

impl Deref for BatchReadOnlyTransaction {
    type Target = ReadOnlyTransaction;

    fn deref(&self) -> &Self::Target {
        return &self.base_tx;
    }
}

impl DerefMut for BatchReadOnlyTransaction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return &mut self.base_tx;
    }
}

impl BatchReadOnlyTransaction {
    pub async fn begin(
        mut session: ManagedSession,
        tb: TimestampBound,
        options: CallOptions,
    ) -> Result<BatchReadOnlyTransaction, Status> {
        match ReadOnlyTransaction::begin(session, tb, options).await {
            Ok(tx) => Ok(BatchReadOnlyTransaction { base_tx: tx }),
            Err(e) => Err(e),
        }
    }

    /// partition_read returns a list of Partitions that can be used to read rows from
    /// the database. These partitions can be executed across multiple processes,
    /// even across different machines. The partition size and count hints can be
    /// configured using PartitionOptions.
    pub async fn partition_read<T, C, K>(
        &mut self,
        table: T,
        keys: K,
        columns: Vec<C>,
        po: PartitionOptions,
        ro: Option<ReadOptions>,
    ) -> Result<Vec<Partition<TableReader>>, Status>
    where
        T: Into<String> + Clone,
        C: Into<String>,
        K: Into<KeySet> + Clone,
    {
        let columns: Vec<String> = columns.into_iter().map(|x| x.into()).collect();

        let opt = match ro {
            Some(o) => o,
            None => ReadOptions::default(),
        };

        let request = PartitionReadRequest {
            session: self.get_session_name(),
            transaction: Some(self.transaction_selector.clone()),
            table: table.clone().into(),
            index: opt.index.clone(),
            columns: columns.clone(),
            key_set: Some(keys.clone().into()),
            partition_options: Some(po),
        };
        let result = match self
            .as_mut_session()
            .spanner_client
            .partition_read(request, None)
            .await
        {
            Ok(r) => Ok(r
                .into_inner()
                .partitions
                .into_iter()
                .map(|x| Partition {
                    reader: TableReader {
                        request: ReadRequest {
                            session: self.get_session_name(),
                            transaction: Some(self.transaction_selector.clone()),
                            table: table.clone().into(),
                            index: opt.index.clone().into(),
                            columns: columns.clone(),
                            key_set: Some(keys.clone().into()),
                            limit: opt.limit,
                            resume_token: vec![],
                            partition_token: x.partition_token,
                            request_options: Transaction::create_request_options(
                                opt.call_options.priority,
                            ),
                        },
                        call_setting: opt.call_options.call_setting.clone(),
                    },
                })
                .collect()),
            Err(e) => Err(e),
        };
        return self.as_mut_session().invalidate_if_needed(result).await;
    }

    /// partition_query returns a list of Partitions that can be used to execute a query against the database.
    pub async fn partition_query(
        &mut self,
        stmt: Statement,
        po: PartitionOptions,
        qo: Option<QueryOptions>,
    ) -> Result<Vec<Partition<StatementReader>>, Status> {
        let opt = match qo {
            Some(o) => o,
            None => QueryOptions::default(),
        };

        let request = PartitionQueryRequest {
            session: self.get_session_name(),
            transaction: Some(self.transaction_selector.clone()),
            sql: stmt.sql.clone(),
            params: Some(prost_types::Struct {
                fields: stmt.params.clone(),
            }),
            param_types: stmt.param_types.clone(),
            partition_options: Some(po),
        };
        let result = match self
            .as_mut_session()
            .spanner_client
            .partition_query(request.clone(), None)
            .await
        {
            Ok(r) => Ok(r
                .into_inner()
                .partitions
                .into_iter()
                .map(|x| Partition {
                    reader: StatementReader {
                        request: ExecuteSqlRequest {
                            session: self.get_session_name(),
                            transaction: Some(self.transaction_selector.clone()),
                            sql: stmt.sql.clone(),
                            params: Some(prost_types::Struct {
                                fields: stmt.params.clone(),
                            }),
                            param_types: stmt.param_types.clone(),
                            resume_token: vec![],
                            query_mode: 0,
                            partition_token: x.partition_token,
                            seqno: 0,
                            query_options: opt.optimizer_options.clone(),
                            request_options: Transaction::create_request_options(
                                opt.call_options.priority,
                            ),
                        },
                        call_setting: opt.call_options.call_setting.clone(),
                    },
                })
                .collect()),
            Err(e) => Err(e),
        };
        return self.as_mut_session().invalidate_if_needed(result).await;
    }

    // execute runs a single Partition obtained from partition_read or partition_query.
    pub async fn execute<T: Reader + Sync + Send + 'static>(
        &mut self,
        partition: Partition<T>,
    ) -> Result<RowIterator<'_>, Status> {
        let session = self.as_mut_session();
        return RowIterator::new(session, Box::new(partition.reader)).await;
    }
}
