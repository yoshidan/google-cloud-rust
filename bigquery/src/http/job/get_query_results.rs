use crate::http::job::JobReference;
use crate::http::table::TableSchema;
use crate::http::tabledata::list::Tuple;
use crate::http::types::{DataFormatOptions, ErrorProto};
use reqwest::{Client, RequestBuilder};

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetQueryResultsRequest {
    /// Zero-based index of the starting row.
    pub start_index: i64,
    /// Page token, returned by a previous call, to request the next page of results.
    pub page_token: Option<String>,
    /// Maximum number of results to read.
    pub max_results: Option<i64>,
    /// Optional: Specifies the maximum amount of time, in milliseconds,
    /// that the client is willing to wait for the query to complete.
    /// By default, this limit is 10 seconds (10,000 milliseconds).
    /// If the query is complete, the jobComplete field in the response is true.
    /// If the query has not yet completed, jobComplete is false.
    /// You can request a longer timeout period in the timeoutMs field.
    /// However, the call is not guaranteed to wait for the specified timeout;
    /// it typically returns after around 200 seconds (200,000 milliseconds),
    /// even if the query is not complete.
    /// If jobComplete is false, you can continue to wait for the query to complete
    /// by calling the getQueryResults method until the jobComplete field in the getQueryResults response is true.
    pub timeout_ms: Option<i64>,
    /// The geographic location of the job. You must specify the location to run the job for the following scenarios:
    /// If the location to run a job is not in the us or the eu multi-regional location
    /// If the job's location is in a single region (for example, us-central1)
    /// For more information, see https://cloud.google.com/bigquery/docs/locations#specifying_your_location.
    pub location: Option<String>,
    /// Optional. Output format adjustments.
    pub format_options: Option<DataFormatOptions>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetQueryResultsResponse {
    /// The resource type.
    pub kind: String,
    /// A hash of this response.
    pub etag: String,
    /// The schema of the results. Present only when the query completes successfully.
    pub schema: Option<TableSchema>,
    /// Reference to the Job that was created to run the query.
    /// This field will be present even if the original request timed out,
    /// in which case jobs.getQueryResults can be used to read the results once the query has completed.
    /// Since this API only returns the first page of results,
    /// subsequent pages can be fetched via the same mechanism (jobs.getQueryResults).
    pub job_reference: JobReference,
    /// The total number of rows in the complete query result set,
    /// which can be more than the number of rows in this single page of results.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub total_rows: i64,
    /// A token used for paging results.
    /// A non-empty token indicates that additional results are available.
    /// To see additional results, query the jobs.getQueryResults method.
    /// For more information, see Paging through table data.
    pub page_token: Option<String>,
    /// An object with as many results as can be contained within the maximum permitted reply size.
    /// To get any additional rows, you can call jobs.getQueryResults and specify the jobReference returned above.
    pub rows: Option<Vec<Tuple>>,
    /// The total number of bytes processed for this query.
    /// If this query was a dry run, this is the number of bytes that would be processed if the query were run.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub total_bytes_processed: i64,
    /// Whether the query has completed or not.
    /// If rows or totalRows are present, this will always be true.
    /// If this is false, totalRows will not be available.
    pub job_complete: bool,
    /// Output only. The first errors or warnings encountered during the running of the job.
    /// The final message includes the number of errors that caused the process to stop.
    /// Errors here do not necessarily mean that the job has completed or was unsuccessful.
    /// For more information about error messages, see Error messages.
    pub errors: Option<Vec<ErrorProto>>,
    /// Whether the query result was fetched from the query cache.
    pub cache_hit: bool,
    /// Output only. The number of rows affected by a DML statement.
    /// Present only for DML statements INSERT, UPDATE or DELETE.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub num_dml_affected_rows: Option<i64>,
}

pub fn build(
    base_url: &str,
    client: &Client,
    project_id: &str,
    job_id: &str,
    data: &GetQueryResultsRequest,
) -> RequestBuilder {
    let url = format!("{}/projects/{}/queries/{}", base_url, project_id, job_id);
    println!("{:?}", serde_json::to_string(data).unwrap());
    client.get(url).query(data)
}
