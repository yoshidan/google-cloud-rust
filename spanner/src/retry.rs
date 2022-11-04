use std::iter::Take;
use std::marker::PhantomData;

use google_cloud_gax::grpc::{Code, Status};
use google_cloud_gax::retry::{CodeCondition, Condition, ExponentialBackoff, Retry, RetrySetting, TryAs};

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
