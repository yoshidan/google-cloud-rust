use gax::call_option as gax_opt;
use gcpauth;
use internal::spanner::v1 as internal;
use once_cell::sync::Lazy;
use std::any::Any;
use std::convert::TryInto;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::sync::{Mutex, OnceCell};
use tonic::metadata::{Ascii, BinaryMetadataValue, KeyAndValueRef, MetadataMap, MetadataValue};
use tonic::transport::Channel;
use tonic::{IntoRequest, Request, Response, Status, Streaming};

const SCOPES: [&'static str; 2] = [
    "https://www.googleapis.com/auth/cloud-platform",
    "https://www.googleapis.com/auth/spanner.data",
];

static AUTHENTICATOR: OnceCell<Box<dyn gcpauth::token::TokenSource>> = OnceCell::const_new();

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

// default retry call settings
fn default_setting() -> gax_opt::CallSettings {
    return gax_opt::CallSettings {
        retryer: gax_opt::BackoffRetryer {
            backoff: gax_opt::Backoff::default(),
            codes: vec![tonic::Code::Unavailable],
            check_session_not_found: false,
        },
    };
}
#[derive(Clone, Debug)]
pub struct Client {
    inner: internal::spanner_client::SpannerClient<Channel>,
}

impl Client {
    // create new spanner client
    pub fn new(inner: internal::spanner_client::SpannerClient<Channel>) -> Client {
        return Client { inner };
    }

    // merge call setting
    fn get_call_setting(call_setting: Option<gax_opt::CallSettings>) -> gax_opt::CallSettings {
        match call_setting {
            Some(s) => s,
            None => default_setting(),
        }
    }

