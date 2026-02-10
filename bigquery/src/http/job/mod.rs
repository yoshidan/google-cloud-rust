use std::collections::HashMap;

use time::OffsetDateTime;

use crate::http::dataset::DatasetReference;
use crate::http::model::{HparamTuningTrial, IterationResult, ModelReference, ModelType};
use crate::http::routine::RoutineReference;
use crate::http::row_access_policy::RowAccessPolicyReference;
use crate::http::table::{
    Clustering, DecimalTargetType, DestinationFormat, ExternalDataConfiguration, HivePartitioningOptions,
    ParquetOptions, RangePartitioning, SourceFormat, TableReference, TableSchema, TimePartitioning,
    UserDefinedFunctionResource,
};
use crate::http::types::{ConnectionProperty, EncryptionConfiguration, ErrorProto, QueryParameter};

pub mod cancel;
pub mod delete;
pub mod get;
pub mod get_query_results;
pub mod insert;
pub mod list;
pub mod query;

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KeyResultStatementKind {
    #[default]
    Last,
    FirstSelect,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScriptOptions {
    /// Timeout period for each statement in a script.
    #[serde(deserialize_with = "crate::http::from_str_option")]
    #[serde(default)]
    pub statement_timeout_ms: Option<i64>,
    /// Limit on the number of bytes billed per statement. Exceeding this budget results in an error.
    #[serde(deserialize_with = "crate::http::from_str_option")]
    #[serde(default)]
    pub statement_byte_budget: Option<i64>,
    /// Determines which statement in the script represents the "key result",
    /// used to populate the schema and query results of the script job. Default is LAST.
    pub key_result_statement: Option<KeyResultStatementKind>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CreateDisposition {
    /// If the table does not exist, BigQuery creates the table.
    #[default]
    CreateIfNeeded,
    /// The table must already exist. If it does not, a 'notFound' error is returned in the job result.
    CreateNever,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WriteDisposition {
    /// If the table already exists, BigQuery overwrites the table data and uses the schema from the query result..
    WriteTruncate,
    /// If the table already exists, BigQuery appends the data to the table..
    WriteAppend,
    /// If the table already exists and contains data, a 'duplicate' error is returned in the job result.
    #[default]
    WriteEmpty,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Priority {
    #[default]
    Interactive,
    Batch,
}
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SchemaUpdateOption {
    /// allow adding a nullable field to the schema.
    AllowFieldAddition,
    /// allow relaxing a required field in the original schema to nullable.
    AllowFieldRelaxation,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct JobConfigurationLoad {
    /// [Required] The fully-qualified URIs that point to your data in Google Cloud.
    /// For Google Cloud Storage URIs: Each URI can contain one '*' wildcard character and it must come after the 'bucket' name. Size limits related to load jobs apply to external data sources. For Google Cloud Bigtable URIs: Exactly one URI can be specified and it has be a fully specified and valid HTTPS URL for a Google Cloud Bigtable table. For Google Cloud Datastore backups: Exactly one URI can be specified. Also, the '*' wildcard character is not allowed.
    pub source_uris: Vec<String>,
    /// Optional. The schema for the destination table. The schema can be omitted if the destination table already exists, or if you're loading data from Google Cloud Datastore.
    pub schema: Option<TableSchema>,
    /// [Required] The destination table to load the data into.
    pub destination_table: TableReference,
    /// Optional. [Experimental] Properties with which to create the destination table if it is new.
    pub destination_table_properties: Option<DestinationTableProperties>,
    /// Optional. Specifies whether the job is allowed to create new tables. The following values are supported:
    /// CREATE_IF_NEEDED: If the table does not exist, BigQuery creates the table.
    /// CREATE_NEVER: The table must already exist. If it does not, a 'notFound' error is returned in the job result. The default value is CREATE_IF_NEEDED. Creation, truncation and append actions occur as one atomic update upon job completion.
    pub create_disposition: Option<CreateDisposition>,
    /// Optional. Specifies the action that occurs if the destination table already exists. The following values are supported:
    /// WRITE_TRUNCATE: If the table already exists, BigQuery overwrites the table data and uses the schema from the load.
    /// WRITE_APPEND: If the table already exists, BigQuery appends the data to the table.
    /// WRITE_EMPTY: If the table already exists and contains data, a 'duplicate' error is returned in the job result.
    /// The default value is WRITE_APPEND. Each action is atomic and only occurs if BigQuery is able to complete the job successfully. Creation, truncation and append actions occur as one atomic update upon job completion.
    pub write_disposition: Option<WriteDisposition>,
    /// Optional. Specifies a string that represents a null value in a CSV file.
    /// For example, if you specify "\N", BigQuery interprets "\N" as a null value when loading a CSV file. The default value is the empty string. If you set this property to a custom value, BigQuery throws an error if an empty string is present for all data types except for STRING and BYTE. For STRING and BYTE columns, BigQuery interprets the empty string as an empty value.
    pub null_marker: Option<String>,
    /// Optional. The separator character for fields in a CSV file.
    /// The separator is interpreted as a single byte. For files encoded in ISO-8859-1, any single character can be used as a separator. For files encoded in UTF-8, characters represented in decimal range 1-127 (U+0001-U+007F) can be used without any modification. UTF-8 characters encoded with multiple bytes (i.e. U+0080 and above) will have only the first byte used for separating fields. The remaining bytes will be treated as a part of the field. BigQuery also supports the escape sequence "\t" (U+0009) to specify a tab separator. The default value is comma (",", U+002C).
    pub field_delimiter: Option<String>,
    /// Optional. The number of rows at the top of a CSV file that BigQuery will skip when loading the data. The default value is 0. This property is useful if you have header rows in the file that should be skipped. When autodetect is on, the behavior is the following:
    /// skipLeadingRows unspecified - Autodetect tries to detect headers in the first row. If they are not detected, the row is read as data. Otherwise data is read starting from the second row.
    /// skipLeadingRows is 0 - Instructs autodetect that there are no headers and data should be read starting from the first row.
    /// skipLeadingRows = N > 0 - Autodetect skips N-1 rows and tries to detect headers in row N. If headers are not detected, row N is just skipped. Otherwise row N is used to extract column names for the detected schema.
    pub skip_leading_rows: Option<i64>,
    /// Optional. The character encoding of the data.
    /// The supported values are UTF-8, ISO-8859-1, UTF-16BE, UTF-16LE, UTF-32BE, and UTF-32LE. The default value is UTF-8. BigQuery decodes the data after the raw, binary data has been split using the values of the quote and fieldDelimiter properties.
    /// If you don't specify an encoding, or if you specify a UTF-8 encoding when the CSV file is not UTF-8 encoded, BigQuery attempts to convert the data to UTF-8. Generally, your data loads successfully, but it may not match byte-for-byte what you expect. To avoid this, specify the correct encoding by using the --encoding flag.
    /// If BigQuery can't convert a character other than the ASCII 0 character, BigQuery converts the character to the standard Unicode replacement character: �.
    pub encoding: Option<String>,
    /// Optional. The value that is used to quote data sections in a CSV file. BigQuery converts the string to ISO-8859-1 encoding, and then uses the first byte of the encoded string to split the data in its raw, binary state. The default value is a double-quote ('"'). If your data does not contain quoted sections, set the property value to an empty string. If your data contains quoted newline characters, you must also set the allowQuotedNewlines property to true. To include the specific quote character within a quoted value, precede it with an additional matching quote character. For example, if you want to escape the default character ' " ', use ' "" '. @default "
    pub quote: Option<String>,
    /// Optional. The maximum number of bad records that BigQuery can ignore when running the job.
    /// If the number of bad records exceeds this value, an invalid error is returned in the job result. The default value is 0, which requires that all records are valid. This is only supported for CSV and NEWLINE_DELIMITED_JSON file formats.
    pub max_bad_records: Option<i64>,
    /// Indicates if BigQuery should allow quoted data sections that contain newline characters in a CSV file.
    /// The default value is false.
    pub allow_quoted_newlines: Option<bool>,
    /// Optional. The format of the data files.
    /// For CSV files, specify "CSV".
    /// For datastore backups, specify "DATASTORE_BACKUP".
    /// For newline-delimited JSON, specify "NEWLINE_DELIMITED_JSON".
    /// For Avro, specify "AVRO".
    /// For parquet, specify "PARQUET".
    /// For orc, specify "ORC".
    /// The default value is CSV.
    pub source_format: Option<SourceFormat>,
    /// Optional. Accept rows that are missing trailing optional columns.
    /// The missing values are treated as nulls.
    /// If false, records with missing trailing columns are treated as bad records,
    /// and if there are too many bad records, an invalid error is returned in the job result.
    /// The default value is false.
    /// Only applicable to CSV, ignored for other formats.
    pub allow_jagged_rows: Option<bool>,
    /// Optional. Indicates if BigQuery should allow extra values that are not represented in the table schema.
    /// If true, the extra values are ignored.
    /// If false, records with extra columns are treated as bad records, and if there are too many bad records,
    /// an invalid error is returned in the job result.
    /// The default value is false.
    /// The sourceFormat property determines what BigQuery treats as an extra value:
    /// CSV: Trailing columns JSON: Named values that don't match any column names in the
    /// table schema Avro, Parquet, ORC: Fields in the file schema that don't exist in the table schema.
    pub ignore_unknown_values: Option<bool>,
    /// If sourceFormat is set to "DATASTORE_BACKUP",
    /// indicates which entity properties to load into BigQuery from a Cloud Datastore backup.
    /// Property names are case sensitive and must be top-level properties.
    /// If no properties are specified, BigQuery loads all properties.
    /// If any named property isn't found in the Cloud Datastore backup,
    /// an invalid error is returned in the job result.
    pub projection_fields: Option<Vec<String>>,
    /// Optional. Indicates if we should automatically infer the options and schema for CSV and JSON sources.
    pub autodetect: Option<bool>,
    /// Allows the schema of the destination table to be updated as a side effect of
    /// the load job if a schema is autodetected or supplied in the job configuration.
    /// Schema update options are supported in two cases:
    /// when writeDisposition is WRITE_APPEND;
    /// when writeDisposition is WRITE_TRUNCATE
    /// and the destination table is a partition of a table,
    /// specified by partition decorators. For normal tables, WRITE_TRUNCATE will always overwrite the schema.
    /// One or more of the following values are specified:
    /// ALLOW_FIELD_ADDITION: allow adding a nullable field to the schema.
    /// ALLOW_FIELD_RELAXATION: allow relaxing a required field in the original schema to nullable.
    pub schema_update_options: Option<Vec<SchemaUpdateOption>>,
    /// Time-based partitioning specification for the destination table.
    /// Only one of timePartitioning and rangePartitioning should be specified.
    pub time_partitioning: Option<TimePartitioning>,
    /// Range partitioning specification for the destination table.
    /// Only one of timePartitioning and rangePartitioning should be specified.
    pub range_partitioning: Option<RangePartitioning>,
    /// Clustering specification for the destination table.
    pub clustering: Option<Clustering>,
    /// Custom encryption configuration (e.g., Cloud KMS keys)
    pub destination_encryption_configuration: Option<EncryptionConfiguration>,
    /// Optional. If sourceFormat is set to "AVRO", indicates whether to interpret logical types as the corresponding BigQuery data type (for example, TIMESTAMP), instead of using the raw type (for example, INTEGER).
    pub use_avro_logical_types: Option<bool>,
    /// Optional. The user can provide a reference file with the reader schema. This file is only loaded if it is part of source URIs, but is not loaded otherwise. It is enabled for the following formats: AVRO, PARQUET, ORC.
    pub reference_file_schema_uri: Option<String>,
    /// Optional. When set, configures hive partitioning support.
    /// Not all storage formats support hive partitioning -- requesting hive partitioning on an unsupported format will lead to an error, as will providing an invalid specification.
    pub hive_partitioning_options: Option<HivePartitioningOptions>,
    /// Defines the list of possible SQL data types to which the source decimal values are converted.
    /// This list and the precision and the scale parameters of the decimal field determine the target type.
    /// In the order of NUMERIC, BIGNUMERIC, and STRING,
    /// a type is picked if it is in the specified list and if it supports the precision and the scale. STRING supports all precision and scale values. If none of the listed types supports the precision and the scale, the type supporting the widest range in the specified list is picked, and if a value exceeds the supported range when reading the data, an error will be thrown.
    /// Example: Suppose the value of this field is ["NUMERIC", "BIGNUMERIC"]. If (precision,scale) is:
    /// - (38,9)  NUMERIC;
    /// - (39,9)  BIGNUMERIC (NUMERIC cannot hold 30 integer digits);
    /// - (38,10)  BIGNUMERIC (NUMERIC cannot hold 10 fractional digits);
    /// - (76,38)  BIGNUMERIC;
    /// - (77,38)  BIGNUMERIC (error if value exeeds supported range).
    ///   This field cannot contain duplicate types. The order of the types in this field is ignored. For example, ["BIGNUMERIC", "NUMERIC"] is the same as ["NUMERIC", "BIGNUMERIC"] and NUMERIC always takes precedence over BIGNUMERIC.
    ///
    /// Defaults to ["NUMERIC", "STRING"] for ORC and ["NUMERIC"] for the other file formats.
    pub decimal_target_types: Option<Vec<DecimalTargetType>>,
    /// Optional. Additional properties to set if sourceFormat is set to PARQUET.
    pub parquet_options: Option<ParquetOptions>,
    /// Optional. When sourceFormat is set to "CSV", this indicates whether the embedded ASCII control characters (the first 32 characters in the ASCII-table, from '\x00' to '\x1F') are preserved.
    pub preserve_ascii_control_characters: Option<bool>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum JobConfigurationSourceTable {
    SourceTable(TableReference),
    SourceTables(Vec<TableReference>),
}

impl Default for JobConfigurationSourceTable {
    fn default() -> Self {
        Self::SourceTable(TableReference::default())
    }
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OperationType {
    #[default]
    OperationTypeUnspecified,
    Copy,
    Snapshot,
    Restore,
    Clone,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct JobConfigurationTableCopy {
    #[serde(flatten)]
    pub source_table: JobConfigurationSourceTable,
    pub destination_table: TableReference,
    /// Optional. Specifies whether the job is allowed to create new tables. The following values are supported:
    /// CREATE_IF_NEEDED: If the table does not exist, BigQuery creates the table.
    /// CREATE_NEVER: The table must already exist. If it does not, a 'notFound' error is returned in the job result.
    /// The default value is CREATE_IF_NEEDED. Creation, truncation and append actions occur as one atomic update upon job completion.
    pub create_disposition: Option<CreateDisposition>,
    /// Optional. Specifies the action that occurs if the destination table already exists. The following values are supported:
    /// WRITE_TRUNCATE: If the table already exists, BigQuery overwrites the table data and uses the schema from the source table.
    /// WRITE_APPEND: If the table already exists, BigQuery appends the data to the table.
    /// WRITE_EMPTY: If the table already exists and contains data, a 'duplicate' error is returned in the job result.
    /// The default value is WRITE_EMPTY. Each action is atomic and only occurs if BigQuery is able to complete the job successfully. Creation, truncation and append actions occur as one atomic update upon job completion.
    pub write_disposition: Option<WriteDisposition>,
    /// Custom encryption configuration (e.g., Cloud KMS keys).
    pub destination_encryption_configuration: Option<EncryptionConfiguration>,
    /// Optional. Supported operation types in table copy job.
    pub operation_type: Option<OperationType>,
    /// Optional. The time when the destination table expires.
    /// Expired tables will be deleted and their storage reclaimed.
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub destination_expiration_time: Option<OffsetDateTime>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct DestinationTableProperties {
    /// Optional. Friendly name for the destination table.
    /// If the table already exists, it should be same as the existing friendly name.
    pub friendly_name: Option<String>,
    /// Optional. The description for the destination table.
    /// This will only be used if the destination table is newly created.
    /// If the table already exists and a value different than the current description is provided, the job will fail.
    pub description: Option<String>,
    /// Optional. The labels associated with this table.
    /// You can use these to organize and group your tables.
    /// This will only be used if the destination table is newly created.
    /// If the table already exists and labels are different than the current labels are provided, the job will fail.
    /// An object containing a list of "key": value pairs.
    /// Example: { "name": "wrench", "mass": "1.3kg", "count": "3" }.
    pub labels: Option<HashMap<String, String>>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum JobConfigurationExtractSource {
    SourceTable(TableReference),
    SourceModel(ModelReference),
}

impl Default for JobConfigurationExtractSource {
    fn default() -> Self {
        Self::SourceTable(TableReference::default())
    }
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModelExtractOptions {
    /// The 1-based ID of the trial to be exported from a hyperparameter tuning model.
    /// If not specified, the trial with id = Model.defaultTrialId is exported.
    /// This field is ignored for models not trained with hyperparameter tuning.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub trial_id: i64,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct JobConfigurationExtract {
    /// A list of fully-qualified Google Cloud Storage URIs where the extracted table should be written.
    pub destination_uris: Vec<String>,
    /// Optional. Whether to print out a header row in the results. Default is true. Not applicable when extracting models.
    pub print_header: Option<bool>,
    /// Optional. When extracting data in CSV format, this defines the delimiter to use between fields in the exported data. Default is ','. Not applicable when extracting models.
    pub field_delimiter: Option<String>,
    /// Optional. The exported file format.
    /// Possible values include CSV, NEWLINE_DELIMITED_JSON, PARQUET, or AVRO for tables and ML_TF_SAVED_MODEL or ML_XGBOOST_BOOSTER for models. The default value for tables is CSV. Tables with nested or repeated fields cannot be exported as CSV. The default value for models is ML_TF_SAVED_MODEL.
    pub destination_format: Option<DestinationFormat>,
    /// Optional. The compression type to use for exported files. Possible values include DEFLATE, GZIP, NONE, SNAPPY, and ZSTD. The default value is NONE. Not all compression formats are support for all file formats. DEFLATE is only supported for Avro. ZSTD is only supported for Parquet. Not applicable when extracting models.
    pub compression: Option<String>,
    /// Whether to use logical types when extracting to AVRO format. Not applicable when extracting models.
    pub use_avro_logical_types: Option<bool>,
    /// Optional. Model extract options only applicable when extracting models.
    pub model_extract_options: Option<ModelExtractOptions>,
    #[serde(flatten)]
    pub source: JobConfigurationExtractSource,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
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
    pub table_definitions: Option<HashMap<String, ExternalDataConfiguration>>,
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
    pub priority: Option<Priority>,
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
    pub query_parameters: Option<Vec<QueryParameter>>,
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
    pub destination_encryption_configuration: Option<EncryptionConfiguration>,
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
    pub create_session: Option<bool>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum JobType {
    Query(JobConfigurationQuery),
    Load(JobConfigurationLoad),
    Copy(JobConfigurationTableCopy),
    Extract(JobConfigurationExtract),
}

impl Default for JobType {
    fn default() -> Self {
        Self::Query(JobConfigurationQuery::default())
    }
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct JobConfiguration {
    /// Output only. The type of the job. Can be QUERY, LOAD, EXTRACT, COPY or UNKNOWN.
    pub job_type: String,
    /// [Pick one] Configures a job.
    #[serde(flatten)]
    pub job: JobType,
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

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct JobReference {
    /// Required. The ID of the project containing this job.
    pub project_id: String,
    /// Required. The ID of the job. The ID must contain only letters (a-z, A-Z), numbers (0-9), underscores (_), or dashes (-). The maximum length is 1,024 characters.
    /// Not found when the job of query is dry run.
    #[serde(default)]
    pub job_id: String,
    /// Optional. The geographic location of the job. The default value is US.
    pub location: Option<String>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
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
    #[serde(rename(deserialize = "user_email"))]
    pub user_email: Option<String>,
    /// Required. Describes the job configuration.
    pub configuration: JobConfiguration,
    /// Reference describing the unique-per-user name of the job.
    pub job_reference: JobReference,
    /// Output only. Information about the job, including starting time and ending time of the job.
    pub statistics: Option<JobStatistics>,
    /// Output only. The status of this job. Examine this value when polling an asynchronous job to see if the job is complete.
    pub status: JobStatus,
}

impl Job {}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JobState {
    #[default]
    Done,
    Pending,
    Running,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct JobStatus {
    /// Output only. Final error result of the job.
    /// If present, indicates that the job has completed and was unsuccessful.
    pub error_result: Option<ErrorProto>,
    /// Output only. The first errors encountered during the running of the job.
    /// The final message includes the number of errors that caused the process to stop.
    /// Errors here do not necessarily mean that the job has not completed or was unsuccessful.
    pub errors: Option<Vec<ErrorProto>>,
    /// Output only. Running state of the job. Valid states include 'PENDING', 'RUNNING', and 'DONE'.
    pub state: JobState,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct JobStatistics {
    /// Output only. Creation time of this job, in milliseconds since the epoch.
    /// This field will be present on all jobs.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub creation_time: i64,
    /// Output only. Start time of this job, in milliseconds since the epoch.
    /// This field will be present when the job transitions from the PENDING state to either RUNNING or DONE.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub start_time: Option<i64>,
    /// Output only. End time of this job, in milliseconds since the epoch.
    /// This field will be present whenever a job is in the DONE state.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub end_time: Option<i64>,
    /// Output only. Total bytes processed for the job.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub total_bytes_processed: Option<i64>,
    /// Output only. [TrustedTester] Job progress (0.0 -> 1.0) for LOAD and EXTRACT jobs.
    pub completion_ratio: Option<f32>,
    /// Output only. Quotas which delayed this job's start time.
    pub quota_deferments: Option<Vec<String>>,
    /// Output only. Statistics for a query job.
    pub query: Option<JobStatisticsQuery>,
    /// Output only. Statistics for a load job.
    pub load: Option<JobStatisticsLoad>,
    /// Output only. Statistics for an extract job.
    pub extract: Option<JobStatisticsExtract>,
    /// Output only. Slot-milliseconds for the job.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub total_slot_ms: Option<i64>,
    /// Output only. Name of the primary reservation assigned to this job.
    /// Note that this could be different than reservations reported in the reservation usage field if parent reservations were used to execute this job.
    pub reservation_id: Option<String>,
    /// Output only. Number of child jobs executed.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub num_child_jobs: Option<i64>,
    /// Output only. If this is a child job, specifies the job ID of the parent.
    pub parent_job_id: Option<String>,
    /// Output only. If this a child job of a script, specifies information about the context of this job within the script.
    pub script_statistics: Option<ScriptStatistics>,
    /// Output only. Statistics for row-level security. Present only for query and extract jobs.
    pub row_level_security_statistics: Option<RowLevelSecurityStatistics>,
    /// Output only. Statistics for data-masking. Present only for query and extract jobs.
    pub data_masking_statistics: Option<DataMaskingStatistics>,
    /// Output only. [Alpha] Information of the multi-statement transaction if this job is part of one.
    /// This property is only expected on a child job or a job that is in a session. A script parent job is not part of the transaction started in the script.
    pub transaction_info: Option<TransactionInfo>,
    /// Output only. Information of the session if this job is part of one.
    pub session_info: Option<SessionInfo>,
    /// Output only. The duration in milliseconds of the execution of the final attempt of this job,
    /// as BigQuery may internally re-attempt to execute the job.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub final_execution_duration_ms: Option<i64>,
}
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    /// Output only. The id of the session.
    pub session_id: Option<String>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct JobStatisticsLoad {
    /// Output only. Number of source files in a load job.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub input_files: Option<i64>,
    /// Output only. Number of bytes of source data in a load job.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub input_file_bytes: Option<i64>,
    /// Output only. Number of rows imported in a load job.
    /// Note that while an import job is in the running state, this value may change.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub output_rows: Option<i64>,
    /// Output only. Size of the loaded data in bytes.
    /// Note that while a load job is in the running state, this value may change.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub output_bytes: Option<i64>,
    /// Output only. The number of bad records encountered. Note that if the job has failed because of more bad records encountered than the maximum allowed in the load job configuration, then this number can be less than the total number of bad records present in the input data.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub bad_records: Option<i64>,
    /// Output only. Describes a timeline of job execution.
    pub timeline: Option<Vec<QueryTimelineSample>>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct JobStatisticsExtract {
    /// Output only. Number of files per destination URI or URI pattern specified in the extract configuration. These values will be in the same order as the URIs specified in the 'destinationUris' field.
    #[serde(default, deserialize_with = "crate::http::from_str_vec_option")]
    pub destination_uri_file_counts: Option<Vec<i64>>,
    /// Output only. Number of user bytes extracted into the result.
    /// This is the byte count as computed by BigQuery for billing purposes and doesn't have any relationship with the number of actual result bytes extracted in the desired format.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub input_bytes: Option<i64>,
    /// Output only. Describes a timeline of job execution.
    pub timeline: Option<Vec<QueryTimelineSample>>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub enum EvaluationKind {
    #[default]
    EvaluationKindUnspecified,
    Statement,
    Expression,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScriptStackFrame {
    /// Output only. One-based start line.
    pub start_line: Option<i64>,
    /// Output only. One-based start column.
    pub start_column: Option<i64>,
    /// Output only. One-based end line.
    pub end_line: Option<i64>,
    /// Output only. One-based end column.
    pub end_column: Option<i64>,
    /// Output only. Name of the active procedure, empty if in a top-level script.
    pub procedure_id: Option<String>,
    /// Output only. Text of the current statement/expression.
    pub text: Option<String>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RowLevelSecurityStatistics {
    /// Whether any accessed data was protected by row access policies.
    pub row_level_security_applied: Option<bool>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct DataMaskingStatistics {
    /// Whether any accessed data was protected by the data masking.
    pub data_masking_applied: Option<bool>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct TransactionInfo {
    /// Output only. [Alpha] Id of the transaction..
    pub transaction_id: Option<String>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScriptStatistics {
    /// Whether this child job was a statement or expression.
    pub evaluation_kind: Option<EvaluationKind>,
    /// Stack trace showing the line/column/procedure name of each frame on the stack at the point where the current evaluation happened. The leaf frame is first, the primary script is last. Never empty.
    pub stack_frames: Option<Vec<ScriptStackFrame>>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct JobStatisticsQuery {
    /// Output only. Describes execution plan for the query.
    pub query_plan: Option<Vec<ExplainQueryStage>>,
    /// Output only. The original estimate of bytes processed for the job.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub estimated_bytes_processed: Option<i64>,
    /// Output only. Describes a timeline of job execution.
    pub timeline: Option<Vec<QueryTimelineSample>>,
    /// Output only. Total number of partitions processed from all partitioned tables referenced in the job.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub total_partitions_processed: Option<i64>,
    /// Output only. Total bytes processed for the job.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub total_bytes_processed: Option<i64>,
    /// Output only. For dry-run jobs, totalBytesProcessed is an estimate and
    /// this field specifies the accuracy of the estimate. Possible values can be:
    /// UNKNOWN: accuracy of the estimate is unknown.
    /// PRECISE: estimate is precise.
    /// LOWER_BOUND: estimate is lower bound of what the query would cost.
    /// UPPER_BOUND: estimate is upper bound of what the query would cost.
    pub total_bytes_processed_accuracy: Option<String>,
    /// Output only. If the project is configured to use on-demand pricing,
    /// then this field contains the total bytes billed for the job. If the project is configured to use flat-rate pricing,
    /// then you are not billed for bytes and this field is informational only.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub total_bytes_billed: Option<i64>,
    /// Output only. Billing tier for the job.
    /// This is a BigQuery-specific concept which is not related to the GCP notion of "free tier".
    /// The value here is a measure of the query's resource consumption relative to the amount of data scanned. For on-demand queries, the limit is 100, and all queries within this limit are billed at the standard on-demand rates. On-demand queries that exceed this limit will fail with a billingTierLimitExceeded error.
    pub billing_tier: Option<i32>,
    /// Output only. Slot-milliseconds for the job.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub total_slot_ms: Option<i64>,
    /// Output only. Whether the query result was fetched from the query cache.
    pub cache_hist: Option<bool>,
    /// Output only. Referenced tables for the job. Queries that reference more than 50 tables will not have a complete list.
    pub referenced_tables: Option<Vec<TableReference>>,
    /// Output only. Referenced routines for the job.
    pub referenced_routines: Option<Vec<RoutineReference>>,
    /// Output only. The schema of the results. Present only for successful dry run of non-legacy SQL queries.
    pub schema: Option<TableSchema>,
    /// Output only. The number of rows affected by a DML statement.
    /// Present only for DML statements INSERT, UPDATE or DELETE.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub num_dml_affected_rows: Option<i64>,
    /// Output only. Detailed statistics for DML statements INSERT, UPDATE, DELETE, MERGE or TRUNCATE
    pub dml_stats: Option<DmlStats>,
    /// Output only. GoogleSQL only: list of undeclared query parameters detected during a dry run validation
    pub undeclared_query_parameters: Option<Vec<QueryParameter>>,
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
    pub statement_type: Option<String>,
    /// Output only. The DDL operation performed, possibly dependent on the pre-existence of the DDL target.
    pub ddl_operation_performed: Option<String>,
    /// Output only. The DDL target table. Present only for CREATE/DROP TABLE/VIEW and DROP ALL ROW ACCESS POLICIES queries.
    pub ddl_target_table: Option<TableReference>,
    /// Output only. The DDL target row access policy. Present only for CREATE/DROP ROW ACCESS POLICY queries.
    pub ddl_target_row_access_policy: Option<RowAccessPolicyReference>,
    /// Output only. The number of row access policies affected by a DDL statement.
    /// Present only for DROP ALL ROW ACCESS POLICIES queries.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
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
    pub dcl_target_table: Option<TableReference>,
    /// Output only. Referenced view for DCL statement.
    pub dcl_target_view: Option<TableReference>,
    /// Output only. Search query specific statistics.
    pub search_statistics: Option<SearchStatistics>,
    /// Output only. Performance insights.
    pub performance_insights: Option<PerformanceInsights>,
    /// Output only. Statistics of a Spark procedure job.
    pub spark_statistics: Option<SparkStatistics>,
    /// Output only. Total bytes transferred for cross-cloud queries such as Cross Cloud Transfer and CREATE TABLE AS SELECT (CTAS).
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub transferred_bytes: Option<i64>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct SearchStatistics {
    /// Specifies the index usage mode for the query.
    pub index_usage_mode: Option<IndexUsageMode>,
    /// When indexUsageMode is UNUSED or PARTIALLY_USED, this field explains why indexes were not used in all or part of the search query. If indexUsageMode is FULLY_USED, this field is not populated.
    pub index_unused_reasons: Option<Vec<IndexUnusedReason>>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IndexUsageMode {
    #[default]
    IndexUsageModeUnspecified,
    Unused,
    PartiallyUsed,
    FullyUsed,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct IndexUnusedReason {
    /// Specifies the high-level reason for the scenario when no search index was used.
    pub code: Option<IndexUnusedCode>,
    /// Free form human-readable reason for the scenario when no search index was used.
    pub message: Option<String>,
    /// Specifies the base table involved in the reason that no search index was used.
    pub base_table: Option<TableReference>,
    /// Specifies the name of the unused search index, if available.
    pub index_name: Option<String>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IndexUnusedCode {
    #[default]
    CodeUnspecified,
    IndexConfigNotAvailable,
    PendingIndexCreation,
    BaseTableTruncated,
    IndexConfigModified,
    TimeTravelQuery,
    NoPruningPower,
    UnindexedSearchFields,
    UnsupportedSearchPattern,
    OptimizedWithMaterializedView,
    SecuredByDataMasking,
    MismatchedTextAnalyzer,
    BaseTableTooSmall,
    BaseTableTooLarge,
    EstimatedPerformanceGainTooLow,
    QueryCacheHit,
    StaleIndex,
    InternalError,
    OtherReason,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceInsights {
    /// Output only. Average execution ms of previous runs. Indicates the job ran slow compared to previous executions. To find previous executions, use INFORMATION_SCHEMA tables and filter jobs with same query hash.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub avg_previous_execution_ms: Option<i64>,
    /// Output only. Standalone query stage performance insights, for exploring potential improvements.
    pub stage_performance_standalone_insights: Option<Vec<StagePerformanceStandaloneInsight>>,
    /// Output only. Query stage performance insights compared to previous runs,
    /// for diagnosing performance regression.
    pub stage_performance_change_insights: Option<Vec<StagePerformanceChangeInsight>>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct StagePerformanceStandaloneInsight {
    /// Output only. The stage id that the insight mapped to.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub stage_id: Option<i64>,
    /// Output only. True if the stage has a slot contention issue.
    pub slot_contention: Option<bool>,
    /// Output only. True if the stage has insufficient shuffle quota.
    pub insufficient_shuffle_quota: Option<bool>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct StagePerformanceChangeInsight {
    /// Output only. The stage id that the insight mapped to.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub stage_id: Option<i64>,
    /// Output only. Input data change insight of the query stage.
    pub input_data_change: Option<InputDataChange>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct InputDataChange {
    /// Output only. Records read difference percentage compared to a previous run
    pub records_read_diff_percentage: Option<f64>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct SparkStatistics {
    /// Output only. Endpoints returned from Dataproc.
    /// Key list: - history_server_endpoint: A link to Spark job UI.
    /// An object containing a list of "key": value pairs.
    /// Example: { "name": "wrench", "mass": "1.3kg", "count": "3" }.
    pub endpoints: Option<HashMap<String, String>>,
    /// Output only. Spark job ID if a Spark job is created successfully.
    pub spark_job_id: Option<String>,
    /// Output only. Location where the Spark job is executed.
    /// A location is selected by BigQueury for jobs configured to run in a multi-region.
    pub spark_job_location: Option<String>,
    /// Output only. Logging info is used to generate a link to Cloud Logging.
    pub logging_info: Option<LoggingInfo>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct LoggingInfo {
    /// Output only. Resource type used for logging.
    pub resource_type: Option<String>,
    /// Output only. Project ID where the Spark logs were written.
    pub project_id: Option<String>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExportDataStatistics {
    /// Number of destination files generated in case of EXPORT DATA statement only.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub file_count: Option<i64>,
    /// [Alpha] Number of destination rows generated in case of EXPORT DATA statement only.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub row_count: Option<i64>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExternalServiceCost {
    /// External service name.
    pub external_service: String,
    /// External service cost in terms of bigquery bytes processed.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub bytes_processed: Option<i64>,
    /// External service cost in terms of bigquery bytes billed.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub bytes_billed: Option<i64>,
    /// External service cost in terms of bigquery slot milliseconds.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub slot_ms: Option<i64>,
    /// Non-preemptable reserved slots used for external job.
    /// For example, reserved slots for Cloua AI Platform job are the VM usages converted to BigQuery slot with equivalent mount of price.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub reserved_slot_count: Option<i64>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct BiEngineStatistics {
    /// Output only. Specifies which mode of BI Engine acceleration was performed (if any).
    pub bi_engine_mode: Option<BiEngineMode>,
    /// Output only. Specifies which mode of BI Engine acceleration was performed (if any).
    pub acceleration_mode: Option<BiEngineAccelerationMode>,
    /// In case of DISABLED or PARTIAL biEngineMode, these contain the explanatory reasons as to why BI Engine could not accelerate. In case the full query was accelerated, this field is not populated.
    pub bi_engine_reasons: Option<Vec<BiEngineReason>>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BiEngineMode {
    #[default]
    AccelerationModeUnspecified,
    Disabled,
    Partial,
    Full,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BiEngineAccelerationMode {
    #[default]
    BiEngineAccelerationModeUnspecified,
    BiEngineDisabled,
    PartialInput,
    FullInput,
    FullQuery,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct BiEngineReason {
    /// Output only. High-level BI Engine reason for partial or disabled acceleration
    pub code: BiEngineCode,
    /// Output only. Free form human-readable reason for partial or disabled acceleration.
    pub message: String,
}
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BiEngineCode {
    #[default]
    CodeUnspecified,
    NoReservation,
    InsufficientReservation,
    UnsupportedSqlText,
    InputTooLarge,
    OtherReason,
    TableExcluded,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct LoadQueryStatistics {
    /// Output only. Number of source files in a LOAD query.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub input_files: Option<i64>,
    /// Output only. Number of bytes of source data in a LOAD query.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub input_file_bytes: Option<i64>,
    /// Output only. Number of rows imported in a LOAD query. Note that while a LOAD query is in the running state, this value may change.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub output_rows: Option<i64>,
    /// Output only. Size of the loaded data in bytes.
    /// Note that while a LOAD query is in the running state, this value may change.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub output_bytes: Option<i64>,
    /// Output only. The number of bad records encountered while processing a LOAD query. Note that if the job has failed because of more bad records encountered than the maximum allowed in the load job configuration, then this number can be less than the total number of bad records present in the input data.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub bad_records: Option<i64>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExplainQueryStage {
    /// Human-readable name for the stage.
    pub name: String,
    /// Unique ID for the stage within the plan.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub id: i64,
    /// Stage start time represented as milliseconds since the epoch
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub start_ms: Option<i64>,
    /// Stage end time represented as milliseconds since the epoch.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub end_ms: Option<i64>,
    /// IDs for stages that are inputs to this stage.
    #[serde(default, deserialize_with = "crate::http::from_str_vec_option")]
    pub input_stages: Option<Vec<i64>>,
    /// Relative amount of time the average shard spent waiting to be scheduled.
    pub wait_ratio_avg: Option<f64>,
    /// Milliseconds the average shard spent waiting to be scheduled.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub wait_ms_avg: Option<i64>,
    /// Relative amount of time the slowest shard spent waiting to be scheduled.
    pub wait_ratio_max: Option<f64>,
    ///Milliseconds the slowest shard spent reading input.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub wait_ms_max: Option<i64>,
    /// Relative amount of time the average shard spent on CPU-bound tasks.
    pub compute_ratio_avg: Option<f64>,
    /// Milliseconds the average shard spent on CPU-bound tasks.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub compute_ms_avg: Option<i64>,
    /// Relative amount of time the slowest shard spent on CPU-bound tasks.
    pub compute_ratio_max: Option<f64>,
    /// Milliseconds the slowest shard spent on CPU-bound tasks.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub compute_ms_max: Option<i64>,
    /// Relative amount of time the average shard spent on writing output.
    pub write_ratio_avg: Option<f64>,
    /// Milliseconds the average shard spent on writing output.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub write_ms_avg: Option<i64>,
    /// Relative amount of time the slowest shard spent on writing output.
    pub write_ratio_max: Option<f64>,
    /// Milliseconds the slowest shard spent on writing output.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub write_ms_max: Option<i64>,
    /// Total number of bytes written to shuffle.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub shuffle_output_bytes: Option<i64>,
    /// Total number of bytes written to shuffle.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub shuffle_output_bytes_spilled: Option<i64>,
    /// Number of records read into the stage.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub records_read: Option<i64>,
    /// Number of records read written by the stage.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub records_written: Option<i64>,
    /// Number of parallel input segments to be processed
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub parallel_inputs: Option<i64>,
    /// Number of parallel input segments completed.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub completed_parallel_inputs: Option<i64>,
    /// Current status for this stage.
    pub status: String,
    /// List of operations within the stage in dependency order (approximately chronological).
    pub steps: Option<Vec<ExplainQueryStep>>,
    /// Slot-milliseconds used by the stage
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub slot_ms: Option<i64>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExplainQueryStep {
    /// Machine-readable operation type.
    pub kind: String,
    /// Human-readable description of the step(s).
    pub substeps: Option<Vec<String>>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct DmlStats {
    /// Output only. Number of inserted Rows. Populated by DML INSERT and MERGE statements
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub inserted_row_count: Option<i64>,
    /// Output only. Number of deleted Rows. populated by DML DELETE, MERGE and TRUNCATE statements.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub deleted_row_count: Option<i64>,
    /// Output only. Number of updated Rows. Populated by DML UPDATE and MERGE statements.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub updated_row_count: Option<i64>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct QueryTimelineSample {
    /// Milliseconds elapsed since the start of query execution.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub elapsed_ms: Option<i64>,
    /// Cumulative slot-ms consumed by the query.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub total_slot_ms: Option<i64>,
    /// Total units of work remaining for the query. This number can be revised (increased or decreased) while the query is running.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub pending_units: Option<i64>,
    /// Total parallel units of work completed by this query.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub completed_units: Option<i64>,
    /// Total number of active workers.
    /// This does not correspond directly to slot usage.
    /// This is the largest value observed since the last sample.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub active_units: Option<i64>,
    /// Units of work that can be scheduled immediately. Providing additional slots for these units of work will accelerate the query, if no other query in the reservation needs additional slots.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub estimated_runnable_units: Option<i64>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct MlStatistics {
    /// Output only. Maximum number of iterations specified as maxIterations in the 'CREATE MODEL' query.
    /// The actual number of iterations may be less than this number due to early stop.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub max_iterations: Option<i64>,
    /// Results for all completed iterations. Empty for hyperparameter tuning jobs.
    pub iteration_results: Option<Vec<IterationResult>>,
    /// Output only. The type of the model that is being trained.
    pub model_type: ModelType,
    /// Output only. Training type of the job.
    pub training_type: TrainingType,
    /// Output only. Trials of a hyperparameter tuning job sorted by trialId.
    pub hparam_trials: Option<Vec<HparamTuningTrial>>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrainingType {
    /// Unspecified training type.
    #[default]
    TrainingTypeUnspecified,
    /// Single training with fixed parameter space.
    SingleTraining,
    /// Hyperparameter tuning training.
    HparamTuning,
}

pub fn is_select_query(statistics: &Option<JobStatistics>, config: &JobConfiguration) -> bool {
    has_statement_type(statistics, config, "SELECT")
}
pub fn is_script(statistics: &Option<JobStatistics>, config: &JobConfiguration) -> bool {
    has_statement_type(statistics, config, "SCRIPT")
}
fn has_statement_type(statistics: &Option<JobStatistics>, config: &JobConfiguration, statement_type: &str) -> bool {
    match config.job {
        JobType::Query(_) => {}
        _ => return false,
    }
    let statistics = match &statistics {
        Some(v) => v,
        None => return false,
    };
    let query = match &statistics.query {
        Some(v) => v,
        None => return false,
    };
    let stmt = match &query.statement_type {
        Some(v) => v,
        None => return false,
    };
    stmt == statement_type
}
