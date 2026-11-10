use std::error::Error;
use std::future::poll_fn;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use google_cloud_gax::conn::Environment;
use google_cloud_googleapis::spanner::v1::struct_type::Field;
use google_cloud_googleapis::spanner::v1::{
    BatchCreateSessionsRequest, BatchCreateSessionsResponse, ExecuteSqlRequest, PartialResultSet, ResultSet,
    ResultSetMetadata, Session, StructType, Type, TypeCode,
};
use google_cloud_spanner::client::{Client, ClientConfig};
use google_cloud_spanner::statement::Statement;
use http::{HeaderMap, HeaderValue, Response, StatusCode};
use prost::Message;
use prost_types::value::Kind;
use prost_types::Value;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{oneshot, Mutex};
use tokio::task::JoinHandle;

use gcloud_spanner as google_cloud_spanner;

const DATABASE: &str = "projects/local-project/instances/test-instance/databases/local-database";
type TestResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Clone)]
enum StreamBehavior {
    WaitForReset {
        partial_result_set: Box<PartialResultSet>,
        observer: Arc<Mutex<Option<oneshot::Sender<bool>>>>,
        completion: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    },
    AbandonWithoutLast {
        observer: Arc<Mutex<Option<oneshot::Sender<bool>>>>,
    },
    InvalidArgument {
        finished: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    },
    Resume(ResumeScenario),
}

#[derive(Clone, Copy)]
enum ResumeScenario {
    S1,
    S2,
    S3,
}

struct FakeServer {
    address: String,
    accept_task: JoinHandle<()>,
    execute_streaming_calls: Arc<AtomicUsize>,
}

impl Drop for FakeServer {
    fn drop(&mut self) {
        self.accept_task.abort();
    }
}

async fn start_fake_server(behavior: StreamBehavior) -> TestResult<FakeServer> {
    let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
    let address = listener.local_addr()?.to_string();
    let execute_streaming_calls = Arc::new(AtomicUsize::new(0));
    let calls = Arc::clone(&execute_streaming_calls);
    let accept_task = tokio::spawn(async move {
        loop {
            let Ok((socket, _)) = listener.accept().await else {
                return;
            };
            let behavior = behavior.clone();
            let calls = Arc::clone(&calls);
            tokio::spawn(async move {
                if let Err(error) = serve_connection(socket, behavior, calls).await {
                    eprintln!("fake h2 connection failed: {error}");
                }
            });
        }
    });
    Ok(FakeServer {
        address,
        accept_task,
        execute_streaming_calls,
    })
}

async fn serve_connection(
    socket: TcpStream,
    behavior: StreamBehavior,
    execute_streaming_calls: Arc<AtomicUsize>,
) -> TestResult<()> {
    let mut connection = h2::server::handshake(socket).await?;
    while let Some(request) = connection.accept().await {
        let (request, respond) = request?;
        let behavior = behavior.clone();
        let calls = Arc::clone(&execute_streaming_calls);
        let completion = match (&behavior, request.uri().path()) {
            (StreamBehavior::WaitForReset { completion, .. }, "/google.spanner.v1.Spanner/ExecuteStreamingSql") => {
                Some(Arc::clone(completion))
            }
            _ => None,
        };
        tokio::spawn(async move {
            let result = handle_fake_request(request, respond, behavior, calls).await;
            if let Some(completion) = completion {
                if let Some(sender) = completion.lock().await.take() {
                    let _ = sender.send(());
                }
            }
            if let Err(error) = result {
                eprintln!("fake h2 request failed: {error}");
            }
        });
    }
    Ok(())
}

