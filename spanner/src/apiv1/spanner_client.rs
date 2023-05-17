use std::time::Duration;

use google_cloud_gax::conn::Channel;
use google_cloud_gax::create_request;
use google_cloud_gax::grpc::{Code, Response, Status, Streaming};
use google_cloud_gax::retry::{invoke_fn, RetrySetting};
use google_cloud_googleapis::spanner::v1::spanner_client::SpannerClient;
use google_cloud_googleapis::spanner::v1::{
    BatchCreateSessionsRequest, BatchCreateSessionsResponse, BeginTransactionRequest, CommitRequest, CommitResponse,
    CreateSessionRequest, DeleteSessionRequest, ExecuteBatchDmlRequest, ExecuteBatchDmlResponse, ExecuteSqlRequest,
    GetSessionRequest, ListSessionsRequest, ListSessionsResponse, PartialResultSet, PartitionQueryRequest,
    PartitionReadRequest, PartitionResponse, ReadRequest, ResultSet, RollbackRequest, Session, Transaction,
};

pub(crate) fn ping_query_request(session_name: impl Into<String>) -> ExecuteSqlRequest {
    ExecuteSqlRequest {
        session: session_name.into(),
        transaction: None,
        sql: "SELECT 1".to_string(),
        params: None,
        param_types: Default::default(),
        resume_token: vec![],
        query_mode: 0,
        partition_token: vec![],
        seqno: 0,
        query_options: None,
        request_options: None,
        data_boost_enabled: false,
    }
}

fn default_setting() -> RetrySetting {
    RetrySetting {
        from_millis: 50,
        max_delay: Some(Duration::from_secs(10)),
        factor: 1u64,
        take: 20,
        codes: vec![Code::Unavailable, Code::Unknown],
    }
}

#[derive(Clone)]
pub struct Client {
    inner: SpannerClient<Channel>,
}

impl Client {
    /// create new spanner client
    pub fn new(inner: SpannerClient<Channel>) -> Client {
        Client { inner }
    }

