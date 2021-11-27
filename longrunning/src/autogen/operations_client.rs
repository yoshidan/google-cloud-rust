use google_cloud_gax::call_option::{Backoff, BackoffRetrySettings, BackoffRetryer};
use google_cloud_gax::invoke::invoke_reuse;
use google_cloud_gax::util::create_request;
use google_cloud_googleapis::longrunning::operations_client::OperationsClient as InternalOperationsClient;
use google_cloud_googleapis::longrunning::{
    CancelOperationRequest, DeleteOperationRequest, GetOperationRequest, Operation,
    WaitOperationRequest,
};
use google_cloud_googleapis::{Code, Status};
use google_cloud_grpc::conn::{Error, TokenSource};
use std::time::Duration;
use tonic::transport::Channel;
use tonic::Response;

fn default_setting() -> BackoffRetrySettings {
    let mut backoff = Backoff::default();
    backoff.initial = Duration::from_millis(500);
    backoff.max = Duration::from_millis(10000);
    backoff.multiplier = 2.0;
    BackoffRetrySettings {
        retryer: BackoffRetryer {
            backoff,
            codes: vec![Code::Unavailable, Code::Unknown],
        },
    }
}

pub struct OperationsClient {
    inner: InternalOperationsClient<Channel>,
    token_source: TokenSource,
}

impl OperationsClient {
    pub async fn new(channel: Channel, token_source: TokenSource) -> Result<Self, Error> {
        Ok(OperationsClient {
            inner: InternalOperationsClient::new(channel),
            token_source,
        })
    }

    /// merge call setting
    fn get_call_setting(call_setting: Option<BackoffRetrySettings>) -> BackoffRetrySettings {
        match call_setting {
            Some(s) => s,
            None => default_setting(),
        }
    }

    /// GetOperation gets the latest state of a long-running operation.  Clients can use this
    /// method to poll the operation result at intervals as recommended by the API service.
    pub async fn get_operation(
        &mut self,
        req: GetOperationRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<Operation>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let name = &req.name;
        let token = self.token_source.token().await?;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("name={}", name), &token, req.clone());
                client
                    .get_operation(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }

    /// DeleteOperation deletes a long-running operation. This method indicates that the client is
    /// no longer interested in the operation result. It does not cancel the
    /// operation. If the server doesn’t support this method, it returns
    /// google.rpc.Code.UNIMPLEMENTED.
    pub async fn delete_operation(
        &mut self,
        req: DeleteOperationRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<()>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let name = &req.name;
        let token = self.token_source.token().await?;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("name={}", name), &token, req.clone());
                client
                    .delete_operation(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
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
        &mut self,
        req: CancelOperationRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<()>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let name = &req.name;
        let token = self.token_source.token().await?;
        return invoke_reuse(
            |client| async {
                let request = create_request(format!("name={}", name), &token, req.clone());
                client
                    .cancel_operation(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
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
        &mut self,
        req: WaitOperationRequest,
        opt: Option<BackoffRetrySettings>,
    ) -> Result<Response<Operation>, Status> {
        let mut setting = Self::get_call_setting(opt);
        let name = &req.name;
        let token = self.token_source.token().await?;
        return invoke_reuse(
            |client| async {
                let request = create_request("".to_string(), &token, req.clone());
                client
                    .wait_operation(request)
                    .await
                    .map_err(|e| (e.into(), client))
            },
            &mut self.inner,
            &mut setting,
        )
        .await;
    }
}
