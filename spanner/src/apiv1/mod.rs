pub mod conn_pool;
pub mod spanner_client;

#[cfg(test)]
mod tests {

    use crate::apiv1::conn_pool::{ConnectionManager, Error};
    use crate::apiv1::spanner_client::Client;
    use google_cloud_googleapis::spanner::v1::execute_batch_dml_request;
    use google_cloud_googleapis::spanner::v1::{
        commit_request, result_set_stats::RowCount, transaction_options, transaction_selector,
        BatchCreateSessionsRequest, BeginTransactionRequest, CommitRequest, CreateSessionRequest,
        DeleteSessionRequest, ExecuteBatchDmlRequest, ExecuteSqlRequest, GetSessionRequest,
        ListSessionsRequest, PartitionQueryRequest, PartitionReadRequest, ReadRequest,
        RequestOptions, RollbackRequest, Session, Transaction, TransactionOptions,
        TransactionSelector,
    };
    use prost_types::field_descriptor_proto::Type::Uint32;
    use prost_types::{value::Kind, ListValue, Value};
    use std::fs::File;
    use std::future::Future;
    use std::sync::Arc;
    use std::sync::Mutex;
    use tonic::{
        metadata::MetadataValue,
        transport::{Certificate, Channel, ClientTlsConfig},
        IntoRequest, Request, Response, Status,
    };

    const SCOPES: [&'static str; 2] = [
        "https://www.googleapis.com/auth/cloud-platform",
        "https://www.googleapis.com/auth/spanner.data",
    ];
    const DATABASE: &str =
        "projects/local-project/instances/test-instance/databases/local-database";

    async fn create_spanner_client() -> Client {
        let cm = ConnectionManager::new(1, Some("http://localhost:9010".to_string()))
            .await
            .unwrap();
        return cm.conn();
    }

    async fn create_session(client: &mut Client) -> Session {
        let session_request = CreateSessionRequest {
            database: DATABASE.to_string(),
            session: None,
        };
        let session_response = client.create_session(session_request, None).await.unwrap();
        return session_response.into_inner();
    }

    async fn begin_read_only_transaction(client: &mut Client, session: &Session) -> Transaction {
        let request = BeginTransactionRequest {
            session: session.name.to_string(),
            options: Option::from(TransactionOptions {
                mode: Option::from(transaction_options::Mode::ReadOnly(
                    transaction_options::ReadOnly {
                        return_read_timestamp: false,
                        timestamp_bound: None,
                    },
                )),
            }),
            request_options: None,
        };
        return client
            .begin_transaction(request, None)
            .await
            .unwrap()
            .into_inner();
    }

    async fn begin_read_write_transaction(client: &mut Client, session: &Session) -> Transaction {
        let request = BeginTransactionRequest {
            session: session.name.to_string(),
            options: Option::from(TransactionOptions {
                mode: Option::from(transaction_options::Mode::ReadWrite(
                    transaction_options::ReadWrite {},
                )),
            }),
            request_options: None,
        };
        return client
            .begin_transaction(request, None)
            .await
            .unwrap()
            .into_inner();
    }

    #[tokio::test]
    async fn test_create_session() {
        //  Init().await;
        let mut client = create_spanner_client().await;
        let request = CreateSessionRequest {
            database: DATABASE.to_string(),
            session: None,
        };

        match client.create_session(request, None).await {
            Ok(res) => {
                println!("created session = {}", res.get_ref().name);
                assert_eq!(true, true);
            }
            Err(err) => {
                println!("error code = {}", err.code());
                assert_eq!(false, true)
            }
        };
    }

    #[tokio::test]
    async fn test_batch_create_session() {
        let mut client = create_spanner_client().await;
        let request = BatchCreateSessionsRequest {
            database: DATABASE.to_string(),
            session_count: 2,
            session_template: None,
        };

        match client.batch_create_sessions(request, None).await {
            Ok(res) => {
                println!("created session size = {}", res.get_ref().session.len());
                assert_eq!(true, true);
            }
            Err(err) => {
                println!("error code = {0}, {1}", err.code(), err.message());
                assert_eq!(false, true)
            }
        };
    }

    #[tokio::test]
    async fn test_get_session() {
        let mut client = create_spanner_client().await;
        let session = create_session(&mut client).await;
        let request = GetSessionRequest {
            name: session.name.to_string(),
        };

        match client.get_session(request, None).await {
            Ok(res) => {
                println!("get session = {}", res.get_ref().name);
                assert_eq!(true, true);
            }
            Err(err) => {
                println!("error code = {0}, {1}", err.code(), err.message());
                assert_eq!(false, true)
            }
        };
    }

