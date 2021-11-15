use crate::call_option::{RetrySettings, Retryer};
use google_cloud_googleapis::Status;
use std::future::Future;

pub trait AsGrpcStatus {
    fn as_status(&self) -> Option<&Status>;
}

impl AsGrpcStatus for Status {
    fn as_status(&self) -> Option<&Status> {
        Some(self)
    }
}

/// Repeats retries when the specified error is detected.
pub async fn invoke<Setting, T, E, Fut>(
    mut f: impl FnMut() -> Fut,
    settings: &mut RetrySettings<Setting>,
) -> Result<T, E>
where
    E: AsGrpcStatus,
    Fut: Future<Output = Result<T, E>>,
    Setting: Retryer + Clone,
{
    let retryer = &mut settings.retryer;
    loop {
        let err = match f().await {
            Ok(s) => return Ok(s),
            Err(e) => e,
        };

        let status = match err.as_status() {
            Some(s) => s,
            None => return Err(err),
        };

        match retryer.retry(status) {
            Some(duration) => tokio::time::sleep(duration).await,
            None => return Err(err),
        };
    }
}

/// Repeats retries when the specified error is detected.
/// The argument specified by 'v' can be reused for each retry.
pub async fn invoke_reuse<Setting, T, E, V, Fut>(
    mut f: impl FnMut(V) -> Fut,
    mut v: V,
    settings: &mut RetrySettings<Setting>,
) -> Result<T, E>
where
    E: AsGrpcStatus,
    Fut: Future<Output = Result<T, (E, V)>>,
    Setting: Retryer + Clone,
{
    let retryer = &mut settings.retryer;
    loop {
        let result = f(v).await;
        let err = match result {
            Ok(s) => return Ok(s),
            Err(e) => {
                v = e.1;
                e.0
            }
        };
        let status = match err.as_status() {
            Some(s) => s,
            None => return Err(err),
        };
        match retryer.retry(status) {
            Some(duration) => tokio::time::sleep(duration).await,
            None => return Err(err),
        };
    }
}
