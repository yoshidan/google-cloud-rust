use std::sync::Arc;

use tonic::transport::Channel;
use tonic::{IntoRequest, Request, Response, Status, Streaming};

use google_cloud_auth::error::Error;
use google_cloud_auth::token::Token;
use google_cloud_auth::token_source::token_source::TokenSource;
use google_cloud_gax::call_option::{Backoff, BackoffRetrySettings, BackoffRetryer};
use google_cloud_gax::invoke::invoke_reuse;
use google_cloud_googleapis::spanner::v1 as internal;
use google_cloud_googleapis::spanner::v1::spanner_client::SpannerClient;
use google_cloud_googleapis::spanner::v1::{
    BatchCreateSessionsRequest, BatchCreateSessionsResponse, BeginTransactionRequest,
    CommitRequest, CommitResponse, CreateSessionRequest, DeleteSessionRequest,
    ExecuteBatchDmlRequest, ExecuteBatchDmlResponse, ExecuteSqlRequest, GetSessionRequest,
    ListSessionsRequest, ListSessionsResponse, PartialResultSet, PartitionQueryRequest,
    PartitionReadRequest, PartitionResponse, ReadRequest, ResultSet, RollbackRequest, Session,
    Transaction,
};

pub(crate) fn ping_query_request(session_name: impl Into<String>) -> internal::ExecuteSqlRequest {
    internal::ExecuteSqlRequest {
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
    }
}

fn default_setting() -> BackoffRetrySettings {
    return BackoffRetrySettings {
        retryer: BackoffRetryer {
            backoff: Backoff::default(),
            codes: vec![tonic::Code::Unavailable],
        },
    };
}

#[derive(Clone)]
pub struct Client {
    inner: SpannerClient<Channel>,
    token_source: Arc<dyn TokenSource>,
}

impl Client {
    /// create new spanner client
    pub fn new(inner: SpannerClient<Channel>, token_source: Arc<dyn TokenSource>) -> Client {
        return Client {
            inner,
            token_source,
        };
    }

