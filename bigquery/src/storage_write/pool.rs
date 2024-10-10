use std::collections::HashMap;
use std::sync::{Arc, Mutex};
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
    router: Router,
    max_insert_count: usize,
    conn_pool: Arc<ConnectionManager>
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