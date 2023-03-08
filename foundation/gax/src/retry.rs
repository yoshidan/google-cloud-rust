use std::future::Future;
use std::iter::Take;
use std::time::Duration;

pub use tokio_retry::strategy::ExponentialBackoff;
pub use tokio_retry::Condition;
use tokio_retry::{Action, RetryIf};

use crate::grpc::{Code, Status};

pub trait TryAs<T> {
    fn try_as(&self) -> Option<&T>;
}

impl TryAs<Status> for Status {
    fn try_as(&self) -> Option<&Status> {
        Some(self)
    }
}

pub trait Retry<E: TryAs<Status>, T: Condition<E>> {
    fn strategy(&self) -> Take<ExponentialBackoff>;
    fn condition(&self) -> T;
}

pub struct CodeCondition {
    codes: Vec<Code>,
}

impl CodeCondition {
    pub fn new(codes: Vec<Code>) -> Self {
        Self { codes }
    }
}

impl<E> Condition<E> for CodeCondition
where
    E: TryAs<Status>,
{
    fn should_retry(&mut self, error: &E) -> bool {
        if let Some(status) = error.try_as() {
            for code in &self.codes {
                if *code == status.code() {
                    return true;
                }
            }
        }
        false
    }
}

#[derive(Clone, Debug)]
pub struct RetrySetting {
    pub from_millis: u64,
    pub max_delay: Option<Duration>,
    pub factor: u64,
    pub take: usize,
    pub codes: Vec<Code>,
}

impl Retry<Status, CodeCondition> for RetrySetting {
    fn strategy(&self) -> Take<ExponentialBackoff> {
        let mut st = ExponentialBackoff::from_millis(self.from_millis);
        if let Some(max_delay) = self.max_delay {
            st = st.max_delay(max_delay);
        }
        st.take(self.take)
    }

    fn condition(&self) -> CodeCondition {
        CodeCondition::new(self.codes.clone())
    }
}

impl Default for RetrySetting {
    fn default() -> Self {
        Self {
            from_millis: 10,
            max_delay: Some(Duration::from_secs(1)),
            factor: 1u64,
            take: 5,
            codes: vec![Code::Unavailable, Code::Unknown, Code::Aborted],
        }
    }
}

pub async fn invoke<A, R, RT, C, E>(retry: Option<RT>, action: A) -> Result<R, E>
where
    E: TryAs<Status> + From<Status>,
    A: Action<Item = R, Error = E>,
    C: Condition<E>,
    RT: Retry<E, C> + Default,
{
    let retry = retry.unwrap_or_default();
    RetryIf::spawn(retry.strategy(), action, retry.condition()).await
}
/// Repeats retries when the specified error is detected.
/// The argument specified by 'v' can be reused for each retry.
pub async fn invoke_fn<R, V, A, RT, C, E>(retry: Option<RT>, mut f: impl FnMut(V) -> A, mut v: V) -> Result<R, E>
where
    E: TryAs<Status> + From<Status>,
    A: Future<Output = Result<R, (E, V)>>,
    C: Condition<E>,
    RT: Retry<E, C> + Default,
{
    let retry = retry.unwrap_or_default();
    let mut strategy = retry.strategy();
    loop {
        let result = f(v).await;
        let status = match result {
            Ok(s) => return Ok(s),
            Err(e) => {
                v = e.1;
                e.0
            }
        };
        if retry.condition().should_retry(&status) {
            let duration = strategy.next().ok_or(status)?;
            tokio::time::sleep(duration).await;
            tracing::trace!("retry fn");
        } else {
            return Err(status);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use tonic::{Code, Status};

    use crate::retry::{invoke, RetrySetting};

    #[tokio::test]
    async fn test_retry() {
        let retry = RetrySetting::default();
        let counter = Arc::new(Mutex::new(0));
        let action = || async {
            let mut lock = counter.lock().unwrap();
            *lock += 1;
            let result: Result<i32, Status> = Err(Status::new(Code::Aborted, "error"));
            result
        };
        let actual = invoke(Some(retry), action).await.unwrap_err();
        let expected = Status::new(Code::Aborted, "error");
        assert_eq!(actual.code(), expected.code());
        assert_eq!(*counter.lock().unwrap(), 6);
    }
}
