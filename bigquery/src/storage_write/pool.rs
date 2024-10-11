use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use google_cloud_gax::grpc::Status;
use google_cloud_googleapis::cloud::bigquery::storage::v1::big_query_write_client::BigQueryWriteClient;
use google_cloud_googleapis::cloud::bigquery::storage::v1::write_stream::{Type, WriteMode};
use google_cloud_googleapis::cloud::bigquery::storage::v1::{GetWriteStreamRequest, WriteStream};
use crate::grpc::apiv1::bigquery_client::{create_write_stream_request, StreamingWriteClient};
use crate::grpc::apiv1::conn_pool::ConnectionManager;
use crate::storage_write::connection::Connection;
use crate::storage_write::flow::FlowController;

enum Router {
    /// key is writer.id
    Simplex(Arc<Mutex<HashMap<String,Arc<Connection>>>>)
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
                if !map.contains_key(&writer_id) {
                    map.insert(writer_id, conn);
                }
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

pub struct Connections {
    router: Router,
    max_insert_count: usize,
    conn_pool: Arc<ConnectionManager>
}

impl Connections {
    pub fn new(max_insert_count: usize, conn_pool: Arc<ConnectionManager>) -> Self {
        Self {
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

pub(crate) struct Pool {
    /// key = location
    cons: HashMap<String, Connections>,
    max_insert_count: usize,
    p_cons: Arc<ConnectionManager>
}

impl Pool {
    pub fn new(max_insert_count: usize, p_cons: Arc<ConnectionManager>) -> Self {
        Self {
            cons: HashMap::new(),
            max_insert_count,
            p_cons
        }
    }

    pub async fn create_stream(&mut self, table: &str, mode: Type) -> Result<WriteStream, Status> {
        let mut client = self.client();
        let req = create_write_stream_request(table, mode);
        let stream = client.create_write_stream(req, None).await?.into_inner();
        self.regional(&stream.location).attach_writer(stream.name.clone());
        Ok(stream)
    }

    pub async fn get_stream(&mut self, name: &str) -> Result<WriteStream, Status> {
        let mut client = self.client();
        let req = GetWriteStreamRequest {
            name: name.to_string(),
            view: 0,
        };
        let stream = client.get_write_stream(req, None).await?.into_inner();
        self.regional(&stream.location).attach_writer(stream.name.clone());
        Ok(stream)
    }

    pub fn client(&self) -> StreamingWriteClient {
        StreamingWriteClient::new(BigQueryWriteClient::new(self.p_cons.conn()))
    }

    pub fn regional(&mut self, location: &str) -> &Connections {
        let cons = self.cons.get(location);
        match cons {
            Some(pool) => pool,
            None => {
                let cons = Connections::new(self.max_insert_count, self.p_cons.clone());
                self.cons.insert(location.to_string(), cons);
                self.cons.get(location).unwrap()
            }
        }
    }
}