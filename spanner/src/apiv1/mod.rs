pub mod conn_pool;
pub mod spanner_client;

#[cfg(test)]
mod tests {
    use prost_types::{value::Kind, ListValue, Value};
    use serial_test::serial;

    use crate::apiv1::conn_pool::ConnectionManager;
    use crate::apiv1::spanner_client::Client;
    use crate::session::client_metadata;
    use google_cloud_gax::conn::{ConnectionOptions, Environment};
    use google_cloud_gax::grpc::Code;
    use google_cloud_googleapis::spanner::v1::mutation::{Operation, Write};
    use google_cloud_googleapis::spanner::v1::transaction_options::IsolationLevel;
    use google_cloud_googleapis::spanner::v1::{
        commit_request, transaction_options, transaction_selector, BatchCreateSessionsRequest, BeginTransactionRequest,
        CommitRequest, CreateSessionRequest, DeleteSessionRequest, ExecuteBatchDmlRequest, ExecuteSqlRequest,
        GetSessionRequest, ListSessionsRequest, PartitionQueryRequest, PartitionReadRequest, ReadRequest,
        RequestOptions, RollbackRequest, Session, Transaction, TransactionOptions, TransactionSelector,
    };
    use google_cloud_googleapis::spanner::v1::{execute_batch_dml_request, KeySet, Mutation};

    const DATABASE: &str = "projects/local-project/instances/test-instance/databases/local-database";

    async fn create_spanner_client() -> Client {
        let cm = ConnectionManager::new(
            1,
            &Environment::Emulator("localhost:9010".to_string()),
            "",
            &ConnectionOptions::default(),
        )
        .await
        .unwrap();
        cm.conn().with_metadata(client_metadata(DATABASE))
    }

    async fn create_session(client: &mut Client) -> Session {
        let session_request = CreateSessionRequest {
            database: DATABASE.to_string(),
            session: None,
        };
        let session_response = client.create_session(session_request, None).await.unwrap();
        session_response.into_inner()
    }

    async fn begin_read_only_transaction(client: &mut Client, session: &Session) -> Transaction {
        let request = BeginTransactionRequest {
            session: session.name.to_string(),
            options: Option::from(TransactionOptions {
                exclude_txn_from_change_streams: false,
                mode: Option::from(transaction_options::Mode::ReadOnly(transaction_options::ReadOnly {
                    return_read_timestamp: false,
                    timestamp_bound: None,
                })),
                isolation_level: IsolationLevel::Unspecified as i32,
            }),
            request_options: None,
            mutation_key: None,
        };
        client.begin_transaction(request, None).await.unwrap().into_inner()
    }

    async fn begin_read_write_transaction(client: &mut Client, session: &Session) -> Transaction {
        let request = BeginTransactionRequest {
            session: session.name.to_string(),
            options: Some(TransactionOptions {
                exclude_txn_from_change_streams: false,
                mode: Some(transaction_options::Mode::ReadWrite(transaction_options::ReadWrite::default())),
                isolation_level: IsolationLevel::Unspecified as i32,
            }),
            request_options: None,
            mutation_key: None,
        };
        client.begin_transaction(request, None).await.unwrap().into_inner()
    }

    #[tokio::test]
    #[serial]
    async fn test_create_session() {
        let mut client = create_spanner_client().await;
        let request = CreateSessionRequest {
            database: DATABASE.to_string(),
            session: None,
        };

        match client.create_session(request, None).await {
            Ok(res) => {
                println!("created session = {}", res.get_ref().name);
                assert!(!res.get_ref().name.is_empty());
            }
            Err(err) => panic!("err: {err:?}"),
        };
    }

    #[tokio::test]
    #[serial]
    async fn test_batch_create_session() {
        let mut client = create_spanner_client().await;
        let request = BatchCreateSessionsRequest {
            database: DATABASE.to_string(),
            session_count: 2,
            session_template: None,
        };

        match client.batch_create_sessions(request, None).await {
            Ok(res) => {
                assert_eq!(
                    res.get_ref().session.len(),
                    2,
                    "created session size = {}",
                    res.get_ref().session.len()
                );
            }
            Err(err) => panic!("err: {err:?}"),
        };
    }

