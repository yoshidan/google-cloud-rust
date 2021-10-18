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
        TransactionRetryer {
            retryer: BackoffRetryer {
                backoff: Backoff {
                    initial: Duration::from_millis(20),
                    max: Duration::from_secs(32),
                    multiplier: 1.3,
                    cur: Duration::from_nanos(0),
                },
                codes,
            },
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
