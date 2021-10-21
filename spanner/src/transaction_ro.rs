use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicI64;

use chrono::NaiveDateTime;
use tonic::Status;

use google_cloud_googleapis::spanner::v1::{
    transaction_options, transaction_selector, BeginTransactionRequest, ExecuteSqlRequest, KeySet,
    PartitionOptions, PartitionQueryRequest, PartitionReadRequest, ReadRequest, TransactionOptions,
    TransactionSelector,
};

use crate::reader::{Reader, RowIterator, StatementReader, TableReader};
use crate::session_pool::ManagedSession;
use crate::statement::Statement;
use crate::transaction::{CallOptions, QueryOptions, ReadOptions, Transaction};
use crate::value::TimestampBound;

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
    pub rts: Option<NaiveDateTime>,
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
        session: ManagedSession,
        tb: TimestampBound,
        options: CallOptions,
    ) -> Result<BatchReadOnlyTransaction, Status> {
        let tx = ReadOnlyTransaction::begin(session, tb, options).await?;
        return Ok(BatchReadOnlyTransaction { base_tx: tx });
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

#[cfg(test)]
mod tests {
    use crate::apiv1::conn_pool::ConnectionManager;
    use crate::apiv1::spanner_client::Client;
    use crate::mutation::insert_or_update;
    use crate::reader::AsyncIterator;
    use crate::row::Row;
    use crate::session_pool::{
        ManagedSession, SessionConfig, SessionHandle, SessionManager, SessionPool,
    };
    use crate::statement::{Kinds, Statement, ToKind, ToStruct, Types};
    use crate::transaction::{CallOptions, QueryOptions};
    use crate::transaction_ro::{BatchReadOnlyTransaction, ReadOnlyTransaction};
    use crate::value::{CommitTimestamp, TimestampBound};
    use chrono::{NaiveDate, NaiveDateTime, Utc};
    use google_cloud_googleapis::spanner::v1::commit_request::Transaction::SingleUseTransaction;
    use google_cloud_googleapis::spanner::v1::transaction_options::{Mode, ReadWrite};
    use google_cloud_googleapis::spanner::v1::{
        CommitRequest, CommitResponse, CreateSessionRequest, TransactionOptions,
    };
    use std::collections::VecDeque;
    use std::ops::DerefMut;
    use std::str::FromStr;
    use std::sync::Arc;
    use std::time::Instant;

    const DATABASE: &str =
        "projects/local-project/instances/test-instance/databases/local-database";

    async fn create_session() -> ManagedSession {
        let cm = ConnectionManager::new(1, Some("localhost:9010".to_string()))
            .await
            .unwrap();
        let session_request = CreateSessionRequest {
            database: DATABASE.to_string(),
            session: None,
        };
        let mut client = cm.conn();
        let session_response = client.create_session(session_request, None).await.unwrap();
        let session = session_response.into_inner();
        let handle = SessionHandle::new(session, client, Instant::now());
        let mut config = SessionConfig::default();
        config.min_opened = 1;
        config.max_opened = 1;
        SessionManager::new(DATABASE, cm, config)
            .await
            .unwrap()
            .get()
            .await
            .unwrap()
    }

    async fn replace_test_data(
        session: &mut SessionHandle,
        user_id: &str,
    ) -> Result<CommitResponse, tonic::Status> {
        session
            .spanner_client
            .commit(
                CommitRequest {
                    session: session.session.name.to_string(),
                    mutations: vec![insert_or_update(
                        "User",
                        vec![
                            "UserId",
                            "NotNullINT64",
                            "NullableINT64",
                            "NotNullFloat64",
                            "NullableFloat64",
                            "NotNullBool",
                            "NullableBool",
                            "NotNullByteArray",
                            "NullableByteArray",
                            "NotNullNumeric",
                            "NullableNumeric",
                            "NotNullTimestamp",
                            "NullableTimestamp",
                            "NotNullDate",
                            "NullableDate",
                            "NotNullArray",
                            "NullableArray",
                            "NullableString",
                            "UpdatedAt",
                        ],
                        vec![
                            user_id.to_kind(),
                            1.to_kind(),
                            None::<i64>.to_kind(),
                            1.0.to_kind(),
                            None::<f64>.to_kind(),
                            true.to_kind(),
                            None::<bool>.to_kind(),
                            vec![1 as u8].to_kind(),
                            None::<Vec<u8>>.to_kind(),
                            rust_decimal::Decimal::from_str("100.24").unwrap().to_kind(),
                            Some(rust_decimal::Decimal::from_str("1000.42342").unwrap()).to_kind(),
                            Utc::now().naive_utc().to_kind(),
                            Some(Utc::now().naive_utc()).to_kind(),
                            Utc::now().date().naive_utc().to_kind(),
                            None::<NaiveDate>.to_kind(),
                            vec![10 as i64, 20 as i64, 30 as i64].to_kind(),
                            None::<Vec<i64>>.to_kind(),
                            Some(user_id).to_kind(),
                            CommitTimestamp::new().to_kind(),
                        ],
                    )],
                    return_commit_stats: false,
                    request_options: None,
                    transaction: Some(SingleUseTransaction(TransactionOptions {
                        mode: Some(Mode::ReadWrite(ReadWrite {})),
                    })),
                },
                None,
            )
            .await
            .map(|x| x.into_inner())
    }

    #[tokio::test]
    async fn test_query() {
        let mut session = create_session().await;
        let user_id = "user_1";
        replace_test_data(session.deref_mut(), user_id)
            .await
            .unwrap();

        let mut tx = match ReadOnlyTransaction::begin(
            session,
            TimestampBound::strong_read(),
            CallOptions::default(),
        )
        .await
        {
            Ok(tx) => tx,
            Err(status) => panic!("begin error {:?}", status),
        };

        let mut stmt = Statement::new("SELECT * FROM User WHERE UserId = @UserID");
        stmt.add_param("UserId", user_id);
        let mut reader = match tx.query(stmt, Some(QueryOptions::default())).await {
            Ok(tx) => tx,
            Err(status) => panic!("query error {:?}", status),
        };
        let maybe_row = match reader.next().await {
            Ok(row) => row,
            Err(status) => panic!("reader aborted {:?}", status),
        };
        assert_eq!(true, maybe_row.is_some(), "row must exists");
        match get_row(maybe_row.unwrap()) {
            Err(err) => panic!("row error {:?}", err),
            _ => {}
        }
    }

    fn get_row(row: Row) -> Result<(), anyhow::Error> {
        // get first row
        let user_id = row.column_by_name::<String>("UserId")?;
        let not_null_int64 = row.column_by_name::<i64>("NotNullINT64")?;
        let nullable_int64 = row.column_by_name::<Option<i64>>("NullableINT64")?;
        let not_null_float64 = row.column_by_name::<f64>("NotNullFloat64")?;
        let nullable_float64 = row.column_by_name::<Option<f64>>("NullableFloat64")?;
        let not_null_bool = row.column_by_name::<bool>("NotNullBool")?;
        let nullable_bool = row.column_by_name::<Option<bool>>("NullableBool")?;
        let not_null_byte_array = row.column_by_name::<Vec<u8>>("NotNullByteArray")?;
        let nullable_byte_array = row.column_by_name::<Option<Vec<u8>>>("NullableByteArray")?;
        let not_null_decimal = row.column_by_name::<rust_decimal::Decimal>("NotNullNumeric")?;
        let nullable_decimal =
            row.column_by_name::<Option<rust_decimal::Decimal>>("NullableNumeric")?;
        let not_null_ts = row.column_by_name::<NaiveDateTime>("NotNullTimestamp")?;
        let nullable_ts = row.column_by_name::<Option<NaiveDateTime>>("NullableTimestamp")?;
        let not_null_date = row.column_by_name::<NaiveDate>("NotNullDate")?;
        let nullable_date = row.column_by_name::<Option<NaiveDate>>("NullableDate")?;
        let not_null_array = row.column_by_name::<Vec<i64>>("NotNullArray")?;
        let nullable_array = row.column_by_name::<Option<Vec<i64>>>("NullableArray")?;
        let not_null_string = row.column_by_name::<Option<String>>("NullableString")?;
        let updated_at = row.column_by_name::<CommitTimestamp>("UpdatedAt")?;
        Ok(())
    }
}
