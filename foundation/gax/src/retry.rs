use crate::status::{Code, Status};
use std::future::Future;
use std::iter::Take;

use std::time::Duration;
use tokio::select;

use crate::cancel::CancellationToken;

#[derive(Debug, Clone)]
pub struct ExponentialBackoff {
    current: u64,
    base: u64,
    factor: u64,
    max_delay: Option<Duration>,
}

impl ExponentialBackoff {
    pub fn from_millis(base: u64) -> ExponentialBackoff {
        ExponentialBackoff {
            current: base,
            base,
            factor: 1u64,
            max_delay: None,
        }
    }
}

impl Iterator for ExponentialBackoff {
    type Item = Duration;

    fn next(&mut self) -> Option<Duration> {
        // set delay duration by applying factor
        let duration = if let Some(duration) = self.current.checked_mul(self.factor) {
            Duration::from_millis(duration)
        } else {
            Duration::from_millis(u64::MAX)
        };

        // check if we reached max delay
        if let Some(ref max_delay) = self.max_delay {
            if duration > *max_delay {
                return Some(*max_delay);
            }
        }

        if let Some(next) = self.current.checked_mul(self.base) {
            self.current = next;
        } else {
            self.current = u64::MAX;
        }

        Some(duration)
    }
}


pub trait TryAs<T> {
    fn try_as(&self) -> Result<&T, ()>;
}

impl TryAs<Status> for Status {
    fn try_as(&self) -> Result<&Status, ()> {
        Ok(self)
    }
}

pub trait Predicate<E> {
    fn should_retry(&mut self, error: &E) -> bool;
}

pub trait Retry<E: TryAs<Status>, T: Predicate<E>> {
    fn strategy(&self) -> Take<ExponentialBackoff>;
    fn predicate(&self) -> T;
}

pub struct CodePredicate {
    codes: Vec<Code>,
}

impl CodePredicate {
    pub fn new(codes: Vec<Code>) -> Self {
        Self { codes }
    }
}

impl<E> Predicate<E> for CodePredicate
where
    E: TryAs<Status>,
{
    fn should_retry(&mut self, error: &E) -> bool {
        let status = match error.try_as() {
            Ok(s) => s,
            Err(_e) => return false,
        };
        for code in &self.codes {
            if *code == status.code() {
                return true;
            }
        }
        return false;
    }
}

#[derive(Clone)]
pub struct RetrySetting {
    pub from_millis: u64,
    pub max_delay: Option<Duration>,
    pub factor: u64,
    pub take: usize,
    pub codes: Vec<Code>,
}

impl Retry<Status, CodePredicate> for RetrySetting {
    fn strategy(&self) -> Take<ExponentialBackoff> {
        let mut st = ExponentialBackoff::from_millis(self.from_millis);
        st.max_delay = self.max_delay;
        return st.take(self.take);
    }

    fn predicate(&self) -> CodePredicate {
        CodePredicate::new(self.codes.clone())
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

pub async fn invoke<A, R, RT, C, E>(
    cancel: Option<CancellationToken>,
    retry: Option<RT>,
    mut a: impl FnMut() -> A
) -> Result<R, E>
where
    E: TryAs<Status> + From<Status>,
    A: Future<Output = Result<R, E>>,
    C: Predicate<E>,
    RT: Retry<E, C> + Default,
{
    let fn_loop = async {
        let retry = retry.unwrap_or_default();
        let mut strategy = retry.strategy();
        loop {
            let result = a().await;
            let status = match result {
                Ok(s) => return Ok(s),
                Err(e) => e
            };
            if !retry.predicate().should_retry(&status) {
                return Err(status)
            }
            match strategy.next() {
                None => return Err(status),
                Some(duration) => tokio::time::sleep(duration).await
            };
        }
    };

    match cancel {
        Some(cancel) => {
            select! {
                _ = cancel.cancelled() => Err(Status::new(tonic::Status::cancelled("client cancel")).into()),
                v = fn_loop => v
            }
        }
        None => fn_loop.await,
    }
}

/// Repeats retries when the specified error is detected.
/// The argument specified by 'v' can be reused for each retry.
pub async fn invoke_fn<R, V, A, RT, C, E>(
    cancel: Option<CancellationToken>,
    retry: Option<RT>,
    mut f: impl FnMut(V) -> A,
    mut v: V,
) -> Result<R, E>
where
    E: TryAs<Status> + From<Status>,
    A: Future<Output = Result<R, (E, V)>>,
    C: Predicate<E>,
    RT: Retry<E, C> + Default,
{
    let fn_loop = async {
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
            if !retry.predicate().should_retry(&status) {
                return Err(status)
            }
            match strategy.next() {
                None => return Err(status),
                Some(duration) => tokio::time::sleep(duration).await
            };
        }
    };

    match cancel {
        Some(cancel) => {
            select! {
                _ = cancel.cancelled() => Err(Status::new(tonic::Status::cancelled("client cancel")).into()),
                v = fn_loop => v
            }
        }
        None => fn_loop.await,
    }
}
