use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicI64;
use std::time::SystemTime;

use time::OffsetDateTime;

use crate::key::KeySet;
use crate::reader::{Reader, RowIterator, StatementReader, TableReader};
use crate::session::ManagedSession;
use crate::statement::Statement;
use crate::transaction::{CallOptions, QueryOptions, ReadOptions, Transaction};
use crate::value::TimestampBound;
use google_cloud_gax::grpc::Status;
use google_cloud_googleapis::spanner::v1::transaction_options::IsolationLevel;
use google_cloud_googleapis::spanner::v1::{
    transaction_options, transaction_selector, BeginTransactionRequest, DirectedReadOptions, ExecuteSqlRequest,
    PartitionOptions, PartitionQueryRequest, PartitionReadRequest, ReadRequest, TransactionOptions,
    TransactionSelector,
};

/// ReadOnlyTransaction provides a snapshot transaction with guaranteed
/// consistency across reads, but does not allow writes.  Read-only transactions
/// can be configured to read at timestamps in the past.
///
/// Read-only transactions do not take locks. Instead, they work by choosing a
/// Cloud Spanner timestamp, then executing all reads at that timestamp. Since
/// they do not acquire locks, they do not block concurrent read-write
/// transactions.
///
/// Unlike locking read-write transactions, read-only transactions never abort.
/// They can fail if the chosen read timestamp is garbage collected; however, the
/// default garbage collection policy is generous enough that most applications
/// do not need to worry about this in practice. See the documentation of
/// TimestampBound for more details.
pub struct ReadOnlyTransaction {
    base_tx: Transaction,
    pub rts: Option<OffsetDateTime>,
}

impl Deref for ReadOnlyTransaction {
    type Target = Transaction;

    fn deref(&self) -> &Self::Target {
        &self.base_tx
    }
}

impl DerefMut for ReadOnlyTransaction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base_tx
    }
}

impl ReadOnlyTransaction {
    pub async fn single(session: ManagedSession, tb: TimestampBound) -> Result<ReadOnlyTransaction, Status> {
        Ok(ReadOnlyTransaction {
            base_tx: Transaction {
                session: Some(session),
                sequence_number: AtomicI64::new(0),
                transaction_selector: TransactionSelector {
                    selector: Some(transaction_selector::Selector::SingleUse(TransactionOptions {
                        exclude_txn_from_change_streams: false,
                        mode: Some(transaction_options::Mode::ReadOnly(tb.into())),
                        isolation_level: IsolationLevel::Unspecified as i32,
                    })),
                },
            },
            rts: None,
        })
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
                exclude_txn_from_change_streams: false,
                mode: Some(transaction_options::Mode::ReadOnly(tb.into())),
                isolation_level: IsolationLevel::Unspecified as i32,
            }),
            request_options: Transaction::create_request_options(options.priority),
            mutation_key: None,
        };

        let result = session.spanner_client.begin_transaction(request, options.retry).await;
        match session.invalidate_if_needed(result).await {
            Ok(response) => {
                let tx = response.into_inner();
                let rts = tx.read_timestamp.unwrap();
                let st: SystemTime = rts.try_into().unwrap();
                Ok(ReadOnlyTransaction {
                    base_tx: Transaction {
                        session: Some(session),
                        sequence_number: AtomicI64::new(0),
                        transaction_selector: TransactionSelector {
                            selector: Some(transaction_selector::Selector::Id(tx.id)),
                        },
                    },
                    rts: Some(OffsetDateTime::from(st)),
                })
            }
            Err(e) => Err(e),
        }
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
        &self.base_tx
    }
}

impl DerefMut for BatchReadOnlyTransaction {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base_tx
    }
}

impl BatchReadOnlyTransaction {
    pub async fn begin(
        session: ManagedSession,
        tb: TimestampBound,
        options: CallOptions,
    ) -> Result<BatchReadOnlyTransaction, Status> {
        let tx = ReadOnlyTransaction::begin(session, tb, options).await?;
        Ok(BatchReadOnlyTransaction { base_tx: tx })
    }