    /// merge call setting
    fn get_call_setting(call_setting: Option<BackoffRetrySettings>) -> BackoffRetrySettings {
        match call_setting {
            Some(s) => s,
            None => default_setting(),
        }
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
    pub async fn create_session(
        &mut self,
        req: CreateSessionRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<Session>, Status> {
        let mut setting = Client::get_call_setting(opt);
        let database = &req.database;
        let token = map_token_error(self.token_source.token().await)?;
        return invoke_reuse(
            |spanner_client| async {
                let request = create_request(
                    format!("database={}", database),
                    token.as_str(),
                    req.clone(),
                );
                spanner_client
                    .create_session(request)
                    .await
                    .map_err(|e| (e, spanner_client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// batch_create_sessions creates multiple new sessions.
    ///
    /// This API can be used to initialize a session cache on the clients.
    /// See https:///goo.gl/TgSFN2 (at https:///goo.gl/TgSFN2) for best practices on session cache management.
    pub async fn batch_create_sessions(
        &mut self,
        req: BatchCreateSessionsRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<tonic::Response<BatchCreateSessionsResponse>, Status> {
        let mut setting = Client::get_call_setting(opt);
        let database = &req.database;
        let token = map_token_error(self.token_source.token().await)?;
        return invoke_reuse(
            |spanner_client| async {
                let request = create_request(
                    format!("database={}", database),
                    token.as_str(),
                    req.clone(),
                );
                spanner_client
                    .batch_create_sessions(request)
                    .await
                    .map_err(|e| (e, spanner_client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// get_session gets a session. Returns NOT_FOUND if the session does not exist.
    /// This is mainly useful for determining whether a session is still
    /// alive.
    pub async fn get_session(
        &mut self,
        req: GetSessionRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<Session>, Status> {
        let mut setting = Client::get_call_setting(opt);
        let name = &req.name;
        let token = map_token_error(self.token_source.token().await)?;
        return invoke_reuse(
            |spanner_client| async {
                let request = create_request(format!("name={}", name), token.as_str(), req.clone());
                spanner_client
                    .get_session(request)
                    .await
                    .map_err(|e| (e, spanner_client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// list_sessions lists all sessions in a given database.
    pub async fn list_sessions(
        &mut self,
        req: ListSessionsRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<ListSessionsResponse>, Status> {
        let mut setting = Client::get_call_setting(opt);
        let database = &req.database;
        let token = map_token_error(self.token_source.token().await)?;
        return invoke_reuse(
            |spanner_client| async {
                let request = create_request(
                    format!("database={}", database),
                    token.as_str(),
                    req.clone(),
                );
                spanner_client
                    .list_sessions(request)
                    .await
                    .map_err(|e| (e, spanner_client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// delete_session ends a session, releasing server resources associated with it. This will
    /// asynchronously trigger cancellation of any operations that are running with
    /// this session.
    pub async fn delete_session(
        &mut self,
        req: DeleteSessionRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<()>, Status> {
        let mut setting = Client::get_call_setting(opt);
        let name = &req.name;
        let token = map_token_error(self.token_source.token().await)?;
        return invoke_reuse(
            |spanner_client| async {
                let request = create_request(format!("name={}", name), token.as_str(), req.clone());
                spanner_client
                    .delete_session(request)
                    .await
                    .map_err(|e| (e, spanner_client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
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
    pub async fn execute_sql(
        &mut self,
        req: ExecuteSqlRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<ResultSet>, Status> {
        let mut setting = Client::get_call_setting(opt);
        let session = &req.session;
        let token = map_token_error(self.token_source.token().await)?;
        return invoke_reuse(
            |spanner_client| async {
                let request =
                    create_request(format!("session={}", session), token.as_str(), req.clone());
                spanner_client
                    .execute_sql(request)
                    .await
                    .map_err(|e| (e, spanner_client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// execute_streaming_sql like ExecuteSql, except returns the result
    /// set as a stream. Unlike ExecuteSql, there
    /// is no limit on the size of the returned result set. However, no
    /// individual row in the result set can exceed 100 MiB, and no
    /// column value can exceed 10 MiB.
    pub async fn execute_streaming_sql(
        &mut self,
        req: ExecuteSqlRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<Streaming<PartialResultSet>>, Status> {
        let mut setting = Client::get_call_setting(opt);
        let session = &req.session;
        let token = map_token_error(self.token_source.token().await)?;
        return invoke_reuse(
            |spanner_client| async {
                let request =
                    create_request(format!("session={}", session), token.as_str(), req.clone());
                spanner_client
                    .execute_streaming_sql(request)
                    .await
                    .map_err(|e| (e, spanner_client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
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
    pub async fn execute_batch_dml(
        &mut self,
        req: ExecuteBatchDmlRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<ExecuteBatchDmlResponse>, Status> {
        let mut setting = Client::get_call_setting(opt);
        let session = &req.session;
        let token = map_token_error(self.token_source.token().await)?;
        return invoke_reuse(
            |spanner_client| async {
                let request =
                    create_request(format!("session={}", session), token.as_str(), req.clone());
                spanner_client
                    .execute_batch_dml(request)
                    .await
                    .map_err(|e| (e, spanner_client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
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
    pub async fn read(
        &mut self,
        req: ReadRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<ResultSet>, Status> {
        let mut setting = Client::get_call_setting(opt);
        let session = &req.session;
        let token = map_token_error(self.token_source.token().await)?;
        return invoke_reuse(
            |spanner_client| async {
                let request =
                    create_request(format!("session={}", session), token.as_str(), req.clone());
                spanner_client
                    .read(request)
                    .await
                    .map_err(|e| (e, spanner_client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// streaming_read like Read, except returns the result set as a
    /// stream. Unlike Read, there is no limit on the
    /// size of the returned result set. However, no individual row in
    /// the result set can exceed 100 MiB, and no column value can exceed
    /// 10 MiB.
    pub async fn streaming_read(
        &mut self,
        req: ReadRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<Streaming<PartialResultSet>>, Status> {
        let mut setting = Client::get_call_setting(opt);
        let session = &req.session;
        let token = map_token_error(self.token_source.token().await)?;
        return invoke_reuse(
            |spanner_client| async {
                let request =
                    create_request(format!("session={}", session), token.as_str(), req.clone());
                spanner_client
                    .streaming_read(request)
                    .await
                    .map_err(|e| (e, spanner_client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// BeginTransaction begins a new transaction. This step can often be skipped:
    /// Read, ExecuteSql and
    /// Commit can begin a new transaction as a
    /// side-effect.
    pub async fn begin_transaction(
        &mut self,
        req: BeginTransactionRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<Transaction>, Status> {
        let mut setting = Client::get_call_setting(opt);
        let session = &req.session;
        let token = map_token_error(self.token_source.token().await)?;
        return invoke_reuse(
            |spanner_client| async {
                let request =
                    create_request(format!("session={}", session), token.as_str(), req.clone());
                spanner_client
                    .begin_transaction(request)
                    .await
                    .map_err(|e| (e, spanner_client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
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
    pub async fn commit(
        &mut self,
        req: CommitRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<CommitResponse>, Status> {
        let mut setting = Client::get_call_setting(opt);
        let session = &req.session;
        let token = map_token_error(self.token_source.token().await)?;
        return invoke_reuse(
            |spanner_client| async {
                let request =
                    create_request(format!("session={}", session), token.as_str(), req.clone());
                spanner_client
                    .commit(request)
                    .await
                    .map_err(|e| (e, spanner_client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// Rollback rolls back a transaction, releasing any locks it holds. It is a good
    /// idea to call this for any transaction that includes one or more
    /// Read or ExecuteSql requests and
    /// ultimately decides not to commit.
    ///
    /// Rollback returns OK if it successfully aborts the transaction, the
    /// transaction was already aborted, or the transaction is not
    /// found. Rollback never returns ABORTED.
    pub async fn rollback(
        &mut self,
        req: RollbackRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<()>, Status> {
        let mut setting = Client::get_call_setting(opt);
        let session = &req.session;
        let token = map_token_error(self.token_source.token().await)?;
        return invoke_reuse(
            |spanner_client| async {
                let request =
                    create_request(format!("session={}", session), token.as_str(), req.clone());
                spanner_client
                    .rollback(request)
                    .await
                    .map_err(|e| (e, spanner_client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
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
    pub async fn partition_query(
        &mut self,
        req: PartitionQueryRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<PartitionResponse>, Status> {
        let mut setting = Client::get_call_setting(opt);
        let session = &req.session;
        let token = map_token_error(self.token_source.token().await)?;
        return invoke_reuse(
            |spanner_client| async {
                let request =
                    create_request(format!("session={}", session), token.as_str(), req.clone());
                spanner_client
                    .partition_query(request)
                    .await
                    .map_err(|e| (e, spanner_client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
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
    pub async fn partition_read(
        &mut self,
        req: PartitionReadRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<PartitionResponse>, Status> {
        let mut setting = Client::get_call_setting(opt);
        let session = &req.session;
        let token = map_token_error(self.token_source.token().await)?;
        return invoke_reuse(
            |spanner_client| async {
                let request =
                    create_request(format!("session={}", session), token.as_str(), req.clone());
                spanner_client
                    .partition_read(request)
                    .await
                    .map_err(|e| (e, spanner_client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }
}

fn map_token_error(result: Result<Token, Error>) -> Result<String, Status> {
    result
        .map_err(|e| {
            tonic::Status::new(
                tonic::Code::Unauthenticated,
                format!("token error: {:?}", e),
            )
        })
        .map(|v| v.value())
}

fn create_request<T>(
    param_string: String,
    token: &str,
    into_request: impl IntoRequest<T>,
) -> Request<T> {
    let mut request = into_request.into_request();
    let target = request.metadata_mut();
    target.append("x-goog-request-params", param_string.parse().unwrap());
    target.insert("authorization", token.parse().unwrap());
    return request;
}
