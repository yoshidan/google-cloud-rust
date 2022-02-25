use crate::status::{Code, Status};
use std::future::Future;
use std::iter::Take;

use std::time::Duration;
use tokio::select;
use tokio_retry::strategy::ExponentialBackoff;
use tokio_retry::{Action, Condition};
use tokio_util::sync::CancellationToken;

use tokio_retry::RetryIf;

pub trait TryAs<T> {
    fn try_as(&self) -> Result<&T, ()>;
}

impl TryAs<Status> for Status {
    fn try_as(&self) -> Result<&Status, ()> {
        Ok(self)
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

impl Retry<Status, CodeCondition> for RetrySetting {
    fn strategy(&self) -> Take<ExponentialBackoff> {
        let mut st = tokio_retry::strategy::ExponentialBackoff::from_millis(self.from_millis);
        if let Some(max_delay) = self.max_delay {
            st = st.max_delay(max_delay);
        }
        return st.take(self.take);
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

pub async fn invoke<A, R, RT, C, E>(
    ctx: CancellationToken,
    opt: Option<RT>,
    action: A,
) -> Result<R, E>
where
    E: TryAs<Status> + From<Status>,
    A: Action<Item = R, Error = E>,
    C: Condition<E>,
    RT: Retry<E, C> + Default,
{
    let setting = opt.unwrap_or_default();
    select! {
        _ = ctx.cancelled() => Err(Status::new(tonic::Status::cancelled("client cancel")).into()),
        v = RetryIf::spawn(setting.strategy(), action, setting.condition()) => v
    }
}
/// Repeats retries when the specified error is detected.
/// The argument specified by 'v' can be reused for each retry.
pub async fn invoke_fn<R, V, A, RT, C, E>(
    ctx: CancellationToken,
    opt: Option<RT>,
    mut f: impl FnMut(V) -> A,
    mut v: V,
) -> Result<R, E>
where
    E: TryAs<Status> + From<Status>,
    A: Future<Output = Result<R, (E, V)>>,
    C: Condition<E>,
    RT: Retry<E, C> + Default,
{
    let fn_loop = async {
        let opt = opt.unwrap_or_default();
        let mut strategy = opt.strategy();
        loop {
            let result = f(v).await;
            let status = match result {
                Ok(s) => return Ok(s),
                Err(e) => {
                    v = e.1;
                    e.0
                }
            };
            if opt.condition().should_retry(&status) {
                let duration = match strategy.next() {
                    None => return Err(status),
                    Some(s) => s,
                };
                tokio::time::sleep(duration).await
            } else {
                return Err(status);
            };
        }
    };

    select! {
       _ = ctx.cancelled() => Err(Status::new(tonic::Status::cancelled("client cancel")).into()),
       v = fn_loop => v
    }
}