async fn handle_fake_request(
    request: http::Request<h2::RecvStream>,
    mut respond: h2::server::SendResponse<Bytes>,
    behavior: StreamBehavior,
    execute_streaming_calls: Arc<AtomicUsize>,
) -> TestResult<()> {
    let path = request.uri().path().to_string();
    let body = read_request_body(request.into_body()).await?;

    match path.as_str() {
        "/google.spanner.v1.Spanner/BatchCreateSessions" => {
            let request = decode_grpc::<BatchCreateSessionsRequest>(&body)?;
            let session_count = usize::try_from(request.session_count.max(0))?;
            let sessions = (0..session_count)
                .map(|index| Session {
                    name: format!("{}/sessions/test-{index}", request.database),
                    ..Default::default()
                })
                .collect();
            let response = BatchCreateSessionsResponse { session: sessions };
            send_unary(&mut respond, Some(grpc_frame(response.encode_to_vec())?), "0").await?;
        }
        "/google.spanner.v1.Spanner/ExecuteStreamingSql" => {
            let request = decode_grpc::<ExecuteSqlRequest>(&body)?;
            let call = execute_streaming_calls.fetch_add(1, Ordering::SeqCst);
            match behavior {
                StreamBehavior::WaitForReset {
                    partial_result_set,
                    observer,
                    ..
                } => {
                    let response = grpc_frame(partial_result_set.encode_to_vec())?;
                    let mut stream = respond.send_response(grpc_response(), false)?;
                    stream.send_data(response, false)?;
                    let reset = tokio::select! {
                        reset = poll_fn(|cx| stream.poll_reset(cx)) => Some(reset?),
                        _ = tokio::time::sleep(Duration::from_millis(500)) => None,
                    };
                    if reset.is_some() {
                        report_reset(&observer, true).await;
                        return Ok(());
                    }
                    send_trailers(&mut stream, "0", None)?;
                    report_reset(&observer, false).await;
                }
                StreamBehavior::AbandonWithoutLast { observer } => {
                    let response = grpc_frame(int64_partial_result_set(false, "t1").encode_to_vec())?;
                    let mut stream = respond.send_response(grpc_response(), false)?;
                    stream.send_data(response, false)?;
                    poll_fn(|cx| stream.poll_reset(cx)).await?;
                    report_reset(&observer, true).await;
                }
                StreamBehavior::InvalidArgument { finished } => {
                    let response = grpc_frame(int64_partial_result_set(true, "").encode_to_vec())?;
                    let mut stream = respond.send_response(grpc_response(), false)?;
                    stream.send_data(response, false)?;
                    send_trailers(&mut stream, "3", Some("test%20invalid%20argument"))?;
                    if let Some(sender) = finished.lock().await.take() {
                        let _ = sender.send(());
                    }
                }
                StreamBehavior::Resume(scenario) => {
                    return send_resume_response(&mut respond, scenario, call, &request.resume_token).await;
                }
            }
        }
        "/google.spanner.v1.Spanner/ExecuteSql" => {
            let response = grpc_frame(ResultSet::default().encode_to_vec())?;
            send_unary(&mut respond, Some(response), "0").await?;
        }
        "/google.spanner.v1.Spanner/DeleteSession" => {
            send_unary(&mut respond, None, "0").await?;
        }
        _ => send_unary(&mut respond, None, "12").await?,
    }
    Ok(())
}

async fn send_resume_response(
    respond: &mut h2::server::SendResponse<Bytes>,
    scenario: ResumeScenario,
    call: usize,
    resume_token: &[u8],
) -> TestResult<()> {
    let expected_token: Option<&[u8]> = match (scenario, call) {
        (_, 0) => Some(b""),
        (ResumeScenario::S1 | ResumeScenario::S2, 1) => Some(b"t1"),
        (ResumeScenario::S3, 1) => Some(b"t2"),
        _ => None,
    };
    if expected_token != Some(resume_token) {
        send_unary(respond, None, "9").await?;
        return Ok(());
    }

    let (result_sets, grpc_status): (Vec<PartialResultSet>, &'static str) = match (scenario, call) {
        (ResumeScenario::S1, 0) => (
            vec![
                string_prs(&["row-1", "row-2"], "t1", false, false, true),
                string_prs(&["row-3-lo"], "", true, false, false),
            ],
            "14",
        ),
        (ResumeScenario::S1, 1) => (vec![string_prs(&["row-3-long-value", "row-4"], "", false, true, true)], "0"),
        (ResumeScenario::S2, 0) => (
            vec![
                string_prs(&["row-1", "row-2"], "t1", false, false, true),
                string_prs(&["row-3"], "", false, true, false),
            ],
            "14",
        ),
        (ResumeScenario::S2, 1) => (vec![string_prs(&["row-3"], "", false, true, true)], "0"),
        (ResumeScenario::S3, 0) => (
            vec![
                string_prs(&["row-1", "row-2"], "t1", false, false, true),
                string_prs(&["row-3"], "t2", false, true, false),
            ],
            "14",
        ),
        (ResumeScenario::S3, 1) => (vec![string_prs(&[], "", false, true, true)], "0"),
        _ => {
            send_unary(respond, None, "9").await?;
            return Ok(());
        }
    };

    let mut stream = respond.send_response(grpc_response(), false)?;
    for result_set in result_sets {
        stream.send_data(grpc_frame(result_set.encode_to_vec())?, false)?;
    }
    send_trailers(&mut stream, grpc_status, None)?;
    Ok(())
}

