use tokio::select;
use tokio_retry::Action;
use tokio_util::sync::CancellationToken;
use tonic::Request;
use tonic::IntoRequest;
use crate::retry::RetrySetting;
use crate::status::Status;

pub mod status;
pub mod retry;
pub mod conn;

pub fn create_request<T>(param_string: String, into_request: impl IntoRequest<T>) -> Request<T> {
    let mut request = into_request.into_request();
    let target = request.metadata_mut();
    if !param_string.is_empty() {
        target.append("x-goog-request-params", param_string.parse().unwrap());
    }
    request
}

