use google_cloud_gax::grpc::Code;
use google_cloud_gax::retry::RetrySetting;

pub mod conn_pool;
pub mod publisher_client;
pub mod schema_client;
pub mod subscriber_client;

pub fn default_retry_setting() -> RetrySetting {
    let mut setting = RetrySetting::default();
    setting.codes.push(Code::DeadlineExceeded);
    setting.codes.push(Code::Internal);
    setting.codes.push(Code::ResourceExhausted);
    return setting;
}