    /// create_session creates a new session. A session can be used to perform
    /// transactions that read and/or modify data in a Cloud Spanner database.
    /// Sessions are meant to be reused for many consecutive
    /// transactions.
    ///
    /// Sessions can only execute one transaction at a time. To execute
    /// multiple concurrent read-write/write-only transactions, create
    /// multiple sessions. Note that standalone reads and queries use a
    /// transaction internally, and count toward the one transaction
    /// limit.
    ///
    /// Active sessions use additional server resources, so it is a good idea to
    /// delete idle and unneeded sessions.
    /// Aside from explicit deletes, Cloud Spanner may delete sessions for which no
    /// operations are sent for more than an hour. If a session is deleted,
    /// requests to it return NOT_FOUND.
    ///
    /// Idle sessions can be kept alive by sending a trivial SQL query
    /// periodically, e.g., "SELECT 1".
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn create_session(
        &mut self,
        req: CreateSessionRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Session>, Status> {
        let setting = retry.unwrap_or_else(default_setting);
        let database = &req.database;
        invoke_fn(
            Some(setting),
            |spanner_client| async {
                let request = create_request(format!("database={database}"), req.clone());
                spanner_client
                    .create_session(request)
                    .await
                    .map_err(|e| (e, spanner_client))
            },
            &mut self.inner,
        )
        .await
    }

    /// batch_create_sessions creates multiple new sessions.
    ///
    /// This API can be used to initialize a session cache on the clients.
    /// See https:///goo.gl/TgSFN2 (at https:///goo.gl/TgSFN2) for best practices on session cache management.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn batch_create_sessions(
        &mut self,
        req: BatchCreateSessionsRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Response<BatchCreateSessionsResponse>, Status> {
        let setting = retry.unwrap_or_else(default_setting);
        let database = &req.database;
        invoke_fn(
            Some(setting),
            |spanner_client| async {
                let request = create_request(format!("database={database}"), req.clone());
                spanner_client
                    .batch_create_sessions(request)
                    .await
                    .map_err(|e| (e, spanner_client))
            },
            &mut self.inner,
        )
        .await
    }

    /// get_session gets a session. Returns NOT_FOUND if the session does not exist.
    /// This is mainly useful for determining whether a session is still alive.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_session(
        &mut self,
        req: GetSessionRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Session>, Status> {
        let setting = retry.unwrap_or_else(default_setting);
        let name = &req.name;
        invoke_fn(
            Some(setting),
            |spanner_client| async {
                let request = create_request(format!("name={name}"), req.clone());
                spanner_client
                    .get_session(request)
                    .await
                    .map_err(|e| (e, spanner_client))
            },
            &mut self.inner,
        )
        .await
    }

    /// list_sessions lists all sessions in a given database.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn list_sessions(
        &mut self,
        req: ListSessionsRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Response<ListSessionsResponse>, Status> {
        let setting = retry.unwrap_or_else(default_setting);
        let database = &req.database;
        invoke_fn(
            Some(setting),
            |spanner_client| async {
                let request = create_request(format!("database={database}"), req.clone());
                spanner_client
                    .list_sessions(request)
                    .await
                    .map_err(|e| (e, spanner_client))
            },
            &mut self.inner,
        )
        .await
    }

    /// delete_session ends a session, releasing server resources associated with it. This will
    /// asynchronously trigger cancellation of any operations that are running with
    /// this session.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn delete_session(
        &mut self,
        req: DeleteSessionRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Response<()>, Status> {
        let setting = retry.unwrap_or_else(default_setting);
        let name = &req.name;
        invoke_fn(
            Some(setting),
            |spanner_client| async {
                let request = create_request(format!("name={name}"), req.clone());
                spanner_client
                    .delete_session(request)
                    .await
                    .map_err(|e| (e, spanner_client))
            },
            &mut self.inner,
        )
        .await
    }

    /// execute_sql executes an SQL statement, returning all results in a single reply. This
    /// method cannot be used to return a result set larger than 10 MiB;
    /// if the query yields more data than that, the query fails with
    /// a FAILED_PRECONDITION error.
    ///
    /// Operations inside read-write transactions might return ABORTED. If
    /// this occurs, the application should restart the transaction from
    /// the beginning. See Transaction for more details.
    ///
    /// Larger result sets can be fetched in streaming fashion by calling
    /// ExecuteStreamingSql instead.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn execute_sql(
        &mut self,
        req: ExecuteSqlRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Response<ResultSet>, Status> {
        let setting = retry.unwrap_or_else(default_setting);
        let session = &req.session;
        invoke_fn(
            Some(setting),
            |spanner_client| async {
                let request = create_request(format!("session={session}"), req.clone());
                spanner_client
                    .execute_sql(request)
                    .await
                    .map_err(|e| (e, spanner_client))
            },
            &mut self.inner,
        )
        .await
    }

    /// execute_streaming_sql like ExecuteSql, except returns the result
    /// set as a stream. Unlike ExecuteSql, there
    /// is no limit on the size of the returned result set. However, no
    /// individual row in the result set can exceed 100 MiB, and no
    /// column value can exceed 10 MiB.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn execute_streaming_sql(
        &mut self,
        req: ExecuteSqlRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Streaming<PartialResultSet>>, Status> {
        let setting = retry.unwrap_or_else(default_setting);
        let session = &req.session;
        invoke_fn(
            Some(setting),
            |spanner_client| async {
                let request = create_request(format!("session={session}"), req.clone());
                spanner_client
                    .execute_streaming_sql(request)
                    .await
                    .map_err(|e| (e, spanner_client))
            },
            &mut self.inner,
        )
        .await
    }

    /// execute_batch_dml executes a batch of SQL DML statements. This method allows many statements
    /// to be run with lower latency than submitting them sequentially with
    /// ExecuteSql.
    ///
    /// Statements are executed in sequential order. A request can succeed even if
    /// a statement fails. The ExecuteBatchDmlResponse.status field in the
    /// response provides information about the statement that failed. Clients must
    /// inspect this field to determine whether an error occurred.
    ///
    /// Execution stops after the first failed statement; the remaining statements
    /// are not executed.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn execute_batch_dml(
        &mut self,
        req: ExecuteBatchDmlRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Response<ExecuteBatchDmlResponse>, Status> {
        let setting = retry.unwrap_or_else(default_setting);
        let session = &req.session;
        invoke_fn(
            Some(setting),
            |spanner_client| async {
                let request = create_request(format!("session={session}"), req.clone());
                let result = spanner_client.execute_batch_dml(request).await;
                match result {
                    Ok(response) => match response.get_ref().status.as_ref() {
                        Some(s) => {
                            let code = Code::from(s.code);
                            if code == Code::Ok {
                                Ok(response)
                            } else {
                                Err((Status::new(code, s.message.to_string()), spanner_client))
                            }
                        }
                        None => Ok(response),
                    },
                    Err(err) => Err((err, spanner_client)),
                }
            },
            &mut self.inner,
        )
        .await
    }

    /// read reads rows from the database using key lookups and scans, as a
    /// simple key/value style alternative to
    /// ExecuteSql.  This method cannot be used to
    /// return a result set larger than 10 MiB; if the read matches more
    /// data than that, the read fails with a FAILED_PRECONDITION
    /// error.
    ///
    /// Reads inside read-write transactions might return ABORTED. If
    /// this occurs, the application should restart the transaction from
    /// the beginning. See Transaction for more details.
    ///
    /// Larger result sets can be yielded in streaming fashion by calling
    /// StreamingRead instead.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn read(&mut self, req: ReadRequest, retry: Option<RetrySetting>) -> Result<Response<ResultSet>, Status> {
        let setting = retry.unwrap_or_else(default_setting);
        let session = &req.session;
        invoke_fn(
            Some(setting),
            |spanner_client| async {
                let request = create_request(format!("session={session}"), req.clone());
                spanner_client.read(request).await.map_err(|e| (e, spanner_client))
            },
            &mut self.inner,
        )
        .await
    }