    #[tokio::test]
    #[serial]
    async fn test_get_session() {
        let mut client = create_spanner_client().await;
        let session = create_session(&mut client).await;
        let request = GetSessionRequest {
            name: session.name.to_string(),
        };

        match client.get_session(request, None).await {
            Ok(res) => {
                assert_eq!(res.get_ref().name, session.name.to_string());
            }
            Err(err) => panic!("err: {err:?}"),
        };
    }

    #[tokio::test]
    #[serial]
    async fn test_list_sessions() {
        let mut client = create_spanner_client().await;
        let request = ListSessionsRequest {
            database: DATABASE.to_string(),
            page_size: 10,
            page_token: "".to_string(),
            filter: "".to_string(),
        };

        match client.list_sessions(request, None).await {
            Ok(res) => {
                println!("list session size = {}", res.get_ref().sessions.len());
            }
            Err(err) => panic!("err: {err:?}"),
        };
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_session() {
        let mut client = create_spanner_client().await;

        // create sessions
        let batch_request = BatchCreateSessionsRequest {
            database: DATABASE.to_string(),
            session_count: 2,
            session_template: None,
        };
        let session_response = client.batch_create_sessions(batch_request, None).await.unwrap();
        let sessions = &session_response.get_ref().session;

        // all delete
        for session in sessions.iter() {
            let request = DeleteSessionRequest {
                name: session.name.to_string(),
            };

            match client.delete_session(request, None).await {
                Ok(_) => {}
                Err(err) => panic!("err: {err:?}"),
            };
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_execute_sql() {
        let mut client = create_spanner_client().await;
        let session = create_session(&mut client).await;
        let request = ExecuteSqlRequest {
            session: session.name.to_string(),
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
            directed_read_options: None,
            data_boost_enabled: false,
            last_statement: false,
        };
        match client.execute_sql(request, None).await {
            Ok(res) => {
                assert_eq!(1, res.into_inner().rows.len());
            }
            Err(err) => panic!("err: {err:?}"),
        };
    }

    #[tokio::test]
    #[serial]
    async fn test_execute_streaming_sql() {
        let mut client = create_spanner_client().await;
        let session = create_session(&mut client).await;
        let mut request = ExecuteSqlRequest {
            session: session.name.to_string(),
            transaction: None,
            sql: "select 1".to_string(),
            params: None,
            param_types: Default::default(),
            resume_token: vec![],
            query_mode: 0,
            partition_token: vec![],
            seqno: 0,
            query_options: None,
            request_options: None,
            directed_read_options: None,
            data_boost_enabled: false,
            last_statement: false,
        };

        let resume_token = match client.execute_streaming_sql(request.clone(), None).await {
            Ok(res) => {
                let mut result = res.into_inner();
                if let Some(next_message) = result.message().await.unwrap() {
                    Some(next_message.resume_token)
                } else {
                    None
                }
            }
            Err(err) => panic!("err: {err:?}"),
        };
        assert!(resume_token.is_some());
        println!("resume token = {:?}", resume_token.clone().unwrap());
        request.resume_token = resume_token.unwrap();

        match client.execute_streaming_sql(request, None).await {
            Ok(res) => {
                let mut result = res.into_inner();
                assert!(!result.message().await.unwrap().unwrap().values.is_empty())
            }
            Err(err) => panic!("err: {err:?}"),
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_begin_transaction() {
        let mut client = create_spanner_client().await;
        let session = create_session(&mut client).await;
        let request = BeginTransactionRequest {
            session: session.name.to_string(),
            options: Option::from(TransactionOptions {
                exclude_txn_from_change_streams: false,
                mode: Option::from(transaction_options::Mode::ReadOnly(transaction_options::ReadOnly {
                    return_read_timestamp: false,
                    timestamp_bound: None,
                })),
                isolation_level: IsolationLevel::Unspecified as i32,
            }),
            request_options: None,
            mutation_key: None,
        };

        match client.begin_transaction(request, None).await {
            Ok(res) => {
                let tx_id = res.into_inner().id;
                println!("tx id is {tx_id:?}");
                assert!(!tx_id.is_empty());
            }
            Err(err) => panic!("err: {err:?}"),
        };
    }

    #[tokio::test]
    #[serial]
    async fn test_execute_batch_dml() {
        let mut client = create_spanner_client().await;
        let session = create_session(&mut client).await;
        let tx = begin_read_write_transaction(&mut client, &session).await;
        let request = ExecuteBatchDmlRequest {
            session: session.name.to_string(),
            transaction: Option::from(TransactionSelector {
                selector: Option::from(transaction_selector::Selector::Id(tx.id.clone())),
            }),
            statements: vec![
                execute_batch_dml_request::Statement {
                    sql: "INSERT INTO Guild (GuildId,OwnerUserId,UpdatedAt) VALUES('1', 'u1', CURRENT_TIMESTAMP())"
                        .to_string(),
                    params: None,
                    param_types: Default::default(),
                },
                execute_batch_dml_request::Statement {
                    sql: "INSERT INTO Guild (GuildId,OwnerUserId,UpdatedAt) VALUES('2', 'u2', CURRENT_TIMESTAMP())"
                        .to_string(),
                    params: None,
                    param_types: Default::default(),
                },
            ],
            seqno: 0,
            request_options: None,
            last_statements: false,
        };

        let result = client.execute_batch_dml(request, None).await;
        client
            .rollback(
                RollbackRequest {
                    session: session.name.to_string(),
                    transaction_id: tx.id,
                },
                None,
            )
            .await
            .unwrap();
        match result {
            Ok(res) => {
                let status = res.into_inner().status.unwrap();
                assert_eq!(Code::Ok, Code::from(status.code), "gRPC success but error found : {status:?}");
            }
            Err(err) => panic!("err: {err:?}"),
        };
    }

    #[tokio::test]
    #[serial]
    async fn test_execute_batch_dml_error_as_tonic_check() {
        let mut client = create_spanner_client().await;
        let session = create_session(&mut client).await;
        let tx = begin_read_write_transaction(&mut client, &session).await;
        let request = ExecuteBatchDmlRequest {
            session: session.name.to_string(),
            transaction: Option::from(TransactionSelector {
                selector: Option::from(transaction_selector::Selector::Id(tx.id.clone())),
            }),
            statements: vec![execute_batch_dml_request::Statement {
                sql: "INSERT INTO GuildX (GuildId,OwnerUserId,UpdatedAt) VALUES('1', 'u1', CURRENT_TIMESTAMP())"
                    .to_string(),
                params: None,
                param_types: Default::default(),
            }],
            seqno: 0,
            request_options: None,
            last_statements: false,
        };

        let result = client.execute_batch_dml(request, None).await;
        client
            .rollback(
                RollbackRequest {
                    session: session.name.to_string(),
                    transaction_id: tx.id,
                },
                None,
            )
            .await
            .unwrap();
        match result {
            Ok(res) => panic!("must be error code = {:?}", res.into_inner().status.unwrap().code),
            Err(status) => {
                assert_eq!(
                    Code::InvalidArgument,
                    status.code(),
                    "gRPC success but error found : {status:?}"
                );
            }
        };
    }

    #[tokio::test]
    #[serial]
    async fn test_read() {
        let mut client = create_spanner_client().await;
        let session = create_session(&mut client).await;
        let request = ReadRequest {
            session: session.name.to_string(),
            transaction: None,
            table: "Guild".to_string(),
            index: "".to_string(),
            columns: vec!["GuildId".to_string()],
            key_set: Some(KeySet {
                keys: vec![],
                ranges: vec![],
                all: true,
            }),
            resume_token: vec![],
            partition_token: vec![],
            request_options: None,
            limit: 0,
            data_boost_enabled: false,
            order_by: 0,
            directed_read_options: None,
            lock_hint: 0,
        };

        match client.read(request, None).await {
            Ok(res) => {
                println!("row size = {:?}", res.into_inner().rows.len());
            }
            Err(err) => panic!("err: {err:?}"),
        };
    }

    #[tokio::test]
    #[serial]
    async fn test_streaming_read() {
        let mut client = create_spanner_client().await;
        let session = create_session(&mut client).await;
        let request = ReadRequest {
            session: session.name.to_string(),
            transaction: None,
            table: "User".to_string(),
            index: "".to_string(),
            columns: vec!["UserId".to_string()],
            key_set: Some(KeySet {
                keys: vec![],
                ranges: vec![],
                all: true,
            }),
            resume_token: vec![],
            partition_token: vec![],
            request_options: None,
            limit: 0,
            data_boost_enabled: false,
            order_by: 0,
            directed_read_options: None,
            lock_hint: 0,
        };

        match client.streaming_read(request, None).await {
            Ok(res) => match res.into_inner().message().await {
                Ok(..) => {}
                Err(err) => panic!("err: {err:?}"),
            },
            Err(err) => panic!("err: {err:?}"),
        };
    }

    #[tokio::test]
    #[serial]
    async fn test_commit() {
        let mut client = create_spanner_client().await;
        let session = create_session(&mut client).await;
        let tx = begin_read_write_transaction(&mut client, &session).await;
        let request = CommitRequest {
            session: session.name.to_string(),
            mutations: vec![Mutation {
                operation: Some(Operation::InsertOrUpdate(Write {
                    table: "Guild".to_string(),
                    columns: vec![
                        "GuildId".to_string(),
                        "OwnerUserId".to_string(),
                        "UpdatedAt".to_string(),
                    ],
                    values: vec![ListValue {
                        values: vec![
                            Value {
                                kind: Some(Kind::StringValue("g1".to_string())),
                            },
                            Value {
                                kind: Some(Kind::StringValue("u1".to_string())),
                            },
                            Value {
                                kind: Some(Kind::StringValue("spanner.commit_timestamp()".to_string())),
                            },
                        ],
                    }],
                })),
            }],
            transaction: Option::from(commit_request::Transaction::TransactionId(tx.id)),
            request_options: Option::from(RequestOptions {
                priority: 10,
                request_tag: "".to_string(),
                transaction_tag: "".to_string(),
            }),
            return_commit_stats: false,
            max_commit_delay: None,
            precommit_token: None,
        };

        match client.commit(request, None).await {
            Ok(res) => {
                assert!(res.into_inner().commit_timestamp.is_some());
            }
            Err(err) => panic!("err: {err:?}"),
        };
    }

    #[tokio::test]
    #[serial]
    async fn test_rollback() {
        let mut client = create_spanner_client().await;
        let session = create_session(&mut client).await;
        let tx = begin_read_write_transaction(&mut client, &session).await;
        let request = RollbackRequest {
            session: session.name.to_string(),
            transaction_id: tx.id,
        };

        match client.rollback(request, None).await {
            Ok(_) => {}
            Err(err) => panic!("err: {err:?}"),
        };
    }

    #[tokio::test]
    #[serial]
    async fn test_partition_query() {
        let mut client = create_spanner_client().await;
        let session = create_session(&mut client).await;
        let tx = begin_read_only_transaction(&mut client, &session).await;
        let request = PartitionQueryRequest {
            session: session.name.to_string(),
            transaction: Option::from(TransactionSelector {
                selector: Option::from(transaction_selector::Selector::Id(tx.id)),
            }),
            sql: "SELECT * FROM User".to_string(),
            params: None,
            param_types: Default::default(),
            partition_options: None,
        };

        match client.partition_query(request, None).await {
            Ok(res) => {
                println!("partition count {:?}", res.into_inner().partitions.len());
                assert_eq!(true, true);
            }
            Err(err) => {
                println!("error code = {0}, {1}", err.code(), err.message());
                assert_eq!(false, true)
            }
        };
    }

    #[tokio::test]
    #[serial]
    async fn test_partition_read() {
        let mut client = create_spanner_client().await;
        let session = create_session(&mut client).await;
        let tx = begin_read_only_transaction(&mut client, &session).await;
        let request = PartitionReadRequest {
            session: session.name.to_string(),
            transaction: Option::from(TransactionSelector {
                selector: Option::from(transaction_selector::Selector::Id(tx.id)),
            }),
            table: "User".to_string(),
            index: "".to_string(),
            columns: vec![],
            partition_options: None,
            key_set: None,
        };

        match client.partition_read(request, None).await {
            Ok(res) => {
                println!("partition count {:?}", res.into_inner().partitions.len());
                assert_eq!(true, true);
            }
            Err(err) => {
                println!("error code = {0}, {1}", err.code(), err.message());
                assert_eq!(false, true)
            }
        };
    }
}
