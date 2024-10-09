use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::{Semaphore, SemaphorePermit};

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

pub enum Router {
    /// key is writer.id
    Simplex(Arc<Mutex<HashMap<String,Arc<FlowController>>>>),
}

impl Router {
    pub fn new_simplex() -> Self {
        Router::Simplex(Arc::new(Mutex::new(HashMap::new())))
    }

    pub fn attach_writer(&self, writer_id: String, max_insert_count: usize) {
        match self {
            Router::Simplex(map) => {
                let fc = Arc::new(FlowController::new(max_insert_count));
                let mut map = map.lock().unwrap();
                map.insert(writer_id, fc);
            }
        }
    }

    pub fn pick(&self, writer_id: &str) -> Option<Arc<FlowController>> {
        match self {
            Router::Simplex(map) => {
                let map = map.lock().unwrap();
                map.get(writer_id).map(|c| c.clone())
            }
        }
    }
}

pub struct Pool {
    pub location: String,
    pub router: Router,
    pub max_insert_count: usize
}

impl Pool {
    pub fn new(location: String, max_insert_count: usize) -> Self {
        Pool {
            location,
            //TODO support shared router
            router: Router::new_simplex(),
            max_insert_count
        }
    }

    pub fn attach_writer(&self, writer_id: String) {
        self.router.attach_writer(writer_id, self.max_insert_count);
    }

    pub fn pick(&self, writer_id: &str) -> Option<Arc<FlowController>> {
        self.router.pick(writer_id)
    }

}