    /// streaming_read like read, except returns the result set as a
    /// stream. Unlike read, there is no limit on the
    /// size of the returned result set. However, no individual row in
    /// the result set can exceed 100 MiB, and no column value can exceed
    /// 10 MiB.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn streaming_read(
        &mut self,
        req: ReadRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Streaming<PartialResultSet>>, Status> {
        let setting = retry.unwrap_or_else(default_setting);
        let session = &req.session;
        invoke_fn(
            Some(setting),
            |spanner_client| async {
                let request = create_request(format!("session={session}"), req.clone());
                spanner_client
                    .streaming_read(request)
                    .await
                    .map_err(|e| (e, spanner_client))
            },
            &mut self.inner,
        )
        .await
    }

    /// BeginTransaction begins a new transaction. This step can often be skipped:
    /// Read, ExecuteSql and
    /// Commit can begin a new transaction as a
    /// side-effect.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn begin_transaction(
        &mut self,
        req: BeginTransactionRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Transaction>, Status> {
        let setting = retry.unwrap_or_else(default_setting);
        let session = &req.session;
        invoke_fn(
            Some(setting),
            |spanner_client| async {
                let request = create_request(format!("session={session}"), req.clone());
                spanner_client
                    .begin_transaction(request)
                    .await
                    .map_err(|e| (e, spanner_client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Commit commits a transaction. The request includes the mutations to be
    /// applied to rows in the database.
    ///
    /// Commit might return an ABORTED error. This can occur at any time;
    /// commonly, the cause is conflicts with concurrent
    /// transactions. However, it can also happen for a variety of other
    /// reasons. If Commit returns ABORTED, the caller should re-attempt
    /// the transaction from the beginning, re-using the same session.
    ///
    /// On very rare occasions, Commit might return UNKNOWN. This can happen,
    /// for example, if the client job experiences a 1+ hour networking failure.
    /// At that point, Cloud Spanner has lost track of the transaction outcome and
    /// we recommend that you perform another read from the database to see the
    /// state of things as they are now.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn commit(
        &mut self,
        req: CommitRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Response<CommitResponse>, Status> {
        let setting = retry.unwrap_or_else(default_setting);
        let session = &req.session;
        invoke_fn(
            Some(setting),
            |spanner_client| async {
                let request = create_request(format!("session={session}"), req.clone());
                spanner_client.commit(request).await.map_err(|e| (e, spanner_client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Rollback rolls back a transaction, releasing any locks it holds. It is a good
    /// idea to call this for any transaction that includes one or more
    /// Read or ExecuteSql requests and
    /// ultimately decides not to commit.
    ///
    /// Rollback returns OK if it successfully aborts the transaction, the
    /// transaction was already aborted, or the transaction is not
    /// found. Rollback never returns ABORTED.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn rollback(
        &mut self,
        req: RollbackRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Response<()>, Status> {
        let setting = retry.unwrap_or_else(default_setting);
        let session = &req.session;
        invoke_fn(
            Some(setting),
            |spanner_client| async {
                let request = create_request(format!("session={session}"), req.clone());
                spanner_client.rollback(request).await.map_err(|e| (e, spanner_client))
            },
            &mut self.inner,
        )
        .await
    }

    /// PartitionQuery creates a set of partition tokens that can be used to execute a query
    /// operation in parallel.  Each of the returned partition tokens can be used
    /// by ExecuteStreamingSql to specify a subset
    /// of the query result to read.  The same session and read-only transaction
    /// must be used by the PartitionQueryRequest used to create the
    /// partition tokens and the ExecuteSqlRequests that use the partition tokens.
    ///
    /// Partition tokens become invalid when the session used to create them
    /// is deleted, is idle for too long, begins a new transaction, or becomes too
    /// old.  When any of these happen, it is not possible to resume the query, and
    /// the whole operation must be restarted from the beginning.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn partition_query(
        &mut self,
        req: PartitionQueryRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Response<PartitionResponse>, Status> {
        let setting = retry.unwrap_or_else(default_setting);
        let session = &req.session;
        invoke_fn(
            Some(setting),
            |spanner_client| async {
                let request = create_request(format!("session={session}"), req.clone());
                spanner_client
                    .partition_query(request)
                    .await
                    .map_err(|e| (e, spanner_client))
            },
            &mut self.inner,
        )
        .await
    }

    /// PartitionRead creates a set of partition tokens that can be used to execute a read
    /// operation in parallel.  Each of the returned partition tokens can be used
    /// by StreamingRead to specify a subset of the read
    /// result to read.  The same session and read-only transaction must be used by
    /// the PartitionReadRequest used to create the partition tokens and the
    /// ReadRequests that use the partition tokens.  There are no ordering
    /// guarantees on rows returned among the returned partition tokens, or even
    /// within each individual StreamingRead call issued with a partition_token.
    ///
    /// Partition tokens become invalid when the session used to create them
    /// is deleted, is idle for too long, begins a new transaction, or becomes too
    /// old.  When any of these happen, it is not possible to resume the read, and
    /// the whole operation must be restarted from the beginning.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn partition_read(
        &mut self,
        req: PartitionReadRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Response<PartitionResponse>, Status> {
        let setting = retry.unwrap_or_else(default_setting);
        let session = &req.session;
        invoke_fn(
            Some(setting),
            |spanner_client| async {
                let request = create_request(format!("session={session}"), req.clone());
                spanner_client
                    .partition_read(request)
                    .await
                    .map_err(|e| (e, spanner_client))
            },
            &mut self.inner,
        )
        .await
    }
}
