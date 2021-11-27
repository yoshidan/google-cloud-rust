use crate::autogen::operations_client::OperationsClient;
use google_cloud_gax::call_option::BackoffRetrySettings;
use google_cloud_gax::invoke::invoke_reuse;
use google_cloud_googleapis::longrunning::{
    operation, CancelOperationRequest, DeleteOperationRequest, GetOperationRequest,
    Operation as InternalOperation,
};
use google_cloud_googleapis::{Code, Status};

pub struct Operation {
    inner: InternalOperation,
    client: OperationsClient,
}

impl Operation {
    pub fn new(client: OperationsClient, inner: InternalOperation) -> Self {
        Self { client, inner }
    }

    /// Name returns the name of the long-running operation.
    /// The name is assigned by the server and is unique within the service
    /// from which the operation is created.
    pub fn name(&self) -> &str {
        &self.inner.name
    }

    /// Done reports whether the long-running operation has completed.
    pub fn done(&self) -> bool {
        self.inner.done
    }

    /// Poll fetches the latest state of a long-running operation.
    ///
    /// If Poll fails, the error is returned and op is unmodified.
    /// If Poll succeeds and the operation has completed with failure,
    /// the error is returned and op.Done will return true.
    /// If Poll succeeds and the operation has completed successfully,
    /// op.Done will return true; if resp != nil, the response of the operation
    /// is stored in resp.
    pub async fn poll<T>(&mut self) -> Result<Option<T>, Status>
    where
        T: prost::Message + Default,
    {
        if !self.done() {
            let operation = self
                .client
                .get_operation(
                    GetOperationRequest {
                        name: self.name().to_string(),
                    },
                    None,
                )
                .await?;
            self.inner = operation.into_inner()
        }
        if !self.done() {
            return Ok(None);
        }
        let operation_result = self.inner.result.clone().unwrap();
        match operation_result {
            operation::Result::Response(message) => {
                //TODO avoid unwrap
                let decoded = T::decode(message.value.as_slice()).unwrap();
                Ok(Some(decoded))
            }
            operation::Result::Error(status) => {
                let tonic_code = tonic::Code::from(status.code);
                Err(tonic::Status::new(tonic_code, status.message.to_string()).into())
            }
        }
    }

    /// wait implements Wait, taking exponentialBackoff and sleeper arguments for testing.
    pub async fn wait<T>(&mut self, mut settings: BackoffRetrySettings) -> Result<Option<T>, Status>
    where
        T: prost::Message + Default,
    {
        settings.retryer.codes.push(Code::DeadlineExceeded);
        return invoke_reuse(
            |me| async {
                let poll_result: Option<T> = match me.poll().await {
                    Ok(s) => s,
                    Err(e) => return Err((e, me)),
                };
                if me.done() {
                    Ok(poll_result)
                } else {
                    Err((
                        tonic::Status::new(tonic::Code::DeadlineExceeded, "wait timeout").into(),
                        me,
                    ))
                }
            },
            self,
            &mut settings,
        )
        .await;
    }

    /// Cancel starts asynchronous cancellation on a long-running operation. The server
    /// makes a best effort to cancel the operation, but success is not
    /// guaranteed. If the server doesn't support this method, it returns
    /// status.Code(err) == codes.Unimplemented. Clients can use
    /// Poll or other methods to check whether the cancellation succeeded or whether the
    /// operation completed despite cancellation. On successful cancellation,
    /// the operation is not deleted; instead, op.Poll returns an error
    /// with code Canceled.
    pub async fn cancel(&mut self) -> Result<(), Status> {
        self.client
            .cancel_operation(
                CancelOperationRequest {
                    name: self.name().to_string(),
                },
                None,
            )
            .await
            .map(|_x| ())
    }

    /// Delete deletes a long-running operation. This method indicates that the client is
    /// no longer interested in the operation result. It does not cancel the
    /// operation. If the server doesn't support this method, status.Code(err) == codes.Unimplemented.
    pub async fn delete(&mut self) -> Result<(), Status> {
        self.client
            .delete_operation(
                DeleteOperationRequest {
                    name: self.name().to_string(),
                },
                None,
            )
            .await
            .map(|_x| ())
    }
}
