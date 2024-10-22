use tokio::sync::{Semaphore, SemaphorePermit};

#[derive(Debug)]
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