    /// partition_read returns a list of Partitions that can be used to read rows from
    /// the database. These partitions can be executed across multiple processes,
    /// even across different machines. The partition size and count hints can be
    /// configured using PartitionOptions.
    pub async fn partition_read(
        &mut self,
        table: &str,
        columns: &[&str],
        keys: impl Into<KeySet> + Clone,
    ) -> Result<Vec<Partition<TableReader>>, Status> {
        self.partition_read_with_option(table, columns, keys, None, ReadOptions::default(), false, None)
            .await
    }

    /// partition_read returns a list of Partitions that can be used to read rows from
    /// the database. These partitions can be executed across multiple processes,
    /// even across different machines. The partition size and count hints can be
    /// configured using PartitionOptions.
    #[allow(clippy::too_many_arguments)]
    pub async fn partition_read_with_option(
        &mut self,
        table: &str,
        columns: &[&str],
        keys: impl Into<KeySet> + Clone,
        po: Option<PartitionOptions>,
        ro: ReadOptions,
        data_boost_enabled: bool,
        directed_read_options: Option<DirectedReadOptions>,
    ) -> Result<Vec<Partition<TableReader>>, Status> {
        let columns: Vec<String> = columns.iter().map(|x| x.to_string()).collect();
        let inner_keyset = keys.into().inner;
        let request = PartitionReadRequest {
            session: self.get_session_name(),
            transaction: Some(self.transaction_selector.clone()),
            table: table.to_string(),
            index: ro.index.clone(),
            columns: columns.clone(),
            key_set: Some(inner_keyset.clone()),
            partition_options: po,
        };
        let result = match self
            .as_mut_session()
            .spanner_client
            .partition_read(request, ro.call_options.retry)
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
                            table: table.to_string(),
                            index: ro.index.clone(),
                            columns: columns.clone(),
                            key_set: Some(inner_keyset.clone()),
                            limit: ro.limit,
                            resume_token: vec![],
                            partition_token: x.partition_token,
                            request_options: Transaction::create_request_options(ro.call_options.priority),
                            directed_read_options: directed_read_options.clone(),
                            data_boost_enabled,
                            order_by: 0,
                            lock_hint: 0,
                        },
                    },
                })
                .collect()),
            Err(e) => Err(e),
        };
        self.as_mut_session().invalidate_if_needed(result).await
    }

    /// partition_query returns a list of Partitions that can be used to execute a query against the database.
    pub async fn partition_query(&mut self, stmt: Statement) -> Result<Vec<Partition<StatementReader>>, Status> {
        self.partition_query_with_option(stmt, None, QueryOptions::default(), false, None)
            .await
    }

    /// partition_query returns a list of Partitions that can be used to execute a query against the database.
    pub async fn partition_query_with_option(
        &mut self,
        stmt: Statement,
        po: Option<PartitionOptions>,
        qo: QueryOptions,
        data_boost_enabled: bool,
        directed_read_options: Option<DirectedReadOptions>,
    ) -> Result<Vec<Partition<StatementReader>>, Status> {
        let request = PartitionQueryRequest {
            session: self.get_session_name(),
            transaction: Some(self.transaction_selector.clone()),
            sql: stmt.sql.clone(),
            params: Some(prost_types::Struct {
                fields: stmt.params.clone(),
            }),
            param_types: stmt.param_types.clone(),
            partition_options: po,
        };
        let result = match self
            .as_mut_session()
            .spanner_client
            .partition_query(request.clone(), qo.call_options.retry.clone())
            .await
        {
            Ok(r) => Ok(r
                .into_inner()
                .partitions
                .into_iter()
                .map(|x| Partition {
                    reader: StatementReader {
                        enable_resume: qo.enable_resume,
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
                            query_options: qo.optimizer_options.clone(),
                            request_options: Transaction::create_request_options(qo.call_options.priority),
                            data_boost_enabled,
                            directed_read_options: directed_read_options.clone(),
                            last_statement: false,
                        },
                    },
                })
                .collect()),
            Err(e) => Err(e),
        };
        self.as_mut_session().invalidate_if_needed(result).await
    }

    /// execute runs a single Partition obtained from partition_read or partition_query.
    pub async fn execute<T: Reader + Sync + Send + 'static>(
        &mut self,
        partition: Partition<T>,
        option: Option<CallOptions>,
    ) -> Result<RowIterator<'_, T>, Status> {
        let session = self.as_mut_session();
        RowIterator::new(session, partition.reader, option).await
    }
}
