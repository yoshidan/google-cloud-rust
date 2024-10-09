use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::{Semaphore, SemaphorePermit};
use google_cloud_gax::grpc::{IntoStreamingRequest, Status, Streaming};
use google_cloud_googleapis::cloud::bigquery::storage::v1::{AppendRowsRequest, AppendRowsResponse};
use google_cloud_googleapis::cloud::bigquery::storage::v1::big_query_write_client::BigQueryWriteClient;
use crate::grpc::apiv1::bigquery_client::StreamingWriteClient;
use crate::grpc::apiv1::conn_pool::ConnectionManager;

pub struct FlowController {
    sem_insert_count: Semaphore
    //TODO support sem_insert_bytes
}

impl FlowController {

    pub fn new(max_insert_count: usize) -> Self {
        FlowController {
            sem_insert_count: Semaphore::new(max_insert_count)
        }
    }
    pub async fn acquire(&self) -> SemaphorePermit {
        self.sem_insert_count.acquire().await.unwrap()
    }

}

pub struct Connection {
    fc: FlowController,
    grpc_conn_pool: Arc<ConnectionManager>
}

impl Connection {
    pub fn new(fc: FlowController, grpc_conn_pool: Arc<ConnectionManager>) -> Self {
        Connection {
            fc,
            grpc_conn_pool
        }
    }

    pub async fn locking_append(&self, req: impl IntoStreamingRequest<Message = AppendRowsRequest>) -> Result<Streaming<AppendRowsResponse>, Status> {
        let permit = self.fc.acquire().await;
        let mut client = StreamingWriteClient::new(BigQueryWriteClient::new(self.grpc_conn_pool.conn()));
        let result = client.append_rows(req).await?.into_inner();
        drop(permit);
        Ok(result)
    }
}

pub enum Router {
    /// key is writer.id
    Simplex(Arc<Mutex<HashMap<String,Arc<Connection>>>>),
    //TODO support shared router
}

impl Router {
    pub fn new_simplex() -> Self {
        Router::Simplex(Arc::new(Mutex::new(HashMap::new())))
    }

    pub fn attach_writer(&self, writer_id: String, max_insert_count: usize, grpc_conn_pool: Arc<ConnectionManager>) {
        match self {
            Router::Simplex(map) => {
                let fc = FlowController::new(max_insert_count);
                let conn = Arc::new(Connection::new(fc, grpc_conn_pool));
                let mut map = map.lock().unwrap();
                map.insert(writer_id, conn);
            }
        }
    }

    pub fn remove_writer(&self, writer_id: &str) {
        match self {
            Router::Simplex(map) => {
                let mut map = map.lock().unwrap();
                map.remove(writer_id);
            }
        }
    }

    pub fn pick(&self, writer_id: &str) -> Option<Arc<Connection>> {
        match self {
            Router::Simplex(map) => {
                let map = map.lock().unwrap();
                map.get(writer_id).map(|c| c.clone())
            }
        }
    }
}

pub struct Pool {
    pub router: Router,
    pub max_insert_count: usize,
    pub conn_pool: Arc<ConnectionManager>
}

impl Pool {
    pub fn new(max_insert_count: usize, conn_pool: Arc<ConnectionManager>) -> Self {
        Pool {
            router: Router::new_simplex(),
            max_insert_count,
            conn_pool
        }
    }

    pub fn attach_writer(&self, writer_id: String) {
        self.router.attach_writer(writer_id, self.max_insert_count, self.conn_pool.clone());
    }

    pub fn pick(&self, writer_id: &str) -> Option<Arc<Connection>> {
        self.router.pick(writer_id)
    }

}