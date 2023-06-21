use reqwest::{Client, RequestBuilder};

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetJobRequest {
    /// The geographic location of the job. You must specify the location to run the job for the following scenarios:
    ///
    /// If the location to run a job is not in the us or the eu multi-regional location
    /// If the job's location is in a single region (for example, us-central1)
    /// For more information, see https://cloud.google.com/bigquery/docs/locations#specifying_your_location.
    pub location: Option<String>,
}

pub fn build(base_url: &str, client: &Client, project_id: &str, job_id: &str, data: &GetJobRequest) -> RequestBuilder {
    let url = format!("{}/projects/{}/jobs/{}", base_url, project_id, job_id);
    client.get(url).query(data)
}
