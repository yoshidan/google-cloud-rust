use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::job::{JobConfiguration, JobReference, JobState, JobStatistics, JobStatus};
use crate::http::types::ErrorProto;

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Projection {
    #[default]
    Manual,
    Full,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListJobsRequest {
    /// Whether to display jobs owned by all users in the project. Default False.
    pub all_users: Option<bool>,
    /// The maximum number of results to return in a single response page. Leverage the page tokens to iterate through the entire collection.
    pub max_results: Option<i64>,
    /// Min value for job creation time, in milliseconds since the POSIX epoch.
    /// If set, only jobs created after or at this timestamp are returned.
    pub min_creation_time: Option<u64>,
    /// Max value for job creation time, in milliseconds since the POSIX epoch.
    /// If set, only jobs created before or at this timestamp are returned.
    pub max_creation_time: Option<u64>,
    /// Restrict information returned to a set of selected fields
    pub projection: Option<Projection>,
    /// Filter for job state
    pub state_filter: Option<Vec<JobState>>,
    /// If set, show only child jobs of the specified parent. Otherwise, show all top-level jobs.
    pub parent_job_id: String,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct JobOverview {
    /// Unique opaque ID of the job.
    pub id: String,
    /// The resource type.
    pub kind: String,
    /// Unique opaque ID of the job.
    pub job_reference: JobReference,
    /// Running state of the job. When the state is DONE,
    /// errorResult can be checked to determine whether the job succeeded or failed.
    pub state: JobState,
    /// A result object that will be present only if the job has failed.
    pub error_result: Option<ErrorProto>,
    /// Output only. Information about the job, including starting time and ending time of the job.
    pub statistics: Option<JobStatistics>,
    /// Required. Describes the job configuration.
    pub configuration: JobConfiguration,
    /// [Full-projection-only] Describes the status of this job.
    pub status: Option<JobStatus>,
    /// [Full-projection-only] Email address of the user who ran the job.
    pub user_email: Option<String>,
    /// [Full-projection-only] String representation of identity of requesting party.
    /// Populated for both first- and third-party identities.
    /// Only present for APIs that support third-party identities.
    pub principal_subject: Option<String>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListJobsResponse {
    /// A hash of this page of results.
    pub etag: String,
    /// The resource type of the response.
    pub kind: String,
    /// A token to request the next page of results.
    pub next_page_token: Option<String>,
    /// List of jobs that were requested.
    pub jobs: Vec<JobOverview>,
}

pub fn build(
    base_url: &str,
    client: &Client,
    project_id: &str,
    data: &ListJobsRequest,
    page_token: Option<String>,
) -> RequestBuilder {
    let url = format!("{}/projects/{}/jobs", base_url, project_id);
    let builder = client.get(url).query(data);
    if let Some(page_token) = page_token {
        builder.query(&[("pageToken", page_token.as_str())])
    } else {
        builder
    }
}
