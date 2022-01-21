pub mod publisher_client;
pub mod schema_client;
pub mod subscriber_client;
pub mod conn_pool;

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
