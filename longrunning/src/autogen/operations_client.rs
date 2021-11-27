use google_cloud_gax::call_option::{Backoff, BackoffRetrySettings, BackoffRetryer};
use google_cloud_googleapis::longrunning::operations_client::OperationsClient as InternalOperationsClient;
use google_cloud_googleapis::Code;
use google_cloud_grpc::conn::{Error, TokenSource};
use std::time::Duration;
use tonic::transport::Channel;

fn default_setting() -> BackoffRetrySettings {
    let mut backoff = Backoff::default();
    backoff.initial = Duration::from_millis(500);
    backoff.max = Duration::from_millis(10000);
    backoff.multiplier = 2.0;
    BackoffRetrySettings {
        retryer: BackoffRetryer {
            backoff,
            codes: vec![Code::Unavailable, Code::Unknown],
        },
    }
}

pub struct OperationsClient {
    inner: InternalOperationsClient<Channel>,
    token_source: TokenSource,
}

impl OperationsClient {
    pub async fn new(channel: Channel, token_source: TokenSource) -> Result<Self, Error> {
        Ok(OperationsClient {
            inner: InternalOperationsClient::new(channel),
            token_source,
        })
    }
}
