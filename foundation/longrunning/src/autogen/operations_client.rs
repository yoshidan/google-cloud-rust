use google_cloud_gax::cancel::CancellationToken;
use google_cloud_gax::conn::{Channel, Error};
use google_cloud_gax::create_request;
use google_cloud_gax::grpc::{Code, Status};
use google_cloud_gax::retry::{invoke, RetrySetting};
use google_cloud_googleapis::longrunning::operations_client::OperationsClient as InternalOperationsClient;
use google_cloud_googleapis::longrunning::{
    CancelOperationRequest, DeleteOperationRequest, GetOperationRequest, Operation, WaitOperationRequest,
};
use std::time::Duration;
use tonic::Response;

pub fn default_retry_setting() -> RetrySetting {
    RetrySetting {
        from_millis: 50,
        max_delay: Some(Duration::from_secs(10)),
        factor: 1u64,
        take: 20,
        codes: vec![Code::Unavailable, Code::Unknown],
    }
}

#[derive(Clone)]
pub struct OperationsClient {
    inner: InternalOperationsClient<Channel>,
}

impl OperationsClient {
    pub async fn new(channel: Channel) -> Result<Self, Error> {
        Ok(OperationsClient {
            inner: InternalOperationsClient::new(channel),
        })
    }

    /// GetOperation gets the latest state of a long-running operation.  Clients can use this
    /// method to poll the operation result at intervals as recommended by the API service.
    pub async fn get_operation(
        &self,
        req: GetOperationRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Operation>, Status> {
        let setting = retry.unwrap_or_else(default_retry_setting);
        let name = &req.name;
        let action = || async {
            let request = create_request(format!("name={}", name), req.clone());
            self.inner.clone().get_operation(request).await.map_err(|e| e)
        };
        invoke(cancel, Some(setting), action).await
    }

    /// DeleteOperation deletes a long-running operation. This method indicates that the client is
    /// no longer interested in the operation result. It does not cancel the
    /// operation. If the server doesn’t support this method, it returns
    /// google.rpc.Code.UNIMPLEMENTED.
    pub async fn delete_operation(
        &self,
        req: DeleteOperationRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<()>, Status> {
        let setting = retry.unwrap_or_else(default_retry_setting);
        let name = &req.name;
        let action = || async {
            let request = create_request(format!("name={}", name), req.clone());
            self.inner.clone().delete_operation(request).await.map_err(|e| e)
        };
        invoke(cancel, Some(setting), action).await
    }

    /// CancelOperation starts asynchronous cancellation on a long-running operation.  The server
    /// makes a best effort to cancel the operation, but success is not
    /// guaranteed.  If the server doesn’t support this method, it returns
    /// google.rpc.Code.UNIMPLEMENTED.  Clients can use
    /// Operations.GetOperation or
    /// other methods to check whether the cancellation succeeded or whether the
    /// operation completed despite cancellation. On successful cancellation,
    /// the operation is not deleted; instead, it becomes an operation with
    /// an Operation.error value with a google.rpc.Status.code of 1,
    /// corresponding to Code.CANCELLED.
    pub async fn cancel_operation(
        &self,
        req: CancelOperationRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<()>, Status> {
        let setting = retry.unwrap_or_else(default_retry_setting);
        let name = &req.name;
        let action = || async {
            let request = create_request(format!("name={}", name), req.clone());
            self.inner.clone().cancel_operation(request).await.map_err(|e| e)
        };
        invoke(cancel, Some(setting), action).await
    }

    /// WaitOperation waits until the specified long-running operation is done or reaches at most
    /// a specified timeout, returning the latest state.  If the operation is
    /// already done, the latest state is immediately returned.  If the timeout
    /// specified is greater than the default HTTP/RPC timeout, the HTTP/RPC
    /// timeout is used.  If the server does not support this method, it returns
    /// google.rpc.Code.UNIMPLEMENTED.
    /// Note that this method is on a best-effort basis.  It may return the latest
    /// state before the specified timeout (including immediately), meaning even an
    /// immediate response is no guarantee that the operation is done.
    pub async fn wait_operation(
        &self,
        req: WaitOperationRequest,
        cancel: Option<CancellationToken>,
        retry: Option<RetrySetting>,
    ) -> Result<Response<Operation>, Status> {
        let setting = retry.unwrap_or_else(default_retry_setting);
        let action = || async {
            let request = create_request("".to_string(), req.clone());
            self.inner.clone().wait_operation(request).await.map_err(|e| e)
        };
        invoke(cancel, Some(setting), action).await
    }
}
