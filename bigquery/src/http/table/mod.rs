pub mod delete;
pub mod get;
pub mod insert;
pub mod patch;

use crate::http::types::EncryptionConfiguration;

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct TableReference {
    /// Required. The ID of the project containing this table.
    pub project_id: String,
    /// Required. The ID of the dataset containing this table.
    pub dataset_id: String,
    /// Required. The ID of the table.
    /// The ID must contain only letters (a-z, A-Z), numbers (0-9), or underscores (_).
    /// The maximum length is 1,024 characters. Certain operations allow suffixing of the table ID with a partition decorator, such as sample_table$20190123.
    pub table_id: String,
}
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct CsvOptions {
    /// Optional. The separator character for fields in a CSV file.
    /// The separator is interpreted as a single byte.
    /// For files encoded in ISO-8859-1, any single character can be used as a separator.
    /// For files encoded in UTF-8,
    /// characters represented in decimal range 1-127 (U+0001-U+007F) can be used without any modification.
    /// UTF-8 characters encoded with multiple bytes (i.e. U+0080 and above) will
    /// have only the first byte used for separating fields.
    /// The remaining bytes will be treated as a part of the field.
    /// BigQuery also supports the escape sequence "\t" (U+0009) to specify a tab separator.
    /// The default value is comma (",", U+002C).
    pub field_delimiter: Option<String>,
    /// Optional. The number of rows at the top of a CSV file that BigQuery will skip when reading the data.
    /// The default value is 0.
    /// This property is useful if you have header rows in the file that should be skipped.
    /// When autodetect is on, the behavior is the following:
    ///
    /// skipLeadingRows unspecified - Autodetect tries to detect headers in the first row.
    ///     If they are not detected, the row is read as data.
    ///     Otherwise data is read starting from the second row.
    /// skipLeadingRows is 0 - Instructs autodetect that there are no headers and data should be read starting from the first row.
    /// skipLeadingRows = N > 0 - Autodetect skips N-1 rows and tries to detect headers in row N.
    ///     If headers are not detected, row N is just skipped. Otherwise row N is used to extract column names for the detected schema.
    #[serde(deserialize_with = "crate::http::from_str_option")]
    #[serde(default)]
    pub skip_leading_rows: Option<i64>,
    /// Optional. The value that is used to quote data sections in a CSV file.
    /// BigQuery converts the string to ISO-8859-1 encoding,
    /// and then uses the first byte of the encoded string to split the data in its raw, binary state.
    /// The default value is a double-quote (").
    /// If your data does not contain quoted sections, set the property value to an empty string.
    /// If your data contains quoted newline characters,
    /// you must also set the allowQuotedNewlines property to true.
    /// To include the specific quote character within a quoted value,
    /// precede it with an additional matching quote character.
    /// For example, if you want to escape the default character ' " ', use ' "" '.
    pub quote: Option<String>,
    /// Optional. Indicates if BigQuery should allow quoted data sections that contain newline characters in a CSV file.
    /// The default value is false.
    pub allow_quote_new_lines: Option<bool>,
    /// Optional. Indicates if BigQuery should accept rows that are missing trailing optional columns.
    /// If true, BigQuery treats missing trailing columns as null values.
    /// If false, records with missing trailing columns are treated as bad records,
    /// and if there are too many bad records, an invalid error is returned in the job result.
    /// The default value is false.
    pub allow_jagged_rows: Option<bool>,
    /// Optional. The character encoding of the data.
    /// The supported values are UTF-8, ISO-8859-1, UTF-16BE, UTF-16LE, UTF-32BE, and UTF-32LE.
    /// The default value is UTF-8.
    /// BigQuery decodes the data after the raw, binary data has been
    /// split using the values of the quote and fieldDelimiter properties.
    pub encoding: Option<String>,
    /// Optional. Indicates if the embedded ASCII control characters
    /// (the first 32 characters in the ASCII-table, from '\x00' to '\x1F') are preserved.
    pub preserve_ascii_control_characters: Option<bool>,
}
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct BigtableOptions {
    /// Optional. tabledata.list of column families to expose in the table schema along with their types.
    /// This list restricts the column families that can be referenced in queries and specifies their value types.
    /// You can use this list to do type conversions - see the 'type' field for more details. If you leave this list empty, all column families are present in the table schema and their values are read as BYTES. During a query only the column families referenced in that query are read from Bigtable.
    pub column_families: Vec<BigtableColumnFamily>,
    /// Optional. If field is true, then the column families that are not specified in columnFamilies list are not exposed in the table schema.
    /// Otherwise, they are read with BYTES type values.
    /// The default value is false.
    pub ignore_unspecified_column_families: Option<bool>,
    /// Optional. If field is true, then the rowkey column families will be read and converted to string.
    /// Otherwise they are read with BYTES type values and users need to manually cast them with CAST if necessary.
    /// The default value is false.
    pub read_rowkey_as_string: Option<bool>,
}
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct BigtableColumnFamily {
    /// Identifier of the column family.
    pub family_id: String,
    /// Optional. The type to convert the value in cells of this column family.
    /// The values are expected to be encoded using HBase Bytes.toBytes function when using the BINARY encoding value. Following BigQuery types are allowed (case-sensitive) - BYTES STRING INTEGER FLOAT BOOLEAN Default type is BYTES.
    /// This can be overridden for a specific column by listing that column in 'columns' and specifying a type for it.
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub data_type: Option<String>,
    /// Optional. The encoding of the values when the type is not STRING.
    /// Acceptable encoding values are: TEXT - indicates values are alphanumeric text strings. BINARY - indicates values are encoded using HBase Bytes.toBytes family of functions.
    /// This can be overridden for a specific column by listing that column in 'columns' and specifying an encoding for it.
    pub encoding: Option<String>,
    /// Optional. Lists of columns that should be exposed as individual fields
    /// as opposed to a list of (column name, value) pairs.
    /// All columns whose qualifier matches a qualifier in this list can be accessed as ..
    /// Other columns can be accessed as a list through .Column field.
    pub columns: Option<Vec<BigtableColumn>>,
    /// Optional. If this is set only the latest version of value are exposed for all columns in this column family.
    /// This can be overridden for a specific column by listing that column in 'columns' and specifying a different setting for that column.
    pub only_read_latest: Option<bool>,
}
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct BigtableColumn {
    /// [Required] Qualifier of the column. Columns in the parent column family that has this exact qualifier are exposed as . field. If the qualifier is valid UTF-8 string, it can be specified in the qualifierString field. Otherwise, a base-64 encoded value must be set to qualifierEncoded. The column field name is the same as the column qualifier. However, if the qualifier is not a valid BigQuery field identifier i.e.
    /// does not match [a-zA-Z][a-zA-Z0-9_]*, a valid identifier must be provided as fieldName.
    pub qualifier_encoded: Option<String>,
    pub qualifier_string: Option<String>,
    /// Optional. If the qualifier is not a valid BigQuery field identifier i.e.
    /// does not match [a-zA-Z][a-zA-Z0-9_]*, a valid identifier must be provided as the column field name and is used as field name in queries.
    pub field_name: Option<String>,
    /// Optional. The type to convert the value in cells of this column.
    /// The values are expected to be encoded using HBase Bytes.
    /// toBytes function when using the BINARY encoding value. Following BigQuery types are allowed (case-sensitive) - BYTES STRING INTEGER FLOAT BOOLEAN Default type is BYTES. 'type' can also be set at the column family level. However, the setting at this level takes precedence if 'type' is set at both levels.
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub data_type: Option<String>,
    /// Optional. The encoding of the values when the type is not STRING.
    /// Acceptable encoding values are: TEXT - indicates values are alphanumeric text strings.
    /// BINARY - indicates values are encoded using HBase Bytes.toBytes family of functions.
    /// 'encoding' can also be set at the column family level.
    /// However, the setting at this level takes precedence if 'encoding' is set at both levels.
    pub encoding: Option<String>,
    /// Optional. If this is set, only the latest version of value in this column are exposed.
    /// 'onlyReadLatest' can also be set at the column family level.
    /// However, the setting at this level takes precedence if 'onlyReadLatest' is set at both levels.
    pub only_read_latest: Option<bool>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct GoogleSheetsOptions {
    /// Optional. The number of rows at the top of a sheet that BigQuery will skip when reading the data.
    /// The default value is 0. This property is useful if you have header rows that should be skipped.
    /// When autodetect is on, the behavior is the following:
    /// * skipLeadingRows unspecified - Autodetect tries to detect headers in the first row.
    /// If they are not detected, the row is read as data.
    /// Otherwise data is read starting from the second row.
    /// * skipLeadingRows is 0 - Instructs autodetect that there are no headers and data should be read starting from the first row.
    /// * skipLeadingRows = N > 0 - Autodetect skips N-1 rows and tries to detect headers in row N.
    /// If headers are not detected, row N is just skipped.
    /// Otherwise row N is used to extract column names for the detected schema.
    #[serde(deserialize_with = "crate::http::from_str_option")]
    #[serde(default)]
    pub skip_leading_rows: Option<i64>,
    /// Optional. Range of a sheet to query from. Only used when non-empty.
    /// Typical format: sheet_name!top_left_cell_id:bottom_right_cell_id For example: sheet1!A1:B20
    pub range: Option<String>,
}
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct HivePartitioningOptions {
    /// Optional. When set, what mode of hive partitioning to use when reading data.
    /// The following modes are supported:
    ///
    /// AUTO: automatically infer partition key name(s) and type(s).
    /// STRINGS: automatically infer partition key name(s). All types are strings.
    /// CUSTOM: partition key schema is encoded in the source URI prefix.
    /// Not all storage formats support hive partitioning.
    /// Requesting hive partitioning on an unsupported format will lead to an error.
    /// Currently supported formats are: JSON, CSV, ORC, Avro and Parquet.
    pub mode: Option<String>,
    /// Optional. When hive partition detection is requested,
    /// a common prefix for all source uris must be required.
    /// The prefix must end immediately before the partition key encoding begins.
    /// For example, consider files following this data layout:
    ///
    /// gs://bucket/path_to_table/dt=2019-06-01/country=USA/id=7/file.avro
    /// gs://bucket/path_to_table/dt=2019-05-31/country=CA/id=3/file.avro
    ///
    /// When hive partitioning is requested with either AUTO or STRINGS detection, the common prefix can be either of gs://bucket/path_to_table or gs://bucket/path_to_table/.
    ///
    /// CUSTOM detection requires encoding the partitioning schema immediately after the common prefix. For CUSTOM, any of
    ///
    /// gs://bucket/path_to_table/{dt:DATE}/{country:STRING}/{id:INTEGER}
    /// gs://bucket/path_to_table/{dt:STRING}/{country:STRING}/{id:INTEGER}
    /// gs://bucket/path_to_table/{dt:DATE}/{country:STRING}/{id:STRING}
    /// would all be valid source URI prefixes.
    pub source_uri_prefix: Option<String>,
    /// Optional. If set to true, queries over this table require a partition filter that can be used for partition elimination to be specified.
    /// Note that this field should only be true when creating a permanent external table or querying a temporary external table.
    /// Hive-partitioned loads with requirePartitionFilter explicitly set to true will fail.
    pub require_partition_filter: Option<bool>,
    /// Output only. For permanent external tables,
    /// this field is populated with the hive partition keys in the order they were inferred.
    /// The types of the partition keys can be deduced by checking the table schema (which will include the partition keys). Not every API will populate this field in the output.
    /// For example, Tables.Get will populate it, but Tables.List will not contain this field.
    pub fields: Vec<String>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DecimalTargetType {
    /// Decimal values could be converted to NUMERIC type.
    #[default]
    Numeric,
    /// Decimal values could be converted to BIGNUMERIC type.
    Bignumeric,
    /// Decimal values could be converted to STRING type.
    String,
}
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct AvroOptions {
    /// Optional. If sourceFormat is set to "AVRO",
    /// indicates whether to interpret logical types as the corresponding BigQuery data type
    /// (for example, TIMESTAMP),
    /// instead of using the raw type (for example, INTEGER).
    pub use_avro_logical_types: Option<bool>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ParquetOptions {
    /// Optional. Indicates whether to infer Parquet ENUM logical type as STRING instead of BYTES by default.
    pub enum_as_string: Option<bool>,
    /// Optional. Indicates whether to use schema inference specifically for Parquet LIST logical type.
    pub enable_list_interface: Option<bool>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ObjectMetadata {
    /// Unspecified by default.
    #[default]
    ObjectMetadataUnspecified,
    /// Directory listing of objects.
    Simple,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MetadataCacheMode {
    /// Unspecified metadata cache mode.
    #[default]
    MetadataCacheModeUnspecified,
    /// Set this mode to trigger automatic background refresh of metadata cache from the external source.
    /// Queries will use the latest available cache version within the table's maxStaleness interval.
    Automatic,
    /// Set this mode to enable triggering manual refresh of the metadata cache from external source.
    /// Queries will use the latest manually triggered cache version within the table's maxStaleness interval.
    Manual,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Streamingbuffer {
    /// Output only. A lower-bound estimate of the number of bytes currently in the streaming buffer.
    pub estimated_bytes: String,
    /// Output only. A lower-bound estimate of the number of rows currently in the streaming buffer.
    pub estimated_rows: String,
    /// Output only. Contains the timestamp of the oldest entry in the streaming buffer,
    /// in milliseconds since the epoch, if the streaming buffer is available.
    #[serde(deserialize_with = "crate::http::from_str_option")]
    #[serde(default)]
    pub oldest_entry_time: Option<u64>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotDefinition {
    /// Required. Reference describing the ID of the table that was snapshot.
    pub base_table_reference: TableReference,
    /// Required. The time at which the base table was snapshot.
    /// This value is reported in the JSON response using RFC3339 format.
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub snapshot_time: Option<time::OffsetDateTime>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct CloneDefinition {
    /// Required. Reference describing the ID of the table that was cloned.
    pub base_table_reference: TableReference,
    /// Required. The time at which the base table was snapshot.
    /// This value is reported in the JSON response using RFC3339 format.
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub clone_time: Option<time::OffsetDateTime>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserDefinedFunctionResource {
    /// [Pick one] A code resource to load from a Google Cloud Storage URI (gs://bucket/path).
    pub resource_uri: Option<String>,
    /// [Pick one] An inline resource that contains code for a user-defined function (UDF).
    /// Providing a inline code resource is equivalent to providing a URI for a file containing the same code.
    pub inline_code: Option<String>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ViewDefinition {
    /// Required. A query that BigQuery executes when the view is referenced.
    pub query: String,
    /// Describes user-defined function resources used in the query.
    pub user_defined_function_resources: Option<Vec<UserDefinedFunctionResource>>,
    /// Queries and views that reference this view must use the same flag value.
    /// A wrapper is used here because the default value is True..
    pub use_legacy_sql: bool,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct MaterializedViewDefinition {
    /// Required. A query whose results are persisted.
    pub query: String,
    /// Output only. The time when this materialized view was last refreshed, in milliseconds since the epoch.
    #[serde(deserialize_with = "crate::http::from_str_option")]
    #[serde(default)]
    pub last_refresh_time: Option<i64>,
    /// Optional. Enable automatic refresh of the materialized view when the base table is updated. The default value is "true".
    pub enable_refresh: Option<bool>,
    /// Optional. The maximum frequency at which this materialized view will be refreshed. The default value is "1800000" (30 minutes).
    #[serde(deserialize_with = "crate::http::from_str_option")]
    #[serde(default)]
    pub refresh_interval_ms: Option<u64>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct PolicyTag {
    /// A list of policy tag resource names. For example,
    /// "projects/1/locations/eu/taxonomies/2/policyTags/3".
    /// At most 1 policy tag is currently allowed.
    pub names: Vec<String>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RoundingMode {
    /// Unspecified will default to using ROUND_HALF_AWAY_FROM_ZERO.
    #[default]
    RoundingModeUnspecified,
    /// ROUND_HALF_AWAY_FROM_ZERO rounds half values away from zero when applying precision and scale upon writing of NUMERIC and BIGNUMERIC values.
    /// For Scale: 0 1.1, 1.2, 1.3, 1.4 => 1 1.5, 1.6, 1.7, 1.8, 1.9 => 2.
    RoundHalfAwayFromZero,
    /// ROUND_HALF_EVEN rounds half values to the nearest even when applying precision and scale upon writing of NUMERIC and BIGNUMERIC values.
    /// For Scale: 0 1.1, 1.2, 1.3, 1.4 => 1 1.5 => 2 1.6, 1.7, 1.8, 1.9 => 2 2.5 => 2
    RoundHalfEven,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TableFieldMode {
    #[default]
    Nullable,
    Required,
    Repeated,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TableFieldType {
    #[default]
    String,
    Bytes,
    Integer,
    Float,
    Boolean,
    Timestamp,
    Record,
    Date,
    Time,
    Datetime,
    Numeric,
    Decimal,
    Bignumeric,
    Interval,
    Json,
    // aliases
    Bool,
    Bigdecimal,
    Int64,
    Flaat64,
    Struct,
}
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct TableFieldSchema {
    /// Required. The field name.
    /// The name must contain only letters (a-z, A-Z), numbers (0-9), or underscores (_),
    /// and must start with a letter or underscore.
    /// The maximum length is 300 characters.
    pub name: String,
    /// Required. The field data type. Possible values include:
    ///
    /// STRING
    /// BYTES
    /// INTEGER (or INT64)
    /// FLOAT (or FLOAT64)
    /// BOOLEAN (or BOOL)
    /// TIMESTAMP
    /// DATE
    /// TIME
    /// DATETIME
    /// GEOGRAPHY,
    /// NUMERIC
    /// BIGNUMERIC
    /// RECORD (or STRUCT)
    /// Use of RECORD/STRUCT indicates that the field contains a nested schema.
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub data_type: TableFieldType,
    /// Optional. The field mode. Possible values include NULLABLE, REQUIRED and REPEATED.
    /// The default value is NULLABLE.
    pub mode: Option<TableFieldMode>,
    /// Optional. Describes the nested schema fields if the type property is set to RECORD.
    pub fields: Option<Vec<TableFieldSchema>>,
    /// Optional. The field description. The maximum length is 1,024 characters.
    pub description: Option<String>,
    /// Optional. The policy tags attached to this field, used for field-level access control.
    /// If not set, defaults to empty policyTags.
    pub policy_tags: Option<PolicyTag>,
    /// Optional. Maximum length of values of this field for STRINGS or BYTES.
    /// If maxLength is not specified, no maximum length constraint is imposed on this field.
    /// If type = "STRING", then maxLength represents the maximum UTF-8 length of strings in this field.
    /// If type = "BYTES", then maxLength represents the maximum number of bytes in this field.
    /// It is invalid to set this field if type ≠ "STRING" and ≠ "BYTES".
    #[serde(deserialize_with = "crate::http::from_str_option")]
    #[serde(default)]
    pub max_length: Option<i64>,
    /// Optional. Precision (maximum number of total digits in base 10) and scale (maximum number of digits in the fractional part in base 10) constraints for values of this field for NUMERIC or BIGNUMERIC.
    ///
    /// It is invalid to set precision or scale if type ≠ "NUMERIC" and ≠ "BIGNUMERIC".
    ///
    /// If precision and scale are not specified, no value range constraint is imposed on this field insofar as values are permitted by the type.
    ///
    /// Values of this NUMERIC or BIGNUMERIC field must be in this range when:
    ///
    /// Precision (P) and scale (S) are specified: [-10P-S + 10-S, 10P-S - 10-S]
    /// Precision (P) is specified but not scale (and thus scale is interpreted to be equal to zero): [-10P + 1, 10P - 1].
    /// Acceptable values for precision and scale if both are specified:
    ///
    /// If type = "NUMERIC": 1 ≤ precision - scale ≤ 29 and 0 ≤ scale ≤ 9.
    /// If type = "BIGNUMERIC": 1 ≤ precision - scale ≤ 38 and 0 ≤ scale ≤ 38.
    /// Acceptable values for precision if only precision is specified but not scale (and thus scale is interpreted to be equal to zero):
    ///
    /// If type = "NUMERIC": 1 ≤ precision ≤ 29.
    /// If type = "BIGNUMERIC": 1 ≤ precision ≤ 38.
    /// If scale is specified but not precision, then it is invalid.
    #[serde(deserialize_with = "crate::http::from_str_option")]
    #[serde(default)]
    pub precision: Option<i64>,
    /// Optional. See documentation for precision.
    #[serde(deserialize_with = "crate::http::from_str_option")]
    #[serde(default)]
    pub scale: Option<i64>,
    /// Optional. Specifies the rounding mode to be used when storing values of NUMERIC and BIGNUMERIC type.
    pub rounding_mode: Option<RoundingMode>,
    /// Optional. Field collation can be set only when the type of field is STRING. The following values are supported:
    ///
    /// 'und:ci': undetermined locale, case insensitive.
    /// '': empty string. Default to case-sensitive behavior.
    pub collation: Option<String>,
    /// Optional. A SQL expression to specify the default value for this field.
    /// https://cloud.google.com/bigquery/docs/default-values
    pub default_value_expression: Option<String>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimePartitionType {
    #[default]
    Hour,
    Day,
    Month,
    Year,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct TimePartitioning {
    /// Required. The supported types are DAY, HOUR, MONTH, and YEAR,
    /// which will generate one partition per day, hour, month, and year, respectively.
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub partition_type: TimePartitionType,
    /// Optional. Number of milliseconds for which to keep the storage for a partition.
    /// A wrapper is used here because 0 is an invalid value.
    #[serde(deserialize_with = "crate::http::from_str_option")]
    #[serde(default)]
    pub expiration_ms: Option<i64>,
    /// Optional. If not set, the table is partitioned by pseudo column '_PARTITIONTIME';
    /// if set, the table is partitioned by this field.
    /// The field must be a top-level TIMESTAMP or DATE field.
    /// Its mode must be NULLABLE or REQUIRED.
    /// A wrapper is used here because an empty string is an invalid value.
    pub field: Option<String>,
}
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct PartitionRange {
    /// Required. [Experimental] The start of range partitioning, inclusive.
    pub start: String,
    /// Required. [Experimental] The end of range partitioning, exclusive.
    pub end: String,
    /// Required. [Experimental] The width of each interval.
    pub interval: String,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RangePartitioning {
    /// Required. [Experimental] The table is partitioned by this field.
    /// The field must be a top-level NULLABLE/REQUIRED field.
    /// The only supported type is INTEGER/INT64.
    pub field: String,
    /// [Experimental] Defines the ranges for range partitioning.
    pub range: PartitionRange,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct TableSchema {
    /// Describes the fields in a table.
    pub fields: Vec<TableFieldSchema>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Clustering {
    /// One or more fields on which data should be clustered. Only top-level, non-repeated, simple-type fields are supported. The ordering of the clustering fields should be prioritized from most to least important for filtering purposes.
    /// Additional information on limitations can be found here: https://cloud.google.com/bigquery/docs/creating-clustered-tables#limitations
    pub fields: Vec<String>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SourceFormat {
    #[default]
    Csv,
    Avro,
    NewlineDelimitedJson,
    DatastoreBackup,
    GoogleSheets,
    Bigtable,
    Parquet,
    Orc,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExternalDataConfiguration {
    /// [Required] The fully-qualified URIs that point to your data in Google Cloud.
    /// For Google Cloud Storage URIs:
    /// Each URI can contain one '*' wildcard character and it must come after the 'bucket' name.
    /// Size limits related to load jobs apply to external data sources.
    /// For Google Cloud Bigtable URIs: Exactly one URI can be specified and it has be
    /// a fully specified and valid HTTPS URL for a Google Cloud Bigtable table.
    /// For Google Cloud Datastore backups,
    /// exactly one URI can be specified. Also, the '*' wildcard character is not allowed.
    pub source_uris: Vec<String>,
    /// Optional. The schema for the data.
    /// Schema is required for CSV and JSON formats if autodetect is not on.
    /// Schema is disallowed for Google Cloud Bigtable,
    /// Cloud Datastore backups, Avro, ORC and Parquet formats.
    pub schema: Option<TableSchema>,
    /// [Required] The data format. For CSV files, specify "CSV".
    /// For Google sheets, specify "GOOGLE_SHEETS".
    /// For newline-delimited JSON, specify "NEWLINE_DELIMITED_JSON".
    /// For Avro files, specify "AVRO".
    /// For Google Cloud Datastore backups, specify "DATASTORE_BACKUP".
    /// For ORC files, specify "ORC". For Parquet files, specify "PARQUET".
    /// [Beta] For Google Cloud Bigtable, specify "BIGTABLE".
    pub source_format: SourceFormat,
    /// Optional. The maximum number of bad records that BigQuery can ignore when reading data.
    /// If the number of bad records exceeds this value, an invalid error is returned in the job result.
    /// The default value is 0, which requires that all records are valid. This setting is ignored for Google Cloud Bigtable, Google Cloud Datastore backups, Avro, ORC and Parquet formats.
    pub max_bad_records: i32,
    /// Try to detect schema and format options automatically. Any option specified explicitly will be honored.
    pub autodetect: bool,
    /// Optional. Indicates if BigQuery should allow extra values that are not represented in the table schema.
    /// If true, the extra values are ignored.
    /// If false, records with extra columns are treated as bad records,
    /// and if there are too many bad records, an invalid error is returned in the job result.
    /// The default value is false.
    /// The sourceFormat property determines what BigQuery treats as an extra value:
    /// CSV:
    /// Trailing columns JSON: Named values that don't match any column names
    /// Google Cloud Bigtable: This setting is ignored.
    /// Google Cloud Datastore backups: This setting is ignored.
    /// Avro: This setting is ignored.
    /// ORC: This setting is ignored.
    /// Parquet: This setting is ignored.
    pub ignore_unknown_values: Option<bool>,
    /// Optional. The compression type of the data source.
    /// Possible values include GZIP and NONE.
    /// The default value is NONE.
    /// This setting is ignored for Google Cloud Bigtable, Google Cloud Datastore backups, Avro, ORC and Parquet formats.
    /// An empty string is an invalid value.
    pub compression: Option<bool>,
    /// Optional. Additional properties to set if sourceFormat is set to CSV.
    pub csv_options: Option<CsvOptions>,
    /// Optional. Additional options if sourceFormat is set to BIGTABLE.
    pub bigtable_options: Option<BigtableOptions>,
    /// Optional. Additional options if sourceFormat is set to GOOGLE_SHEETS.
    pub google_sheets_options: Option<GoogleSheetsOptions>,
    /// Optional. When set, configures hive partitioning support.
    /// Not all storage formats support hive partitioning -- requesting hive partitioning on an unsupported format will lead to an error,
    /// as will providing an invalid specification..
    pub hive_partitioning_options: Option<HivePartitioningOptions>,
    /// Optional. The connection specifying the credentials to be used to read external storage,
    /// such as Azure Blob, Cloud Storage, or S3.
    /// The connectionId can have the form "<project_id>.<location_id>.<connection_id>" or "projects/<project_id>/locations/<location_id>/connections/<connection_id>".
    pub connection_id: Option<String>,
    /// Defines the list of possible SQL data types to which the source decimal values are converted. This list and the precision and the scale parameters of the decimal field determine the target type. In the order of NUMERIC, BIGNUMERIC, and STRING, a type is picked if it is in the specified list and if it supports the precision and the scale. STRING supports all precision and scale values. If none of the listed types supports the precision and the scale, the type supporting the widest range in the specified list is picked, and if a value exceeds the supported range when reading the data, an error will be thrown.
    ///
    /// Example: Suppose the value of this field is ["NUMERIC", "BIGNUMERIC"]. If (precision,scale) is:
    ///
    /// (38,9) -> NUMERIC;
    /// (39,9) -> BIGNUMERIC (NUMERIC cannot hold 30 integer digits);
    /// (38,10) -> BIGNUMERIC (NUMERIC cannot hold 10 fractional digits);
    /// (76,38) -> BIGNUMERIC;
    /// (77,38) -> BIGNUMERIC (error if value exeeds supported range).
    /// This field cannot contain duplicate types. The order of the types in this field is ignored. For example, ["BIGNUMERIC", "NUMERIC"] is the same as ["NUMERIC", "BIGNUMERIC"] and NUMERIC always takes precedence over BIGNUMERIC.
    ///
    /// Defaults to ["NUMERIC", "STRING"] for ORC and ["NUMERIC"] for the other file format
    pub decimal_target_types: Option<Vec<DecimalTargetType>>,
    /// Optional. Additional properties to set if sourceFormat is set to AVRO.
    pub avro_options: Option<AvroOptions>,
    /// Optional. Additional properties to set if sourceFormat is set to PARQUET.
    pub parquet_options: Option<ParquetOptions>,
    /// Optional. When creating an external table, the user can provide a reference file with the table schema.
    /// This is enabled for the following formats: AVRO, PARQUET, ORC.
    pub reference_file_schema_uri: Option<String>,
    /// Optional. Metadata Cache Mode for the table. Set this to enable caching of metadata from external data source.
    pub metadata_cache_mode: Option<MetadataCacheMode>,
    /// Optional. ObjectMetadata is used to create Object Tables. Object Tables contain a listing of objects (with their metadata) found at the sourceUris. If ObjectMetadata is set, sourceFormat should be omitted.
    /// Currently SIMPLE is the only supported Object Metadata type.
    pub object_metadata: Option<ObjectMetadata>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Table {
    /// Output only. The resource type.
    pub kind: String,
    /// Output only. A hash of the resource.
    pub etag: String,
    /// Output only. The fully-qualified unique name of the dataset in the format projectId:datasetId.
    /// The dataset name without the project name is given in the datasetId field. When creating a new dataset,
    /// leave this field blank, and instead specify the datasetId field.
    pub id: String,
    /// Output only. A URL that can be used to access the resource again.
    /// You can use this URL in Get or Update requests to the resource.
    pub self_link: String,
    /// Required. Reference describing the ID of this table.
    pub table_reference: TableReference,
    /// Optional. A descriptive name for the dataset.
    pub friendly_name: Option<String>,
    /// Optional. Optional. A user-friendly description of the dataset.
    pub description: Option<String>,
    /// The labels associated with this dataset.
    /// You can use these to organize and group your datasets.
    /// You can set this property when inserting or updating a dataset.
    /// See Creating and Updating Dataset Labels for more information.
    ///
    /// An object containing a list of "key": value pairs.
    /// Example: { "name": "wrench", "mass": "1.3kg", "count": "3" }.
    pub labels: Option<std::collections::HashMap<String, String>>,
    /// Optional. Describes the schema of this table.
    pub schema: Option<TableSchema>,
    /// If specified, configures time-based partitioning for this table.
    pub time_partitioning: Option<TimePartitioning>,
    /// If specified, configures range partitioning for this table.
    pub range_partitioning: Option<RangePartitioning>,
    /// Clustering specification for the table.
    /// Must be specified with time-based partitioning,
    /// data in the table will be first partitioned and subsequently clustered.
    pub clustering: Option<Clustering>,
    /// Optional. If set to true, queries over this table require a partition filter that can be used for partition elimination to be specified.
    pub require_partition_filter: Option<bool>,
    /// Output only. The size of this table in logical bytes, excluding any data in the streaming buffer.
    #[serde(deserialize_with = "crate::http::from_str_option")]
    pub num_bytes: Option<i64>,
    /// Output only. The number of logical bytes in the table that are considered "long-term storage".
    #[serde(deserialize_with = "crate::http::from_str")]
    pub num_long_term_bytes: i64,
    /// Output only. The number of rows of data in this table, excluding any data in the streaming buffer.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub num_rows: u64,
    /// Output only. The time when this table was created, in milliseconds since the epoch.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub creation_time: i64,
    /// Optional. The time when this table expires, in milliseconds since the epoch.
    /// If not present, the table will persist indefinitely. Expired tables will be deleted and their storage reclaimed. The defaultTableExpirationMs property of the encapsulating dataset can be used to set a default expirationTime on newly created tables.
    #[serde(deserialize_with = "crate::http::from_str_option")]
    #[serde(default)]
    pub expiration_time: Option<i64>,
    /// Output only. The time when this table was last modified, in milliseconds since the epoch.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub last_modified_time: u64,
    /// Output only. Describes the table type. The following values are supported:
    ///
    /// TABLE: A normal BigQuery table.
    /// VIEW: A virtual table defined by a SQL query.
    /// EXTERNAL: A table that references data stored in an external storage system, such as Google Cloud Storage.
    /// MATERIALIZED_VIEW: A precomputed view defined by a SQL query.
    /// SNAPSHOT: An immutable BigQuery table that preserves the contents of a base table at a particular time. See additional information on table snapshots.
    /// The default value is TABLE.
    #[serde(rename(deserialize = "type"))]
    pub table_type: String,
    /// Optional. The view definition.
    pub view: Option<ViewDefinition>,
    /// Optional. The materialized view definition.
    pub materialized_view: Option<MaterializedViewDefinition>,
    /// Optional. Describes the data format, location,
    /// and other properties of a table stored outside of BigQuery.
    /// By defining these properties, the data source can then be queried as if it were a standard BigQuery table.
    pub external_data_configuration: Option<ExternalDataConfiguration>,
    /// Output only. The geographic location where the table resides.
    /// This value is inherited from the dataset.
    pub location: String,
    /// Output only. Contains information regarding this table's streaming buffer, if one is present.
    /// This field will be absent if the table is not being streamed to or
    /// if there is no data in the streaming buffer.
    pub streaming_buffer: Option<Streamingbuffer>,
    /// Custom encryption configuration (e.g., Cloud KMS keys).
    pub encryption_configuration: Option<EncryptionConfiguration>,
    /// Output only. Contains information about the snapshot. This value is set via snapshot creation.
    pub snapshot_definition: Option<SnapshotDefinition>,
    /// Optional. Defines the default collation specification of new STRING fields in the table.
    /// During table creation or update, if a STRING field is added to this table without explicit collation specified, then the table inherits the table default collation. A change to this field affects only fields added afterwards, and does not alter the existing fields. The following values are supported:
    ///
    /// 'und:ci': undetermined locale, case insensitive.
    /// '': empty string. Default to case-sensitive behavior.
    pub default_collation: Option<String>,
    /// Output only. Contains information about the clone. This value is set via the clone operation.
    pub clone_definition: Option<CloneDefinition>,
    /// Optional. The maximum staleness of data that could be returned when the table (or stale MV) is queried.
    /// Staleness encoded as a string encoding of sql IntervalValue type.
    pub max_staleness: Option<String>,
}
