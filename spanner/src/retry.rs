use std::future::Future;
use std::iter::Take;
use std::marker::PhantomData;

use google_cloud_gax::grpc::{Code, Status};
use google_cloud_gax::retry::{CodeCondition, Condition, ExponentialBackoff, Retry, RetrySetting, TryAs};

use crate::session::{is_session_not_found_status, ManagedSession, SessionError};

pub struct TransactionCondition<E>
where
    E: TryAs<Status>,
{
    inner: CodeCondition,
    _marker: PhantomData<E>,
}

impl<E> Condition<E> for TransactionCondition<E>
where
    E: TryAs<Status>,
{
    fn should_retry(&mut self, error: &E) -> bool {
        if let Some(status) = error.try_as() {
            let code = status.code();
            if code == Code::Internal
                && !status.message().contains("stream terminated by RST_STREAM")
                && !status.message().contains("HTTP/2 error code: INTERNAL_ERROR")
                && !status.message().contains("Connection closed with unknown cause")
                && !status
                    .message()
                    .contains("Received unexpected EOS on DATA frame from server")
            {
                return false;
            }
            if code == Code::NotFound {
                return is_session_not_found_status(status);
            }
            return self.inner.should_retry(error);
        }
        false
    }
}

pub struct TransactionRetry<E>
where
    E: TryAs<Status>,
{
    strategy: Take<ExponentialBackoff>,
    condition: TransactionCondition<E>,
}

impl<E> TransactionRetry<E>
where
    E: TryAs<Status>,
{
    pub async fn next(&mut self, status: E) -> Result<(), E> {
        let duration = if self.condition.should_retry(&status) {
            self.strategy.next()
        } else {
            None
        };
        match duration {
            Some(duration) => {
                tokio::time::sleep(duration).await;
                Ok(())
            }
            None => Err(status),
        }
    }

    pub fn new() -> Self {
        let setting = TransactionRetrySetting::default();
        let strategy = <TransactionRetrySetting as Retry<E, TransactionCondition<E>>>::strategy(&setting);
        Self {
            strategy,
            condition: setting.condition(),
        }
    }
}

impl<E> Default for TransactionRetry<E>
where
    E: TryAs<Status>,
{
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct TransactionRetrySetting {
    pub inner: RetrySetting,
}

impl<E> Retry<E, TransactionCondition<E>> for TransactionRetrySetting
where
    E: TryAs<Status>,
{
    fn strategy(&self) -> Take<ExponentialBackoff> {
        self.inner.strategy()
    }

    fn condition(&self) -> TransactionCondition<E> {
        TransactionCondition {
            inner: CodeCondition::new(self.inner.codes.clone()),
            _marker: PhantomData,
        }
    }

    fn notify(error: &E, duration: std::time::Duration) {
        if let Some(status) = error.try_as() {
            tracing::trace!("transaction retry fn, error: {:?}, duration: {:?}", status, duration);
        };
    }
}

impl TransactionRetrySetting {
    pub fn new(codes: Vec<Code>) -> Self {
        Self {
            inner: RetrySetting {
                codes,
                ..Default::default()
            },
        }
    }
}

impl Default for TransactionRetrySetting {
    fn default() -> Self {
        TransactionRetrySetting::new(vec![Code::Aborted])
    }
}

/// Result wrapper used by higher-level helpers to describe whether a failed
/// attempt should be retried with a refreshed session.
pub enum SessionRetryAction<E> {
    Retry { error: E, session: Box<ManagedSession> },
    Fail(E),
}

impl<E> SessionRetryAction<E> {
    pub fn retry(error: E, session: ManagedSession) -> Self {
        SessionRetryAction::Retry {
            error,
            session: Box::new(session),
        }
    }

    pub fn fail(error: E) -> Self {
        SessionRetryAction::Fail(error)
    }
}

impl<E> From<SessionRetryAction<E>> for (E, Option<ManagedSession>) {
    fn from(action: SessionRetryAction<E>) -> Self {
        match action {
            SessionRetryAction::Retry { error, session } => (error, Some(*session)),
            SessionRetryAction::Fail(error) => (error, None),
        }
    }
}

impl<E> From<(E, Option<ManagedSession>)> for SessionRetryAction<E> {
    fn from(value: (E, Option<ManagedSession>)) -> Self {
        match value {
            (error, Some(session)) => SessionRetryAction::Retry {
                error,
                session: Box::new(session),
            },
            (error, None) => SessionRetryAction::Fail(error),
        }
    }
}

/// Retry helper that understands how to renew sessions when Cloud Spanner reports
/// `Session not found`. The closure receives ownership of the current session
/// (wrapped in `Option`) and must return it on failure so it can be retried.
pub async fn invoke_with_session_retry<R, Fut, F, E>(
    retry: Option<TransactionRetrySetting>,
    mut session: Option<ManagedSession>,
    mut f: F,
) -> Result<R, E>
where
    E: TryAs<Status> + From<Status> + From<SessionError>,
    Fut: Future<Output = Result<R, (E, Option<ManagedSession>)>>,
    F: FnMut(Option<ManagedSession>) -> Fut,
{
    let retry = retry.unwrap_or_default();
    let mut strategy = <TransactionRetrySetting as Retry<E, TransactionCondition<E>>>::strategy(&retry);
    let mut condition = <TransactionRetrySetting as Retry<E, TransactionCondition<E>>>::condition(&retry);

    loop {
        let current_session = session.take();
        match f(current_session).await {
            Ok(value) => return Ok(value),
            Err((err, returned_session)) => {
                session = returned_session;
                if !condition.should_retry(&err) {
                    return Err(err);
                }

                let mut skip_delay = false;
                if let Some(status) = err.try_as() {
                    if is_session_not_found_status(status) {
                        if let Some(ref mut managed_session) = session {
                            if let Err(renew_err) = managed_session.renew(None).await {
                                return Err(E::from(renew_err));
                            }
                            skip_delay = true;
                        } else {
                            return Err(err);
                        }
                    }
                }

                if skip_delay {
                    continue;
                }

                if let Some(duration) = strategy.next() {
                    tokio::time::sleep(duration).await;
                } else {
                    return Err(err);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use google_cloud_gax::grpc::{Code, Status};
    use google_cloud_gax::retry::{Condition, Retry};

    use crate::client::Error;
    use crate::retry::TransactionRetrySetting;

    #[test]
    fn test_transaction_condition() {
        let err = &Error::GRPC(Status::new(Code::Internal, "stream terminated by RST_STREAM"));
        let default = TransactionRetrySetting::default();
        assert!(!default.condition().should_retry(err));

        let err = &Error::GRPC(Status::new(Code::Aborted, ""));
        assert!(default.condition().should_retry(err));
    }

    #[test]
    fn test_session_not_found_should_retry() {
        let err = &Error::GRPC(Status::new(
            Code::NotFound,
            "Session not found: projects/local/instances/test/databases/db/sessions/session-id",
        ));
        let default = TransactionRetrySetting::default();
        assert!(
            default.condition().should_retry(err),
            "Session not found should be treated as retryable"
        );
    }
}