fn string_prs(
    values: &[&str],
    resume_token: &str,
    chunked_value: bool,
    last: bool,
    with_metadata: bool,
) -> PartialResultSet {
    let metadata = with_metadata.then(|| ResultSetMetadata {
        row_type: Some(StructType {
            fields: vec![Field {
                name: "value".to_string(),
                r#type: Some(Type {
                    code: TypeCode::String as i32,
                    ..Default::default()
                }),
            }],
        }),
        ..Default::default()
    });
    PartialResultSet {
        metadata,
        values: values
            .iter()
            .map(|value| Value {
                kind: Some(Kind::StringValue((*value).to_string())),
            })
            .collect(),
        resume_token: resume_token.as_bytes().to_vec(),
        chunked_value,
        last,
        ..Default::default()
    }
}

fn int64_partial_result_set(last: bool, resume_token: &str) -> PartialResultSet {
    PartialResultSet {
        metadata: Some(ResultSetMetadata {
            row_type: Some(StructType {
                fields: vec![Field {
                    name: "value".to_string(),
                    r#type: Some(Type {
                        code: TypeCode::Int64 as i32,
                        ..Default::default()
                    }),
                }],
            }),
            ..Default::default()
        }),
        values: vec![Value {
            kind: Some(Kind::StringValue("42".to_string())),
        }],
        resume_token: resume_token.as_bytes().to_vec(),
        last,
        ..Default::default()
    }
}

async fn report_reset(observer: &Arc<Mutex<Option<oneshot::Sender<bool>>>>, saw_reset: bool) {
    if let Some(sender) = observer.lock().await.take() {
        let _ = sender.send(saw_reset);
    }
}

