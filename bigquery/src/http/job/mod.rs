use std::collections::HashMap;
use std::iter::Map;
use crate::http::dataset::DatasetReference;
use crate::http::routine::RoutineReference;
use crate::http::table::{Clustering, ExternalDataConfiguration, RangePartitioning, TableReference, TableSchema, TimePartitioning, UserDefinedFunctionResource};
use crate::http::types::{ConnectionProperty, EncryptionConfiguration, QueryParameter};

pub mod delete;

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KeyResultStatementKind {
    #[default]
    Last,
    FirstSelect
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScriptOptions {
    /// Timeout period for each statement in a script.
    #[serde(deserialize_with = "crate::http::from_str_option")]
    #[serde(default)]
    pub statement_timeout_ms: Option<i64>,
    /// Limit on the number of bytes billed per statement. Exceeding this budget results in an error.
    #[serde(deserialize_with = "crate::http::from_str_option")]
    #[serde(default)]
    pub statement_byte_budget : Option<i64>,
    /// Determines which statement in the script represents the "key result",
    /// used to populate the schema and query results of the script job. Default is LAST.
    pub key_result_statement: Option<KeyResultStatementKind>
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CreateDisposition {
    /// If the table does not exist, BigQuery creates the table.
    #[default]
    CreateIfNeeded,
    /// The table must already exist. If it does not, a 'notFound' error is returned in the job result.
    CreateNever
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WriteDisposition {
    /// If the table already exists, BigQuery overwrites the table data and uses the schema from the query result..
    WriteTruncate,
    /// If the table already exists, BigQuery appends the data to the table..
    WriteAppend,
    /// If the table already exists and contains data, a 'duplicate' error is returned in the job result.
    #[default]
    WriteEmpty
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Priority {
    #[default]
    Interactive,
    Batch,
}
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SchemaUpdateOption {
    /// allow adding a nullable field to the schema.
    AllowFieldAddition,
    /// allow relaxing a required field in the original schema to nullable.
    AllowFieldRelaxation
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct JobConfigurationQuery {
    /// [Required] SQL query text to execute.
    /// The useLegacySql field can be used to indicate whether the query uses legacy SQL or GoogleSQL.
    pub query: String,
    /// Optional. Describes the table where the query results should be stored.
    /// This property must be set for large results that exceed the maximum response size. For queries that produce anonymous (cached) results, this field will be populated by BigQuery.
    pub destination_table: Option<TableReference>,
    /// Optional. You can specify external table definitions,
    /// which operate as ephemeral tables that can be queried. These definitions are configured using a JSON map, where the string key represents the table identifier, and the value is the corresponding external data configuration object.
    /// An object containing a list of "key": value pairs. Example: { "name": "wrench", "mass": "1.3kg", "count": "3" }.
    pub table_definitions: Option<HashMap<String,ExternalDataConfiguration>>,
    /// Describes user-defined function resources used in the query.
    pub user_defined_function_resources: Option<Vec<UserDefinedFunctionResource>>,
    /// Optional. Specifies whether the job is allowed to create new tables. The following values are supported:
    /// CREATE_IF_NEEDED: If the table does not exist, BigQuery creates the table.
    /// CREATE_NEVER: The table must already exist. If it does not, a 'notFound' error is returned in the job result.
    /// The default value is CREATE_IF_NEEDED. Creation, truncation and append actions occur as one atomic update upon job completion.
    pub create_disposition: Option<CreateDisposition>,
    /// Optional. Specifies the action that occurs if the destination table already exists. The following values are supported:
    /// WRITE_TRUNCATE: If the table already exists, BigQuery overwrites the table data and uses the schema from the query result.
    /// WRITE_APPEND: If the table already exists, BigQuery appends the data to the table.
    /// WRITE_EMPTY: If the table already exists and contains data, a 'duplicate' error is returned in the job result.
    /// The default value is WRITE_EMPTY. Each action is atomic and only occurs if BigQuery is able to complete the job successfully. Creation, truncation and append actions occur as one atomic update upon job completion.
    pub write_disposition: Option<WriteDisposition>,
    /// Optional. Specifies the default dataset to use for unqualified table names in the query.
    /// This setting does not alter behavior of unqualified dataset names.
    /// Setting the system variable @@dataset_id achieves the same behavior.
    pub default_dataset: Option<DatasetReference>,
    /// Optional. Specifies a priority for the query.
    /// Possible values include INTERACTIVE and BATCH. The default value is INTERACTIVE.
    pub priority: Optiopn<Priority>,
    /// Optional. If true and query uses legacy SQL dialect,
    /// allows the query to produce arbitrarily large result tables at a slight cost in performance.
    /// Requires destinationTable to be set.
    /// For GoogleSQL queries, this flag is ignored and large results are always allowed.
    /// However, you must still set destinationTable when result size exceeds the allowed maximum response size.
    pub allow_large_results: Option<bool>,
    /// Optional. Whether to look for the result in the query cache.
    /// The query cache is a best-effort cache that will be flushed whenever tables in the query are modified.
    /// Moreover, the query cache is only available when a query does not have a destination table specified.
    /// The default value is true.
    pub use_query_cache: Option<bool>,
    /// Optional. If true and query uses legacy SQL dialect,
    /// flattens all nested and repeated fields in the query results.
    /// allowLargeResults must be true if this is set to false. For GoogleSQL queries,
    /// this flag is ignored and results are never flattened.
    pub flatten_results: Option<bool>,
    /// Limits the bytes billed for this job.
    /// Queries that will have bytes billed beyond this limit will fail (without incurring a charge).
    /// If unspecified, this will be set to your project default.
    #[serde(deserialize_with = "crate::http::from_str_option")]
    #[serde(default)]
    pub maximum_bytes_billed: Option<i64>,
    /// Optional. Specifies whether to use BigQuery's legacy SQL dialect for this query.
    /// The default value is true. If set to false, the query will use
    /// BigQuery's GoogleSQL: https://cloud.google.com/bigquery/sql-reference/
    /// When useLegacySql is set to false, the value of flattenResults is ignored;
    /// query will be run as if flattenResults is false.
    pub use_legacy_sql: Option<bool>,
    /// GoogleSQL only. Set to POSITIONAL to use positional (?) query parameters or to NAMED to use named (@myparam) query parameters in this query.
    pub parameter_mode: Option<String>,
    /// Query parameters for GoogleSQL queries.
    pub query_parameters: Vec<QueryParameter>,
    /// Allows the schema of the destination table to be updated as a side effect of the query job. Schema update options are supported in two cases: when writeDisposition is WRITE_APPEND; when writeDisposition is WRITE_TRUNCATE and the destination table is a partition of a table, specified by partition decorators. For normal tables, WRITE_TRUNCATE will always overwrite the schema. One or more of the following values are specified:
    /// ALLOW_FIELD_ADDITION: allow adding a nullable field to the schema.
    /// ALLOW_FIELD_RELAXATION: allow relaxing a required field in the origin
    pub schema_update_options: Option<Vec<SchemaUpdateOption>>,
    /// Time-based partitioning specification for the destination table.
    /// Only one of timePartitioning and rangePartitioning should be specified
    pub time_partitioning: Option<TimePartitioning>,
    /// Range partitioning specification for the destination table.
    /// Only one of timePartitioning and rangePartitioning should be specified.
    pub range_partitioning: Option<RangePartitioning>,
    /// Clustering specification for the destination table.
    pub clustering: Option<Clustering>,
    /// Custom encryption configuration (e.g., Cloud KMS keys)
    pub destination_encryption_configuration : Option<EncryptionConfiguration>,
    /// Options controlling the execution of scripts.
    pub script_options: Option<ScriptOptions>,
    /// Connection properties which can modify the query behavior.
    pub connection_properties: Option<Vec<ConnectionProperty>>,
    /// if this property is true, the job creates a new session using a randomly generated sessionId.
    /// To continue using a created session with subsequent queries,
    /// pass the existing session identifier as a ConnectionProperty value.
    /// The session identifier is returned as part of the SessionInfo message within the query statistics.
    /// The new session's location will be set to Job.JobReference.location if it is present,
    /// otherwise it's set to the default location based on existing routing logic.
    pub create_session: bool,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct JobConfiguration {
    /// Output only. The type of the job. Can be QUERY, LOAD, EXTRACT, COPY or UNKNOWN.
    pub job_type: String,
    /// [Pick one] Configures a query job.
    pub query: Option<JobConfigurationQuery>,
    /// [Pick one] Configures a load job.
    pub load: Option<JobConfigurationLoad>,
    /// [Pick one] Copies a table.
    pub copy: Option<JobConfigurationTableCopy>,
    /// [Pick one] Configures an extract job.
    pub extract: Option<JobConfigurationTableExtract>,
    /// Optional. If set, don't actually run this job.
    /// A valid query will return a mostly empty response with some processing statistics,
    /// while an invalid query will return the same error it would if it wasn't a dry run.
    /// Behavior of non-query jobs is undefined.
    pub dry_run: Option<bool>,
    /// Optional. Job timeout in milliseconds.
    /// If this time limit is exceeded, BigQuery might attempt to stop the job.
    #[serde(deserialize_with = "crate::http::from_str_option")]
    #[serde(default)]
    pub job_timeout_ms: Option<i64>,
    /// The labels associated with this job.
    /// You can use these to organize and group your jobs.
    /// Label keys and values can be no longer than 63 characters, can only contain lowercase letters, numeric characters, underscores and dashes. International characters are allowed. Label values are optional. Label keys must start with a letter and each label in the list must have a different key.
    /// An object containing a list of "key": value pairs. Example: { "name": "wrench", "mass": "1.3kg", "count": "3" }.
    pub labels: Option<HashMap<String, String>>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct JobReference {
    /// Required. The ID of the project containing this job.
    pub project_id: String,
    /// Required. The ID of the job. The ID must contain only letters (a-z, A-Z), numbers (0-9), underscores (_), or dashes (-). The maximum length is 1,024 characters.
    pub job_id: String,
    /// Optional. The geographic location of the job. The default value is US.
    pub location: Option<String>
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    /// Output only. The resource type.
    pub kind: String,
    /// Output only. A hash of the resource.
    pub etag: String,
    /// Output only. Opaque ID field of the job.
    pub id: String,
    /// Output only. A URL that can be used to access the resource again.
    pub self_link: String,
    /// Output only. Email address of the user who ran the job.
    pub user_email: String,
    /// Required. Describes the job configuration.
    pub configuration: JobConfiguration,
    /// Optional. Reference describing the unique-per-user name of the job.
    pub job_reference: Option<JobReference>,
    /// Output only. Information about the job, including starting time and ending time of the job.
    pub statistics: JobStatistics,
    /// Output only. The status of this job. Examine this value when polling an asynchronous job to see if the job is complete.
    pub job_status: JobStatus
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct JobStatistics {
    /// Output only. Creation time of this job, in milliseconds since the epoch.
    /// This field will be present on all jobs.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub creation_time: i64,
    /// Output only. Start time of this job, in milliseconds since the epoch.
    /// This field will be present when the job transitions from the PENDING state to either RUNNING or DONE.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub start_time: i64,
    /// Output only. End time of this job, in milliseconds since the epoch.
    /// This field will be present whenever a job is in the DONE state.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub end_time: i64,
    /// Output only. Total bytes processed for the job.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub total_bytes_processed: i64,
    /// Output only. [TrustedTester] Job progress (0.0 -> 1.0) for LOAD and EXTRACT jobs.
    pub completion_ratio: f32,
    /// Output only. Quotas which delayed this job's start time.
    pub quota_deferments: Vec<String>,
    /// Output only. Statistics for a query job.
    pub query: Option<JobStatisticsQuery>,
    /// Output only. Statistics for a load job.
    pub load: Option<JobStatisticsLoad>,
    /// Output only. Statistics for an extract job.
    pub extract: Option<JobStatisticsLoad>,
    /// Output only. Slot-milliseconds for the job.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub total_slot_ms: i64,
    /// Output only. Name of the primary reservation assigned to this job.
    /// Note that this could be different than reservations reported in the reservation usage field if parent reservations were used to execute this job.
    pub reservation_id: String,
    /// Output only. Number of child jobs executed.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub num_child_jobs: i64,
    /// Output only. If this is a child job, specifies the job ID of the parent.
    pub parent_job_id: String,
    /// Output only. If this a child job of a script, specifies information about the context of this job within the script.
    pub script_statistics: ScriptStatistics,
    /// Output only. Statistics for row-level security. Present only for query and extract jobs.
    pub row_level_security_statistics: RowLevelSecurityStatistics ,
    /// Output only. Statistics for data-masking. Present only for query and extract jobs.
    pub data_masking_statistics: DataMaskingStatistics,
    /// Output only. [Alpha] Information of the multi-statement transaction if this job is part of one.
    /// This property is only expected on a child job or a job that is in a session. A script parent job is not part of the transaction started in the script.
    pub transaction_info: TransactionInfo,
    /// Output only. Information of the session if this job is part of one.
    pub session_info: SessionInfo,
    /// Output only. The duration in milliseconds of the execution of the final attempt of this job,
    /// as BigQuery may internally re-attempt to execute the job.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub final_execution_duration_ms: i64
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct JobStatisticsQuery {
    /// Output only. Describes execution plan for the query.
    pub query_plan: Vec<ExpalinQueryStage>,
    /// Output only. The original estimate of bytes processed for the job.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub estimated_bytes_processed:  i64,
    /// Output only. Describes a timeline of job execution.
    pub timeline: Vec<QueryTimelineSample>,
    /// Output only. Total number of partitions processed from all partitioned tables referenced in the job.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub total_partitions_processed: i64,
    /// Output only. Total bytes processed for the job.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub total_bytes_processed: i64,
    /// Output only. For dry-run jobs, totalBytesProcessed is an estimate and
    /// this field specifies the accuracy of the estimate. Possible values can be:
    /// UNKNOWN: accuracy of the estimate is unknown.
    /// PRECISE: estimate is precise.
    /// LOWER_BOUND: estimate is lower bound of what the query would cost.
    /// UPPER_BOUND: estimate is upper bound of what the query would cost.
    pub total_bytes_processed_accuracy: String,
    /// Output only. If the project is configured to use on-demand pricing,
    /// then this field contains the total bytes billed for the job. If the project is configured to use flat-rate pricing,
    /// then you are not billed for bytes and this field is informational only.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub total_bytes_billed: i64,
    /// Output only. Billing tier for the job.
    /// This is a BigQuery-specific concept which is not related to the GCP notion of "free tier".
    /// The value here is a measure of the query's resource consumption relative to the amount of data scanned. For on-demand queries, the limit is 100, and all queries within this limit are billed at the standard on-demand rates. On-demand queries that exceed this limit will fail with a billingTierLimitExceeded error.
    pub billing_tier: i32,
    /// Output only. Slot-milliseconds for the job.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub total_slot_ms: i64,
    /// Output only. Whether the query result was fetched from the query cache.
    pub cache_hist: bool,
    /// Output only. Referenced tables for the job. Queries that reference more than 50 tables will not have a complete list.
    pub referenced_tables: Vec<TableReference>,
    /// Output only. Referenced routines for the job.
    pub referenced_routines: Vec<RoutineReference>,
    /// Output only. The schema of the results. Present only for successful dry run of non-legacy SQL queries.
    pub schema: Option<TableSchema>,
    /// Output only. The number of rows affected by a DML statement.
    /// Present only for DML statements INSERT, UPDATE or DELETE.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub num_dml_affected_rows: i64,
    /// Output only. Detailed statistics for DML statements INSERT, UPDATE, DELETE, MERGE or TRUNCATE
    pub dml_stats: DmlStats,
    /// Output only. GoogleSQL only: list of undeclared query parameters detected during a dry run validation
    pub undeclared_query_parameters: Vec<QueryParameter>,
    /// Output only. The type of query statement, if valid. Possible values:
    /// SELECT: SELECT statement.
    /// INSERT: INSERT statement.
    /// UPDATE: UPDATE statement.
    /// DELETE: DELETE statement.
    /// MERGE: MERGE statement.
    /// ALTER_TABLE: ALTER TABLE statement.
    /// ALTER_VIEW: ALTER VIEW statement.
    /// ASSERT: ASSERT statement.
    /// CREATE_FUNCTION: CREATE FUNCTION statement.
    /// CREATE_MODEL: CREATE MODEL statement.
    /// CREATE_PROCEDURE: CREATE PROCEDURE statement.
    /// CREATE_ROW_ACCESS_POLICY: CREATE ROW ACCESS POLICY statement.
    /// CREATE_TABLE: CREATE TABLE statement, without AS SELECT.
    /// CREATE_TABLE_AS_SELECT: CREATE TABLE AS SELECT statement.
    /// CREATE_VIEW: CREATE VIEW statement.
    /// DROP_FUNCTION : DROP FUNCTION statement.
    /// DROP_PROCEDURE: DROP PROCEDURE statement.
    /// DROP_ROW_ACCESS_POLICY: DROP [ALL] ROW ACCESS POLICY|POLICIES statement.
    /// DROP_TABLE: DROP TABLE statement.
    /// DROP_VIEW: DROP VIEW statement.
    /// EXPORT_MODEL: EXPORT MODEL statement.
    /// LOAD_DATA: LOAD DATA statement.
    pub statement_type: String,
    /// Output only. The DDL operation performed, possibly dependent on the pre-existence of the DDL target.
    pub ddl_operation_performed: String,
    /// Output only. The DDL target table. Present only for CREATE/DROP TABLE/VIEW and DROP ALL ROW ACCESS POLICIES queries.
    pub ddl_target_table: Option<TableReference>,
    /// Output only. The DDL target row access policy. Present only for CREATE/DROP ROW ACCESS POLICY queries.
    pub ddl_target_row_access_policy: Option<RowAccessPolicyReference>,
    /// Output only. The number of row access policies affected by a DDL statement.
    /// Present only for DROP ALL ROW ACCESS POLICIES queries.
    #[serde(deserialize_with = "crate::http::from_str_option")]
    #[serde(default)]
    pub ddl_affected_row_access_policy_count: Option<i64>,
    /// Output only. [Beta] The DDL target routine. Present only for CREATE/DROP FUNCTION/PROCEDURE queries.
    pub ddl_target_routine: Option<RoutineReference>,
    /// Output only. Statistics of a BigQuery ML training job.
    pub ml_statistics: Option<MlStatistics>,
    /// Output only. Stats for EXPORT DATA statement.
    pub export_data_statistics: Option<ExportDataStatistics>,
    /// Output only. Job cost breakdown as bigquery internal cost and external service costs.
    pub external_service_costs: Option<Vec<ExternalServiceCost>>,
    /// Output only. BI Engine specific Statistics.
    pub bi_engine_statistics: Option<BiEngineStatistics>,
    /// Output only. Statistics for a LOAD query.
    pub load_query_statistics: Option<LoadQueryStatistics>,
    /// Output only. Referenced table for DCL statement.
    pub dcl_target_table: Option<TableReference> ,
    /// Output only. Referenced view for DCL statement.
    pub dcl_target_view: Option<TableReference>,
    /// Output only. Search query specific statistics.
    pub search_statistics: SearchStatistics,
    /// Output only. Performance insights.
    pub performance_insights: PeformanceInsights,
    /// Output only. Statistics of a Spark procedure job.
    pub spark_statistics: Option<SparkStatistics>,
    /// Output only. Total bytes transferred for cross-cloud queries such as Cross Cloud Transfer and CREATE TABLE AS SELECT (CTAS).
    #[serde(deserialize_with = "crate::http::from_str_option")]
    #[serde(default)]
    pub transferred_bytes: Option<i64>
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExplainStageQuery {
    /// Human-readable name for the stage.
    pub name: String,
    /// Unique ID for the stage within the plan.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub id: i64,
    /// Stage start time represented as milliseconds since the epoch
    #[serde(deserialize_with = "crate::http::from_str")]
    pub start_ms: i64,
    /// Stage end time represented as milliseconds since the epoch.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub end_ms: i64,
    /// IDs for stages that are inputs to this stage.
    #[serde(deserialize_with = "crate::http::from_str_vec")]
    pub input_stages: Vec<i64>,
    /// Relative amount of time the average shard spent waiting to be scheduled.
    pub wait_ratio_avg: f64,
    /// Milliseconds the average shard spent waiting to be scheduled.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub wait_ms_avg: i64,
    /// Relative amount of time the slowest shard spent waiting to be scheduled.
    pub wait_ratio_max: f64,
    ///Milliseconds the slowest shard spent reading input.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub wait_ms_max: i64,
    /// Relative amount of time the average shard spent on CPU-bound tasks.
    pub compute_ratio_avg: f64,
    /// Milliseconds the average shard spent on CPU-bound tasks.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub compute_ms_avg: i64,
    /// Relative amount of time the slowest shard spent on CPU-bound tasks.
    pub compute_ratio_max: f64,
    /// Milliseconds the slowest shard spent on CPU-bound tasks.
    pub compute_ms_max: i64,
    /// Relative amount of time the average shard spent on writing output.
    pub write_ratio_avg: f64,
    /// Milliseconds the average shard spent on writing output.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub write_ms_avg: i64,
    /// Relative amount of time the slowest shard spent on writing output.
    pub write_ratio_max: f64,
    /// Milliseconds the slowest shard spent on writing output.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub write_ms_max: i64,
    /// Total number of bytes written to shuffle.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub shuffle_output_bytes: i64,
    /// Total number of bytes written to shuffle.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub shuffle_output_bytes_spilled: i64,
    /// Number of records read into the stage.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub records_read: i64,
    /// Number of records read written by the stage.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub records_written: i64,
    /// Number of parallel input segments to be processed
    #[serde(deserialize_with = "crate::http::from_str")]
    pub parallel_inputs: i64,
    /// Number of parallel input segments completed.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub completed_parallel_inputs: i64,
    /// Current status for this stage.
    pub status: String,
    /// List of operations within the stage in dependency order (approximately chronological).
    pub steps: Vec<ExplainQueryStep>,
    /// Slot-milliseconds used by the stage
    #[serde(deserialize_with = "crate::http::from_str")]
    pub slot_ms: i64
}
