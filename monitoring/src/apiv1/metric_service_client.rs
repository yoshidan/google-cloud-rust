use std::sync::Arc;

use google_cloud_gax::conn::Channel;
use google_cloud_gax::create_request;
use google_cloud_gax::grpc::Response;
use google_cloud_gax::grpc::Status;
use google_cloud_gax::retry::{invoke, MapErr, RetrySetting};
use google_cloud_googleapis::monitoring::v3::metric_service_client::MetricServiceClient as InternalMetricServiceClient;
use google_cloud_googleapis::monitoring::v3::CreateTimeSeriesRequest;

use crate::apiv1::conn_pool::ConnectionManager;

#[derive(Clone, Debug)]
pub(crate) struct MetricServiceClient {
    cm: Arc<ConnectionManager>,
}

#[allow(dead_code)]
impl MetricServiceClient {
    /// create new metric client
    pub fn new(cm: ConnectionManager) -> Self {
        Self { cm: Arc::new(cm) }
    }

    #[inline]
    fn client(&self) -> InternalMetricServiceClient<Channel> {
        InternalMetricServiceClient::new(self.cm.conn())
    }

    /// create_time_series creates or adds data to one or more time series.
    /// If any time series could not be written, a corresponding failure message is
    /// included in the error response.
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn create_time_series(
        &self,
        req: CreateTimeSeriesRequest,
        retry: Option<RetrySetting>,
    ) -> Result<Response<()>, Status> {
        let name = &req.name;
        let action = || async {
            let mut client = self.client();
            let request = create_request(format!("name={name}"), req.clone());
            client.create_time_series(request).await.map_transient_err()
        };
        invoke(retry, action).await
    }
}