    // CreateSession creates a new session. A session can be used to perform
    // transactions that read and/or modify data in a Cloud Spanner database.
    // Sessions are meant to be reused for many consecutive
    // transactions.
    //
    // Sessions can only execute one transaction at a time. To execute
    // multiple concurrent read-write/write-only transactions, create
    // multiple sessions. Note that standalone reads and queries use a
    // transaction internally, and count toward the one transaction
    // limit.
    //
    // Active sessions use additional server resources, so it is a good idea to
    // delete idle and unneeded sessions.
    // Aside from explicit deletes, Cloud Spanner may delete sessions for which no
    // operations are sent for more than an hour. If a session is deleted,
    // requests to it return NOT_FOUND.
    //
    // Idle sessions can be kept alive by sending a trivial SQL query
    // periodically, e.g., "SELECT 1".
    pub async fn create_session(
        &mut self,
        req: internal::CreateSessionRequest,
        opt: Option<gax_opt::CallSettings>,
    ) -> Result<tonic::Response<internal::Session>, tonic::Status> {
        let mut setting = Client::get_call_setting(opt);
        let database = &req.database;
        let token = get_token().await;
        return gax::invoke::invoke_reuse(
            |spanner_client| async {
                let request = create_request(
                    format!("database={}", database),
                    token.as_str(),
                    req.clone().into_request(),
                );
                return match spanner_client.create_session(request).await {
                    Ok(o) => Ok((o, spanner_client)),
                    Err(o) => Err((o, spanner_client)),
                };
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    // BatchCreateSessions creates multiple new sessions.
    //
    // This API can be used to initialize a session cache on the clients.
    // See https://goo.gl/TgSFN2 (at https://goo.gl/TgSFN2) for best practices on session cache management.
    pub async fn batch_create_sessions(
        &mut self,
        req: internal::BatchCreateSessionsRequest,
        opt: Option<gax_opt::CallSettings>,
    ) -> Result<tonic::Response<internal::BatchCreateSessionsResponse>, tonic::Status> {
        let mut setting = Client::get_call_setting(opt);
        let database = &req.database;
        let token = get_token().await;
        return gax::invoke::invoke_reuse(
            |spanner_client| async {
                let request = create_request(
                    format!("database={}", database),
                    token.as_str(),
                    req.clone().into_request(),
                );
                return match spanner_client.batch_create_sessions(request).await {
                    Ok(o) => Ok((o, spanner_client)),
                    Err(o) => Err((o, spanner_client)),
                };
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    // GetSession gets a session. Returns NOT_FOUND if the session does not exist.
    // This is mainly useful for determining whether a session is still
    // alive.
    pub async fn get_session(
        &mut self,
        req: internal::GetSessionRequest,
        opt: Option<gax_opt::CallSettings>,
    ) -> Result<tonic::Response<internal::Session>, tonic::Status> {
        let mut setting = Client::get_call_setting(opt);
        let name = &req.name;
        let token = get_token().await;
        return gax::invoke::invoke_reuse(
            |spanner_client| async {
                let request = create_request(
                    format!("name={}", name),
                    token.as_str(),
                    req.clone().into_request(),
                );
                return match spanner_client.get_session(request).await {
                    Ok(o) => Ok((o, spanner_client)),
                    Err(o) => Err((o, spanner_client)),
                };
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    // ListSessions lists all sessions in a given database.
    pub async fn list_sessions(
        &mut self,
        req: internal::ListSessionsRequest,
        opt: Option<gax_opt::CallSettings>,
    ) -> Result<tonic::Response<internal::ListSessionsResponse>, tonic::Status> {
        let mut setting = Client::get_call_setting(opt);
        let database = &req.database;
        let token = get_token().await;
        return gax::invoke::invoke_reuse(
            |spanner_client| async {
                let request = create_request(
                    format!("database={}", database),
                    token.as_str(),
                    req.clone().into_request(),
                );
                return match spanner_client.list_sessions(request).await {
                    Ok(o) => Ok((o, spanner_client)),
                    Err(o) => Err((o, spanner_client)),
                };
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    // DeleteSession ends a session, releasing server resources associated with it. This will
    // asynchronously trigger cancellation of any operations that are running with
    // this session.
    pub async fn delete_session(
        &mut self,
        req: internal::DeleteSessionRequest,
        opt: Option<gax_opt::CallSettings>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let mut setting = Client::get_call_setting(opt);
        let name = &req.name;
        let token = get_token().await;
        return gax::invoke::invoke_reuse(
            |spanner_client| async {
                let request = create_request(
                    format!("name={}", name),
                    token.as_str(),
                    req.clone().into_request(),
                );
                return match spanner_client.delete_session(request).await {
                    Ok(o) => Ok((o, spanner_client)),
                    Err(o) => Err((o, spanner_client)),
                };
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    // ExecuteSql executes an SQL statement, returning all results in a single reply. This
    // method cannot be used to return a result set larger than 10 MiB;
    // if the query yields more data than that, the query fails with
    // a FAILED_PRECONDITION error.
    //
    // Operations inside read-write transactions might return ABORTED. If
    // this occurs, the application should restart the transaction from
    // the beginning. See Transaction for more details.
    //
    // Larger result sets can be fetched in streaming fashion by calling
    // ExecuteStreamingSql instead.
    pub async fn execute_sql(
        &mut self,
        req: internal::ExecuteSqlRequest,
        opt: Option<gax_opt::CallSettings>,
    ) -> Result<Response<internal::ResultSet>, tonic::Status> {
        let mut setting = Client::get_call_setting(opt);
        let session = &req.session;
        let token = get_token().await;
        return gax::invoke::invoke_reuse(
            |spanner_client| async {
                let request = create_request(
                    format!("session={}", session),
                    token.as_str(),
                    req.clone().into_request(),
                );
                return match spanner_client.execute_sql(request).await {
                    Ok(o) => Ok((o, spanner_client)),
                    Err(o) => Err((o, spanner_client)),
                };
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    // ExecuteStreamingSql like ExecuteSql, except returns the result
    // set as a stream. Unlike ExecuteSql, there
    // is no limit on the size of the returned result set. However, no
    // individual row in the result set can exceed 100 MiB, and no
    // column value can exceed 10 MiB.
    pub async fn execute_streaming_sql(
        &mut self,
        req: internal::ExecuteSqlRequest,
        opt: Option<gax_opt::CallSettings>,
    ) -> Result<Response<Streaming<internal::PartialResultSet>>, Status> {
        let mut setting = Client::get_call_setting(opt);
        let session = &req.session;
        let token = get_token().await;
        return gax::invoke::invoke_reuse(
            |spanner_client| async {
                let request = create_request(
                    format!("session={}", session),
                    token.as_str(),
                    req.clone().into_request(),
                );
                return match spanner_client.execute_streaming_sql(request).await {
                    Ok(o) => Ok((o, spanner_client)),
                    Err(o) => Err((o, spanner_client)),
                };
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    // ExecuteBatchDml executes a batch of SQL DML statements. This method allows many statements
    // to be run with lower latency than submitting them sequentially with
    // ExecuteSql.
    //
    // Statements are executed in sequential order. A request can succeed even if
    // a statement fails. The ExecuteBatchDmlResponse.status field in the
    // response provides information about the statement that failed. Clients must
    // inspect this field to determine whether an error occurred.
    //
    // Execution stops after the first failed statement; the remaining statements
    // are not executed.
    pub async fn execute_batch_dml(
        &mut self,
        req: internal::ExecuteBatchDmlRequest,
        opt: Option<gax_opt::CallSettings>,
    ) -> Result<Response<internal::ExecuteBatchDmlResponse>, Status> {
        let mut setting = Client::get_call_setting(opt);
        let session = &req.session;
        let token = get_token().await;
        return gax::invoke::invoke_reuse(
            |spanner_client| async {
                let request = create_request(
                    format!("session={}", session),
                    token.as_str(),
                    req.clone().into_request(),
                );
                return match spanner_client.execute_batch_dml(request).await {
                    Ok(o) => Ok((o, spanner_client)),
                    Err(o) => Err((o, spanner_client)),
                };
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    // Read reads rows from the database using key lookups and scans, as a
    // simple key/value style alternative to
    // ExecuteSql.  This method cannot be used to
    // return a result set larger than 10 MiB; if the read matches more
    // data than that, the read fails with a FAILED_PRECONDITION
    // error.
    //
    // Reads inside read-write transactions might return ABORTED. If
    // this occurs, the application should restart the transaction from
    // the beginning. See Transaction for more details.
    //
    // Larger result sets can be yielded in streaming fashion by calling
    // StreamingRead instead.
    pub async fn read(
        &mut self,
        req: internal::ReadRequest,
        opt: Option<gax_opt::CallSettings>,
    ) -> Result<Response<internal::ResultSet>, Status> {
        let mut setting = Client::get_call_setting(opt);
        let session = &req.session;
        let token = get_token().await;
        return gax::invoke::invoke_reuse(
            |spanner_client| async {
                let request = create_request(
                    format!("session={}", session),
                    token.as_str(),
                    req.clone().into_request(),
                );
                return match spanner_client.read(request).await {
                    Ok(o) => Ok((o, spanner_client)),
                    Err(o) => Err((o, spanner_client)),
                };
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    // StreamingRead like Read, except returns the result set as a
    // stream. Unlike Read, there is no limit on the
    // size of the returned result set. However, no individual row in
    // the result set can exceed 100 MiB, and no column value can exceed
    // 10 MiB.
    pub async fn streaming_read(
        &mut self,
        req: internal::ReadRequest,
        opt: Option<gax_opt::CallSettings>,
    ) -> Result<Response<Streaming<internal::PartialResultSet>>, Status> {
        let mut setting = Client::get_call_setting(opt);
        let session = &req.session;
        let token = get_token().await;
        return gax::invoke::invoke_reuse(
            |spanner_client| async {
                let request = create_request(
                    format!("session={}", session),
                    token.as_str(),
                    req.clone().into_request(),
                );
                return match spanner_client.streaming_read(request).await {
                    Ok(o) => Ok((o, spanner_client)),
                    Err(o) => Err((o, spanner_client)),
                };
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    // BeginTransaction begins a new transaction. This step can often be skipped:
    // Read, ExecuteSql and
    // Commit can begin a new transaction as a
    // side-effect.
    pub async fn begin_transaction(
        &mut self,
        req: internal::BeginTransactionRequest,
        opt: Option<gax_opt::CallSettings>,
    ) -> Result<Response<internal::Transaction>, Status> {
        let mut setting = Client::get_call_setting(opt);
        let session = &req.session;
        let token = get_token().await;
        return gax::invoke::invoke_reuse(
            |spanner_client| async {
                let request = create_request(
                    format!("session={}", session),
                    token.as_str(),
                    req.clone().into_request(),
                );
                return match spanner_client.begin_transaction(request).await {
                    Ok(o) => Ok((o, spanner_client)),
                    Err(o) => Err((o, spanner_client)),
                };
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    // Commit commits a transaction. The request includes the mutations to be
    // applied to rows in the database.
    //
    // Commit might return an ABORTED error. This can occur at any time;
    // commonly, the cause is conflicts with concurrent
    // transactions. However, it can also happen for a variety of other
    // reasons. If Commit returns ABORTED, the caller should re-attempt
    // the transaction from the beginning, re-using the same session.
    //
    // On very rare occasions, Commit might return UNKNOWN. This can happen,
    // for example, if the client job experiences a 1+ hour networking failure.
    // At that point, Cloud Spanner has lost track of the transaction outcome and
    // we recommend that you perform another read from the database to see the
    // state of things as they are now.
    pub async fn commit(
        &mut self,
        req: internal::CommitRequest,
        opt: Option<gax_opt::CallSettings>,
    ) -> Result<Response<internal::CommitResponse>, Status> {
        let mut setting = Client::get_call_setting(opt);
        let session = &req.session;
        let token = get_token().await;
        return gax::invoke::invoke_reuse(
            |spanner_client| async {
                let request = create_request(
                    format!("session={}", session),
                    token.as_str(),
                    req.clone().into_request(),
                );
                return match spanner_client.commit(request).await {
                    Ok(o) => Ok((o, spanner_client)),
                    Err(o) => Err((o, spanner_client)),
                };
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    // Rollback rolls back a transaction, releasing any locks it holds. It is a good
    // idea to call this for any transaction that includes one or more
    // Read or ExecuteSql requests and
    // ultimately decides not to commit.
    //
    // Rollback returns OK if it successfully aborts the transaction, the
    // transaction was already aborted, or the transaction is not
    // found. Rollback never returns ABORTED.
    pub async fn rollback(
        &mut self,
        req: internal::RollbackRequest,
        opt: Option<gax_opt::CallSettings>,
    ) -> Result<Response<()>, Status> {
        let mut setting = Client::get_call_setting(opt);
        let session = &req.session;
        let token = get_token().await;
        return gax::invoke::invoke_reuse(
            |spanner_client| async {
                let request = create_request(
                    format!("session={}", session),
                    token.as_str(),
                    req.clone().into_request(),
                );
                return match spanner_client.rollback(request).await {
                    Ok(o) => Ok((o, spanner_client)),
                    Err(o) => Err((o, spanner_client)),
                };
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    // PartitionQuery creates a set of partition tokens that can be used to execute a query
    // operation in parallel.  Each of the returned partition tokens can be used
    // by ExecuteStreamingSql to specify a subset
    // of the query result to read.  The same session and read-only transaction
    // must be used by the PartitionQueryRequest used to create the
    // partition tokens and the ExecuteSqlRequests that use the partition tokens.
    //
    // Partition tokens become invalid when the session used to create them
    // is deleted, is idle for too long, begins a new transaction, or becomes too
    // old.  When any of these happen, it is not possible to resume the query, and
    // the whole operation must be restarted from the beginning.
    pub async fn partition_query(
        &mut self,
        req: internal::PartitionQueryRequest,
        opt: Option<gax_opt::CallSettings>,
    ) -> Result<Response<internal::PartitionResponse>, Status> {
        let mut setting = Client::get_call_setting(opt);
        let session = &req.session;
        let token = get_token().await;
        return gax::invoke::invoke_reuse(
            |spanner_client| async {
                let request = create_request(
                    format!("session={}", session),
                    token.as_str(),
                    req.clone().into_request(),
                );
                return match spanner_client.partition_query(request).await {
                    Ok(o) => Ok((o, spanner_client)),
                    Err(o) => Err((o, spanner_client)),
                };
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    // PartitionRead creates a set of partition tokens that can be used to execute a read
    // operation in parallel.  Each of the returned partition tokens can be used
    // by StreamingRead to specify a subset of the read
    // result to read.  The same session and read-only transaction must be used by
    // the PartitionReadRequest used to create the partition tokens and the
    // ReadRequests that use the partition tokens.  There are no ordering
    // guarantees on rows returned among the returned partition tokens, or even
    // within each individual StreamingRead call issued with a partition_token.
    //
    // Partition tokens become invalid when the session used to create them
    // is deleted, is idle for too long, begins a new transaction, or becomes too
    // old.  When any of these happen, it is not possible to resume the read, and
    // the whole operation must be restarted from the beginning.
    pub async fn partition_read(
        &mut self,
        req: internal::PartitionReadRequest,
        opt: Option<gax_opt::CallSettings>,
    ) -> Result<Response<internal::PartitionResponse>, Status> {
        let mut setting = Client::get_call_setting(opt);
        let session = &req.session;
        let token = get_token().await;
        return gax::invoke::invoke_reuse(
            |spanner_client| async {
                let request = create_request(
                    format!("session={}", session),
                    token.as_str(),
                    req.clone().into_request(),
                );
                return match spanner_client.partition_read(request).await {
                    Ok(o) => Ok((o, spanner_client)),
                    Err(o) => Err((o, spanner_client)),
                };
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }
}

async fn get_token() -> String {
    let ts = AUTHENTICATOR
        .get_or_try_init(|| {
            gcpauth::create_token_source(gcpauth::Config {
                audience: Some("https://spanner.googleapis.com/"),
                scopes: Some(&SCOPES),
            })
        })
        .await
        .unwrap();
    return ts.token().await.unwrap().value();
}

fn create_request<T>(
    param_string: String,
    token: &str,
    mut request: tonic::Request<T>,
) -> tonic::Request<T> {
    let target = request.metadata_mut();
    target.append("x-goog-request-params", param_string.parse().unwrap());
    target.insert("authorization", token.parse().unwrap());
    return request;
}

fn insert_metadata(target: &mut MetadataMap, from: &MetadataMap) {
    for kv in from.iter() {
        match kv {
            KeyAndValueRef::Ascii(k, v) => {
                target.append(k, MetadataValue::from(v));
            }
            KeyAndValueRef::Binary(k, v) => {
                target.append_bin(k, MetadataValue::from(v));
            }
        }
    }
}
