use std::iter::Take;
use std::marker::PhantomData;

use crate::session::SessionError;
use google_cloud_gax::grpc::{Code, Status};
use google_cloud_gax::retry::{CodeCondition, Condition, ExponentialBackoff, Retry, RetrySetting, TryAs};

pub struct TransactionCondition<E>
where
    E: TryAs<Status> + From<SessionError> + From<Status>,
{
    inner: CodeCondition,
    _marker: PhantomData<E>,
}

impl<E> Condition<E> for TransactionCondition<E>
where
    E: TryAs<Status> + From<SessionError> + From<Status>,
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
            return self.inner.should_retry(error);
        }
        false
    }
}

#[derive(Clone)]
pub struct TransactionRetrySetting {
    pub inner: RetrySetting,
}

impl<E> Retry<E, TransactionCondition<E>> for TransactionRetrySetting
where
    E: TryAs<Status> + From<SessionError> + From<Status>,
{
    fn strategy(&self) -> Take<ExponentialBackoff> {
        self.inner.strategy()
    }

    fn condition(&self) -> TransactionCondition<E> {
        TransactionCondition {
            inner: CodeCondition::new(self.inner.codes.clone()),
            _marker: PhantomData::default(),
        }
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

#[cfg(test)]
mod tests {
    use crate::client::{RunInTxError, TxError};
    use crate::retry::TransactionRetrySetting;
    use google_cloud_gax::grpc::{Code, Status};
    use google_cloud_gax::retry::{Condition, Retry};

    #[test]
    fn test_transaction_condition() {
        let err = &TxError::GRPC(Status::new(Code::Internal, "stream terminated by RST_STREAM"));
        let default = TransactionRetrySetting::default();
        assert!(!default.condition().should_retry(err));

        let err = &RunInTxError::GRPC(Status::new(Code::Aborted, ""));
        assert!(default.condition().should_retry(err));
    }
}
