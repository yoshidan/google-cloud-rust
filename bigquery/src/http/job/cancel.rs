use reqwest::header::CONTENT_LENGTH;
use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::job::Job;

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct CancelJobRequest {
    /// The geographic location of the job. You must specify the location to run the job for the following scenarios:
    ///
    /// If the location to run a job is not in the us or the eu multi-regional location
    /// If the job's location is in a single region (for example, us-central1)
    /// For more information, see https://cloud.google.com/bigquery/docs/locations#specifying_your_location.
    pub location: Option<String>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct CancelJobResponse {
    /// The resource type of the response.
    pub kind: String,
    /// The final state of the job
    pub job: Job,
}

pub fn build(
    base_url: &str,
    client: &Client,
    project_id: &str,
    job_id: &str,
    data: &CancelJobRequest,
) -> RequestBuilder {
    let url = format!("{}/projects/{}/jobs/{}/cancel", base_url, project_id, job_id);
    client.post(url).query(data).header(CONTENT_LENGTH, 0)
}
