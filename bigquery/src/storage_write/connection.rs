use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::{Semaphore, SemaphorePermit};
use google_cloud_gax::grpc::{IntoStreamingRequest, Status, Streaming};
use google_cloud_googleapis::cloud::bigquery::storage::v1::{AppendRowsRequest, AppendRowsResponse};
use google_cloud_googleapis::cloud::bigquery::storage::v1::big_query_write_client::BigQueryWriteClient;
use crate::grpc::apiv1::bigquery_client::StreamingWriteClient;
use crate::grpc::apiv1::conn_pool::ConnectionManager;
use crate::storage_write::flow::FlowController;

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