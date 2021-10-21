use tokio::time::Duration;
use tonic::{Code, Status};

use google_cloud_gax::call_option::{Backoff, BackoffRetryer, RetrySettings, Retryer};

pub(crate) type TransactionRetrySettings = RetrySettings<TransactionRetryer>;

pub(crate) fn new_default_tx_retry() -> TransactionRetrySettings {
    RetrySettings {
        retryer: TransactionRetryer::new(vec![Code::Aborted]),
    }
}

pub(crate) fn new_tx_retry_with_codes(codes: Vec<Code>) -> TransactionRetrySettings {
    RetrySettings {
        retryer: TransactionRetryer::new(codes),
    }
}

#[derive(Clone)]
pub(crate) struct TransactionRetryer {
    retryer: BackoffRetryer,
}

impl TransactionRetryer {
    pub fn new(codes: Vec<Code>) -> TransactionRetryer {
        let mut backoff = Backoff::default();
        backoff.initial = Duration::from_millis(20);
        backoff.max = Duration::from_secs(32);
        backoff.timeout = Duration::from_secs(32);
        TransactionRetryer {
            retryer: BackoffRetryer { backoff, codes },
        }
    }
}

impl Retryer for TransactionRetryer {
    fn retry(&mut self, status: &Status) -> Option<Duration> {
        let code = status.code();
        if code == Code::Internal
            && !status.message().contains("stream terminated by RST_STREAM")
            && !status
                .message()
                .contains("HTTP/2 error code: INTERNAL_ERROR")
            && !status
                .message()
                .contains("Connection closed with unknown cause")
            && !status
                .message()
                .contains("Received unexpected EOS on DATA frame from server")
        {
            return None;
        }
        //TODO extract server delay
        return self.retryer.retry(status);
    }
}

#[cfg(test)]
mod tests {
    use crate::retry::TransactionRetryer;
    use google_cloud_gax::call_option::Retryer;
    use std::thread::sleep;
    use std::time::Duration;
    use tonic::{Code, Status};

    #[test]
    fn test_retry() {
        let mut retry = TransactionRetryer::new(vec![Code::Internal]);
        let mut durations = vec![];
        retry.retryer.backoff.timeout = Duration::from_millis(100);
        loop {
            match retry.retry(&Status::new(
                Code::Internal,
                "stream terminated by RST_STREAM",
            )) {
                None => break,
                Some(d) => durations.push(d),
            };
            sleep(Duration::from_millis(50));
        }
        println!("retry count = {}", durations.len());
        assert!(!durations.is_empty());
    }

    #[test]
    fn test_retry_invalid_message() {
        let mut retry = TransactionRetryer::new(vec![Code::Internal]);
        let mut durations = vec![];
        retry.retryer.backoff.timeout = Duration::from_millis(100);
        loop {
            match retry.retry(&Status::new(Code::Internal, "test")) {
                None => break,
                Some(d) => durations.push(d),
            };
            sleep(Duration::from_millis(50));
        }
        println!("retry count = {}", durations.len());
        assert!(durations.is_empty());
    }
}
