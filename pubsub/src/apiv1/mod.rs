pub mod publisher_client;
pub mod schema_client;
pub mod subscriber_client;
pub mod conn_pool;

use std::iter::Take;
use std::time::Duration;
use tokio_retry::Condition;
use tokio_retry::strategy::ExponentialBackoff;
use tonic::{IntoRequest, Request};
use google_cloud_gax::call_option::{Backoff, BackoffRetrySettings, BackoffRetryer};
use google_cloud_googleapis::{Code, Status};

fn default_setting() -> BackoffRetrySettings {
    BackoffRetrySettings {
        retryer: BackoffRetryer {
            backoff: Backoff::default(),
            codes: vec![Code::Unavailable, Code::Unknown, Code::Aborted],
        },
    }
}

fn create_request<T>(param_string: String, into_request: impl IntoRequest<T>) -> Request<T> {
    let mut request = into_request.into_request();
    let target = request.metadata_mut();
    if !param_string.is_empty() {
        target.append("x-goog-request-params", param_string.parse().unwrap());
    }
    request
}

#[derive(Clone)]
pub struct RetrySetting {
    pub from_millis: u64,
    pub max_delay: Option<Duration>,
    pub factor: u64,
    pub take: usize,
    pub codes: Vec<Code>
}

impl RetrySetting {
    pub fn strategy(&self) -> Take<ExponentialBackoff>  {
        let mut st= tokio_retry::strategy::ExponentialBackoff::from_millis(self.from_millis);
        if let Some(max_delay) = self.max_delay {
            st = st.max_delay(max_delay);
        }
        return st.take(self.take);

    }
    pub fn condition(&self) -> impl Condition<Status> + '_ {
        move |e: &Status| {
            for code in &self.codes {
                if *code == e.code() {
                    return true;
                }
            }
            return false;
        }
    }
}

impl Default for RetrySetting {
    fn default() -> Self {
        Self {
            from_millis: 10,
            max_delay: Some(Duration::from_secs(1)),
            factor: 1u64,
            take: 5,
            codes: vec![Code::Unavailable, Code::Unknown, Code::Aborted]
        }
    }
}