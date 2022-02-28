use std::iter::Take;
use std::marker::PhantomData;

use crate::session::SessionError;
use google_cloud_gax::retry::{CodePredicate, ExponentialBackoff, Predicate, Retry, RetrySetting, TryAs};
use google_cloud_gax::status::{Code, Status};

pub struct TransactionPredicate<E>
where
    E: TryAs<Status> + From<SessionError> + From<Status>,
{
    inner: CodePredicate,
    _marker: PhantomData<E>,
}

impl<E> Predicate<E> for TransactionPredicate<E>
where
    E: TryAs<Status> + From<SessionError> + From<Status>,
{
    fn should_retry(&mut self, error: &E) -> bool {
        let status = match error.try_as() {
            Ok(s) => s,
            Err(_e) => return false,
        };
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
}

#[derive(Clone)]
pub struct TransactionRetrySetting {
    pub inner: RetrySetting,
}

impl<E> Retry<E, TransactionPredicate<E>> for TransactionRetrySetting
where
    E: TryAs<Status> + From<SessionError> + From<Status>,
{
    fn strategy(&self) -> Take<ExponentialBackoff> {
        self.inner.strategy()
    }

    fn predicate(&self) -> TransactionPredicate<E> {
        TransactionPredicate {
            inner: CodePredicate::new(self.inner.codes.clone()),
            _marker: PhantomData::default(),
        }
    }
}

impl TransactionRetrySetting {
    pub fn new(codes: Vec<Code>) -> Self {
        let mut inner = RetrySetting::default();
        inner.codes = codes;
        Self { inner }
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
    use google_cloud_gax::retry::{Predicate, Retry};
    use google_cloud_gax::status::Status;

    #[test]
    fn test_transaction_predicate() {
        let status = tonic::Status::new(tonic::Code::Internal, "stream terminated by RST_STREAM");
        let default = TransactionRetrySetting::default();
        assert!(!default.predicate().should_retry(&TxError::GRPC(Status::from(status))));

        let status = tonic::Status::new(tonic::Code::Aborted, "default");
        assert!(default
            .predicate()
            .should_retry(&RunInTxError::GRPC(Status::from(status))));
    }
}