async fn read_request_body(mut body: h2::RecvStream) -> TestResult<Vec<u8>> {
    let mut bytes = Vec::new();
    while let Some(chunk) = body.data().await {
        let chunk = chunk?;
        body.flow_control().release_capacity(chunk.len())?;
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

fn decode_grpc<T: Message + Default>(bytes: &[u8]) -> TestResult<T> {
    if bytes.len() < 5 || bytes[0] != 0 {
        return Err("invalid uncompressed gRPC frame".into());
    }
    let length = u32::from_be_bytes(bytes[1..5].try_into()?) as usize;
    if bytes.len() != length + 5 {
        return Err("gRPC frame length mismatch".into());
    }
    Ok(T::decode(&bytes[5..])?)
}

fn grpc_frame(message: Vec<u8>) -> TestResult<Bytes> {
    let length = u32::try_from(message.len())?;
    let mut frame = Vec::with_capacity(message.len() + 5);
    frame.push(0);
    frame.extend_from_slice(&length.to_be_bytes());
    frame.extend_from_slice(&message);
    Ok(Bytes::from(frame))
}

async fn send_unary(
    respond: &mut h2::server::SendResponse<Bytes>,
    message: Option<Bytes>,
    grpc_status: &'static str,
) -> TestResult<()> {
    let mut stream = respond.send_response(grpc_response(), false)?;
    if let Some(message) = message {
        stream.send_data(message, false)?;
    }
    send_trailers(&mut stream, grpc_status, None)?;
    Ok(())
}

fn grpc_response() -> Response<()> {
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/grpc")
        .header("grpc-encoding", "identity")
        .body(())
        .expect("static gRPC response headers are valid")
}

fn send_trailers(
    stream: &mut h2::SendStream<Bytes>,
    grpc_status: &'static str,
    grpc_message: Option<&'static str>,
) -> TestResult<()> {
    let mut trailers = HeaderMap::new();
    trailers.insert("grpc-status", HeaderValue::from_static(grpc_status));
    if let Some(message) = grpc_message {
        trailers.insert("grpc-message", HeaderValue::from_static(message));
    }
    stream.send_trailers(trailers)?;
    Ok(())
}

async fn create_client(address: &str) -> Client {
    let mut config = ClientConfig {
        environment: Environment::Emulator(address.to_string()),
        ..Default::default()
    };
    config.channel_config.num_channels = 1;
    config.session_config.min_opened = 1;
    config.session_config.max_opened = 1;
    Client::new(DATABASE, config).await.expect("fake server client")
}

async fn run_delayed_trailer_case(read_to_none: bool, resume_token: &str) -> TestResult<()> {
    let (reset_tx, reset_rx) = oneshot::channel();
    let (completion_tx, completion_rx) = oneshot::channel();
    let observer = Arc::new(Mutex::new(Some(reset_tx)));
    let completion = Arc::new(Mutex::new(Some(completion_tx)));
    let partial_result_set = int64_partial_result_set(true, resume_token);
    let server = start_fake_server(StreamBehavior::WaitForReset {
        partial_result_set: Box::new(partial_result_set),
        observer,
        completion,
    })
    .await?;
    let client = create_client(&server.address).await;
    let mut transaction = client.single().await?;
    let mut iter = transaction.query(Statement::new("SELECT 42 AS value")).await?;

    let row = iter.next().await?.expect("expected the result row");
    let mut values = vec![row.column::<i64>(0)?];
    if read_to_none {
        while let Some(row) = iter.next().await? {
            values.push(row.column::<i64>(0)?);
        }
    }
    drop(iter);
    drop(transaction);

    assert_eq!(values, vec![42]);
    let saw_reset = tokio::time::timeout(Duration::from_secs(2), reset_rx).await??;
    tokio::time::timeout(Duration::from_secs(2), completion_rx).await??;
    assert!(!saw_reset, "server observed a client reset before trailers");
    assert_eq!(server.execute_streaming_calls.load(Ordering::SeqCst), 1);
    client.close().await;
    Ok(())
}

#[tokio::test]
async fn query_reads_trailers_after_last_partial_result_set() -> TestResult<()> {
    run_delayed_trailer_case(true, "").await
}

#[tokio::test]
async fn query_drains_trailers_after_one_next_without_resume_token() -> TestResult<()> {
    run_delayed_trailer_case(false, "").await
}

#[tokio::test]
async fn query_drains_trailers_after_one_next_with_resume_token() -> TestResult<()> {
    run_delayed_trailer_case(false, "t1").await
}

#[tokio::test]
async fn query_cancels_abandoned_stream_without_last() -> TestResult<()> {
    let (reset_tx, reset_rx) = oneshot::channel();
    let observer = Arc::new(Mutex::new(Some(reset_tx)));
    let server = start_fake_server(StreamBehavior::AbandonWithoutLast { observer }).await?;
    let client = create_client(&server.address).await;
    let mut transaction = client.single().await?;
    let mut iter = transaction.query(Statement::new("SELECT 42 AS value")).await?;

    let row = iter.next().await?.expect("expected the result row");
    assert_eq!(row.column::<i64>(0)?, 42);
    drop(iter);
    drop(transaction);

    let saw_reset = tokio::time::timeout(Duration::from_secs(2), reset_rx).await??;
    assert!(saw_reset, "server did not observe cancellation of an abandoned result");
    client.close().await;
    Ok(())
}

#[tokio::test]
async fn query_ignores_error_status_after_last_partial_result_set() -> TestResult<()> {
    let (finished_tx, finished_rx) = oneshot::channel();
    let finished = Arc::new(Mutex::new(Some(finished_tx)));
    let server = start_fake_server(StreamBehavior::InvalidArgument { finished }).await?;
    let client = create_client(&server.address).await;
    let mut transaction = client.single().await?;
    let mut iter = transaction.query(Statement::new("SELECT 42 AS value")).await?;

    let row = iter.next().await?.expect("expected the result row");
    assert_eq!(row.column::<i64>(0)?, 42);
    assert!(iter.next().await?.is_none());
    drop(iter);
    drop(transaction);

    tokio::time::timeout(Duration::from_secs(2), finished_rx).await??;
    assert_eq!(server.execute_streaming_calls.load(Ordering::SeqCst), 1);
    client.close().await;
    Ok(())
}

async fn run_resume_scenario(scenario: ResumeScenario) -> TestResult<(Vec<String>, usize)> {
    let server = start_fake_server(StreamBehavior::Resume(scenario)).await?;
    let client = create_client(&server.address).await;
    let mut transaction = client.single().await?;
    let mut iter = transaction.query(Statement::new("SELECT value FROM Test")).await?;

    let mut values = Vec::new();
    while let Some(row) = iter.next().await? {
        values.push(row.column::<String>(0)?);
    }
    drop(iter);
    drop(transaction);
    client.close().await;

    let calls = server.execute_streaming_calls.load(Ordering::SeqCst);
    Ok((values, calls))
}

#[tokio::test]
async fn query_resumes_mid_stream_without_duplicate_or_corrupt_rows() -> TestResult<()> {
    let (values, calls) = run_resume_scenario(ResumeScenario::S1).await?;

    assert_eq!(
        values,
        vec![
            "row-1".to_string(),
            "row-2".to_string(),
            "row-3-long-value".to_string(),
            "row-4".to_string()
        ]
    );
    assert_eq!(calls, 2);
    Ok(())
}

#[tokio::test]
async fn query_does_not_resume_when_stream_fails_after_last_before_trailer() -> TestResult<()> {
    let (values, calls) = run_resume_scenario(ResumeScenario::S2).await?;

    assert_eq!(values, vec!["row-1".to_string(), "row-2".to_string(), "row-3".to_string()]);
    assert_eq!(calls, 1);
    Ok(())
}

#[tokio::test]
async fn query_finishes_when_stream_fails_after_tokened_last() -> TestResult<()> {
    let (values, calls) = run_resume_scenario(ResumeScenario::S3).await?;

    assert_eq!(values, vec!["row-1".to_string(), "row-2".to_string(), "row-3".to_string()]);
    assert_eq!(calls, 1);
    Ok(())
}
