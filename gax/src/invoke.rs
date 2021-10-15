use crate::call_option::{CallSettings, Retryer};
use std::future::Future;
use tonic::Status;

pub trait AsTonicStatus {
    fn as_tonic_status(&self) -> Option<&tonic::Status>;
}

impl AsTonicStatus for tonic::Status {
    fn as_tonic_status(&self) -> Option<&Status> {
        Some(self)
    }
}

pub async fn invoke<T, E, Fut>(
    mut f: impl FnMut() -> Fut,
    settings: &mut CallSettings,
) -> Result<T, E>
where
    E: AsTonicStatus,
    Fut: Future<Output = Result<T, E>>,
{
    let retryer = &mut settings.retryer;
    loop {
        let err = match f().await {
            Ok(s) => return Ok(s),
            Err(e) => e,
        };

        let status = match err.as_tonic_status() {
            Some(s) => s,
            None => return Err(err),
        };

        match retryer.retry(status) {
            Some(duration) => tokio::time::sleep(duration).await,
            None => return Err(err),
        };
    }
}

pub async fn invoke_reuse<T, E, V, Fut>(
    mut f: impl FnMut(V) -> Fut,
    mut v: V,
    settings: &mut CallSettings,
) -> Result<T, E>
where
    E: AsTonicStatus,
    Fut: Future<Output = Result<(T, V), (E, V)>>,
{
    let retryer = &mut settings.retryer;
    loop {
        let result = f(v).await;
        let err = match result {
            Ok(s) => return Ok(s.0),
            Err(e) => {
                v = e.1;
                e.0
            }
        };
        let status = match err.as_tonic_status() {
            Some(s) => s,
            None => return Err(err),
        };
        match retryer.retry(status) {
            Some(duration) => tokio::time::sleep(duration).await,
            None => return Err(err),
        };
    }
}
