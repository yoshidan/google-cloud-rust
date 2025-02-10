use std::collections::HashMap;

use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::dataset::DatasetReference;
use crate::http::job::{DmlStats, JobReference, SessionInfo};
use crate::http::table::TableSchema;
use crate::http::tabledata::list::Tuple;
use crate::http::types::{ConnectionProperty, DataFormatOptions, ErrorProto, QueryParameter};

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct QueryRequest {
    /// The resource type of the request.
    pub kind: String,
    /// Required. A query string to execute, using Google Standard SQL or legacy SQL syntax.
    /// Example: "SELECT COUNT(f1) FROM myProjectId.myDatasetId.myTableId".
    pub query: String,
    /// Optional. The maximum number of rows of data to return per page of results.
    /// Setting this flag to a small value such as 1000 and then paging through
    /// results might improve reliability when the query result set is large.
    /// In addition to this limit, responses are also limited to 10 MB.
    /// By default, there is no maximum row count, and only the byte limit applies.
    pub max_results: Option<i64>,
    /// Optional. Specifies the default datasetId and projectId to assume for any unqualified table names in the query.
    /// If not set, all table names in the query string must be qualified in the format 'datasetId.tableId'.
    pub default_dataset: Option<DatasetReference>,
    /// Optional. Optional: Specifies the maximum amount of time, in milliseconds,
    /// that the client is willing to wait for the query to complete.
    /// By default, this limit is 10 seconds (10,000 milliseconds).
    /// If the query is complete, the jobComplete field in the response is true.
    /// If the query has not yet completed, jobComplete is false.
    /// You can request a longer timeout period in the timeoutMs field.
    /// However, the call is not guaranteed to wait for the specified timeout;
    /// it typically returns after around 200 seconds (200,000 milliseconds), even if the query is not complete.
    /// If jobComplete is false, you can continue to wait for the query to complete
    /// by calling the getQueryResults method until the jobComplete field in the getQueryResults response is true.
    pub timeout_ms: Option<i64>,
    /// Optional. If set to true, BigQuery doesn't run the job.
    /// Instead, if the query is valid,
    /// BigQuery returns statistics about the job such as how many bytes would be processed.
    /// If the query is invalid, an error returns. The default value is false.
    pub dry_run: Option<bool>,
    /// Optional. Whether to look for the result in the query cache.
    /// The query cache is a best-effort cache that will be flushed whenever tables in the query are modified.
    /// The default value is true.
    pub use_query_cache: Option<bool>,
    /// Specifies whether to use BigQuery's legacy SQL dialect for this query.
    /// The default value is true. If set to false, the query will use
    /// BigQuery's GoogleSQL: https://cloud.google.com/bigquery/sql-reference/ When useLegacySql is set to false, the value of flattenResults is ignored; query will be run as if flattenResults is false.
    pub use_legacy_sql: bool,
    /// GoogleSQL only. Set to POSITIONAL to use positional (?) query parameters or
    /// to NAMED to use named (@myparam) query parameters in this query.
    pub parameter_mode: Option<String>,
    /// jobs.query parameters for GoogleSQL queries.
    pub query_parameters: Vec<QueryParameter>,
    /// The geographic location where the job should run.
    /// See details at https://cloud.google.com/bigquery/docs/locations#specifying_your_location.
    pub location: String,
    /// Optional. Output format adjustments.
    pub format_options: Option<DataFormatOptions>,
    /// Optional. Connection properties which can modify the query behavior.
    pub connection_properties: Vec<ConnectionProperty>,
    /// Optional. The labels associated with this query.
    /// Labels can be used to organize and group query jobs.
    /// Label keys and values can be no longer than 63 characters,
    /// can only contain lowercase letters, numeric characters, underscores and dashes.
    /// International characters are allowed. Label keys must start with a letter and each
    /// label in the list must have a different key.
    /// An object containing a list of "key": value pairs.
    /// Example: { "name": "wrench", "mass": "1.3kg", "count": "3" }.
    pub labels: Option<HashMap<String, String>>,
    ///Optional. Limits the bytes billed for this query.
    /// Queries with bytes billed above this limit will fail (without incurring a charge).
    /// If unspecified, the project default is used.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub maximum_bytes_billed: Option<i64>,
    /// Optional. A unique user provided identifier to ensure idempotent behavior for queries.
    /// Note that this is different from the jobId. It has the following properties:
    /// 1.It is case-sensitive, limited to up to 36 ASCII characters. A UUID is recommended.
    /// 2.Read only queries can ignore this token since they are nullipotent by definition.
    /// 3.For the purposes of idempotency ensured by the requestId,
    ///     a request is considered duplicate of another only if they have the same requestId and are actually duplicates.
    ///     When determining whether a request is a duplicate of another request,
    ///     all parameters in the request that may affect the result are considered.
    ///     For example, query, connectionProperties, queryParameters, useLegacySql are parameters that affect the result
    ///     and are considered when determining whether a request is a duplicate,
    ///     but properties like timeoutMs don't affect the result and are thus not considered.
    ///     Dry run query requests are never considered duplicate of another request.
    /// 4.When a duplicate mutating query request is detected, it returns:
    ///     a. the results of the mutation if it completes successfully within the timeout.
    ///     b. the running operation if it is still in progress at the end of the timeout.
    /// 5.Its lifetime is limited to 15 minutes.
    ///     In other words, if two requests are sent with the same requestId,
    ///     but more than 15 minutes apart, idempotency is not guaranteed.
    pub request_id: Option<String>,
    /// Optional. If true, creates a new session using a randomly generated sessionId.
    /// If false, runs query with an existing sessionId passed in ConnectionProperty,
    /// otherwise runs query in non-session mode.
    /// The session location will be set to QueryRequest.location if it is present,
    /// otherwise it's set to the default location based on existing routing logic.
    pub create_session: Option<bool>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct QueryResponse {
    /// The resource type.
    #[serde(default)]
    pub kind: String,
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
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub total_rows: Option<i64>,
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
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub total_bytes_processed: Option<i64>,
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
    pub cache_hit: Option<bool>,
    /// Output only. The number of rows affected by a DML statement.
    /// Present only for DML statements INSERT, UPDATE or DELETE.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub num_dml_affected_rows: Option<i64>,
    /// Output only. Information of the session if this job is part of one.
    pub session_info: Option<SessionInfo>,
    /// Output only. Detailed statistics for DML statements INSERT, UPDATE, DELETE, MERGE or TRUNCATE.
    pub dml_stats: Option<DmlStats>,
}

pub fn build(base_url: &str, client: &Client, project_id: &str, data: &QueryRequest) -> RequestBuilder {
    let url = format!("{}/projects/{}/queries", base_url, project_id);
    client.post(url).json(data)
}