    #[tokio::test]
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
                assert_eq!(true, true);
            }
            Err(err) => {
                println!("error code = {0}, {1}", err.code(), err.message());
                assert_eq!(false, true)
            }
        };
    }

    #[tokio::test]
    async fn test_delete_session() {
        let mut client = create_spanner_client().await;

        // create sessions
        let batch_request = BatchCreateSessionsRequest {
            database: DATABASE.to_string(),
            session_count: 2,
            session_template: None,
        };
        let session_response = client
            .batch_create_sessions(batch_request, None)
            .await
            .unwrap();
        let sessions = &session_response.get_ref().session;

        // all delete
        for session in sessions.iter() {
            let request = DeleteSessionRequest {
                name: session.name.to_string(),
            };

            match client.delete_session(request, None).await {
                Ok(res) => {
                    println!("delete session");
                    assert_eq!(true, true);
                }
                Err(err) => {
                    println!("error code = {0}, {1}", err.code(), err.message());
                    assert_eq!(false, true)
                }
            };
        }
    }

    #[tokio::test]
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
        };
        match client.execute_sql(request, None).await {
            Ok(res) => {
                println!("row size {}", res.into_inner().rows.len());
                assert_eq!(true, true);
            }
            Err(err) => {
                println!("error code = {0}, {1}", err.code(), err.message());
                assert_eq!(false, true)
            }
        };
    }

    #[tokio::test]
    async fn test_execute_streaming_sql() {
        let mut client = create_spanner_client().await;
        let session = create_session(&mut client).await;
        let request = ExecuteSqlRequest {
            session: session.name.to_string(),
            transaction: None,
            sql: "select * from User limit 5000".to_string(),
            params: None,
            param_types: Default::default(),
            resume_token: vec![],
            query_mode: 0,
            partition_token: vec![],
            seqno: 0,
            query_options: None,
            request_options: None,
        };

        let resume_token = match client.execute_streaming_sql(request, None).await {
            Ok(res) => {
                let mut result = res.into_inner();
                if let Some(next_message) = result.message().await.unwrap() {
                    let mut counter = 0;
                    for i in next_message.values {
                        if counter > 0 {
                            break;
                        }
                        counter += 1;
                        let kind = i.kind.unwrap();
                        match kind {
                            Kind::StringValue(s) => println!("string {:?}", s),
                            Kind::BoolValue(s) => println!("bool {:?}", s),
                            Kind::NumberValue(s) => println!("number {:?}", s),
                            Kind::NullValue(s) => println!("null {:?}", s),
                            _ => {}
                        }
                    }
                    Some(next_message.resume_token)
                } else {
                    None
                }
            }
            Err(err) => {
                println!("error code = {0}, {1}", err.code(), err.message());
                assert_eq!(false, true);
                None
            }
        };
        if !resume_token.is_some() {
            assert_eq!(false, true);
            return;
        }
        println!("resume token = {:?}", resume_token.clone().unwrap());
        let request2 = ExecuteSqlRequest {
            session: session.name.to_string(),
            transaction: None,
            sql: "select * from User limit 5000 ".to_string(),
            params: None,
            param_types: Default::default(),
            resume_token: resume_token.unwrap(),
            query_mode: 0,
            partition_token: vec![],
            seqno: 0,
            query_options: None,
            request_options: None,
        };
        match client.execute_streaming_sql(request2, None).await {
            Ok(res) => {
                let mut result = res.into_inner();
                if let Some(next_message) = result.message().await.unwrap() {
                    let mut counter = 0;
                    for i in next_message.values {
                        counter += 1;
                        if counter > 1 {
                            continue;
                        }
                        let kind = i.kind.unwrap();
                        match kind {
                            Kind::StringValue(s) => println!("string {:?}", s),
                            Kind::BoolValue(s) => println!("bool {:?}", s),
                            Kind::NumberValue(s) => println!("number {:?}", s),
                            Kind::NullValue(s) => println!("null {:?}", s),
                            _ => {}
                        }
                    }
                    println!("{}", counter)
                }
            }
            Err(err) => {
                println!("error code = {0}, {1}", err.code(), err.message());
                assert_eq!(false, true);
            }
        }
    }

    #[tokio::test]
    async fn test_begin_transaction() {
        let mut client = create_spanner_client().await;
        let session = create_session(&mut client).await;
        let request = BeginTransactionRequest {
            session: session.name.to_string(),
            options: Option::from(TransactionOptions {
                mode: Option::from(transaction_options::Mode::ReadOnly(
                    transaction_options::ReadOnly {
                        return_read_timestamp: false,
                        timestamp_bound: None,
                    },
                )),
            }),
            request_options: None,
        };

        match client.begin_transaction(request, None).await {
            Ok(res) => {
                println!("tx id {:?}", res.into_inner().id);
                assert_eq!(true, true);
            }
            Err(err) => {
                println!("error code = {0}, {1}", err.code(), err.message());
                assert_eq!(false, true)
            }
        };
    }

    #[tokio::test]
    async fn test_execute_batch_dml() {
        let mut client = create_spanner_client().await;
        let session = create_session(&mut client).await;
        let tx = begin_read_write_transaction(&mut client, &session).await;
        let request = ExecuteBatchDmlRequest {
            session: session.name.to_string(),
            transaction: Option::from(TransactionSelector {
                selector: Option::from(transaction_selector::Selector::Id(tx.id)),
            }),
            statements: vec![
                execute_batch_dml_request::Statement {
                    sql: "INSERT INTO User (ID) VALUES(1)".to_string(),
                    params: None,
                    param_types: Default::default(),
                },
                execute_batch_dml_request::Statement {
                    sql: "INSERT INTO User (ID2) VALUES(1)".to_string(),
                    params: None,
                    param_types: Default::default(),
                },
                execute_batch_dml_request::Statement {
                    sql: "INSERT INTO User (ID3) VALUES(1)".to_string(),
                    params: None,
                    param_types: Default::default(),
                },
            ],
            seqno: 0,
            request_options: None,
        };

        match client.execute_batch_dml(request, None).await {
            Ok(res) => {
                let status = res.into_inner().status.unwrap();
                println!("result code {}, {}", status.code, status.message);
                assert_eq!(true, true);
            }
            Err(err) => {
                println!("error code = {0}, {1}", err.code(), err.message());
                assert_eq!(false, true)
            }
        };
    }

    #[tokio::test]
    async fn test_read() {
        let mut client = create_spanner_client().await;
        let session = create_session(&mut client).await;
        let request = ReadRequest {
            session: session.name.to_string(),
            transaction: None,
            table: "User".to_string(),
            index: "".to_string(),
            columns: vec!["UserID".to_string()],
            key_set: None,
            resume_token: vec![],
            partition_token: vec![],
            request_options: None,
            limit: 0,
        };

        match client.read(request, None).await {
            Ok(res) => {
                println!("row size{:?}", res.into_inner().rows.len());
                assert_eq!(true, true);
            }
            Err(err) => {
                println!("error code = {0}, {1}", err.code(), err.message());
                assert_eq!(false, true)
            }
        };
    }

    #[tokio::test]
    async fn test_streaming_read() {
        let mut client = create_spanner_client().await;
        let session = create_session(&mut client).await;
        let request = ReadRequest {
            session: session.name.to_string(),
            transaction: None,
            table: "User".to_string(),
            index: "".to_string(),
            columns: vec!["UserID".to_string()],
            key_set: None,
            resume_token: vec![],
            partition_token: vec![],
            request_options: None,
            limit: 0,
        };

        match client.streaming_read(request, None).await {
            Ok(res) => match res.into_inner().message().await {
                Ok(message) => {
                    println!("token {:?}", message.unwrap().resume_token);
                    assert_eq!(true, true);
                }
                Err(err) => {
                    println!("internal error code = {0}, {1}", err.code(), err.message());
                    assert_eq!(false, true)
                }
            },
            Err(err) => {
                println!("error code = {0}, {1}", err.code(), err.message());
                assert_eq!(false, true)
            }
        };
    }

    #[tokio::test]
    async fn test_commit() {
        let mut client = create_spanner_client().await;
        let session = create_session(&mut client).await;
        let tx = begin_read_write_transaction(&mut client, &session).await;
        let request = CommitRequest {
            session: session.name.to_string(),
            mutations: vec![],
            transaction: Option::from(commit_request::Transaction::TransactionId(tx.id)),
            request_options: Option::from(RequestOptions {
                priority: 10,
                request_tag: "".to_string(),
                transaction_tag: "".to_string(),
            }),
            return_commit_stats: true,
        };

        match client.commit(request, None).await {
            Ok(res) => {
                println!(
                    "mutation count {:?}",
                    res.into_inner().commit_stats.unwrap().mutation_count
                );
                assert_eq!(true, true);
            }
            Err(err) => {
                println!("error code = {0}, {1}", err.code(), err.message());
                assert_eq!(false, true)
            }
        };
    }

    #[tokio::test]
    async fn test_rollback() {
        let mut client = create_spanner_client().await;
        let session = create_session(&mut client).await;
        let tx = begin_read_write_transaction(&mut client, &session).await;
        let request = RollbackRequest {
            session: session.name.to_string(),
            transaction_id: tx.id,
        };

        match client.rollback(request, None).await {
            Ok(res) => {
                println!("rollback success");
                assert_eq!(true, true);
            }
            Err(err) => {
                println!("error code = {0}, {1}", err.code(), err.message());
                assert_eq!(false, true)
            }
        };
    }

    #[tokio::test]
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
