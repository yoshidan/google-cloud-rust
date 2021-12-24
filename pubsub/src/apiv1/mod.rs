pub mod publisher_client;
pub mod schema_client;

use google_cloud_gax::call_option::{Backoff, BackoffRetrySettings, BackoffRetryer};
use google_cloud_googleapis::{Code, Status};

fn default_setting() -> BackoffRetrySettings {
    BackoffRetrySettings {
        retryer: BackoffRetryer {
            backoff: Backoff::default(),
            //handle gRPC stream error (Status { code: Unknown, message: "transport error", source: None })
            codes: vec![Code::Unavailable, Code::Unknown],
        },
    }
}
