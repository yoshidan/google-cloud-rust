use std::collections::HashMap;

use crate::http::types::{StandardSqlDataType, StandardSqlField};

pub mod delete;
pub mod get;
pub mod insert;
pub mod list;
pub mod update;

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RoutineReference {
    /// Required. The ID of the project containing this table.
    pub project_id: String,
    /// Required. The ID of the dataset containing this table.
    pub dataset_id: String,
    /// Required. The ID of the routine.
    /// The ID must contain only letters (a-z, A-Z), numbers (0-9), or underscores (_).
    /// The maximum length is 256 characters.
    pub routine_id: String,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RoutineType {
    #[default]
    RoutineTypeUnspecified,
    ScalarFunction,
    Procedure,
    TableValuedFunction,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Language {
    #[default]
    LanguageUnspecified,
    Sql,
    Javascript,
    Python,
    Java,
    Scala,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ArgumentKind {
    #[default]
    ArgumentKindUnspecified,
    FixedType,
    AnyType,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Mode {
    #[default]
    ModeUnspecified,
    In,
    Out,
    Inout,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct StandardSqlTableType {
    /// The columns in this table type
    pub columns: Vec<StandardSqlField>,
}

/// JavaScript UDF determinism levels.
/// If all JavaScript UDFs are DETERMINISTIC, the query result is potentially cachable (see below).
/// If any JavaScript UDF is NOT_DETERMINISTIC, the query result is not cacheable.
/// Even if a JavaScript UDF is deterministic, many other factors can prevent usage of cached query results.
/// Example factors include but not limited to: DDL/DML, non-deterministic SQL function calls,
/// update of referenced tables/views/UDFs or imported JavaScript libraries.
/// SQL UDFs cannot have determinism specified. Their determinism is automatically determined.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeterminismLevel {
    #[default]
    DeterminismLevelUnspecified,
    Deterministic,
    NotDeterministic,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RemoteFunctionOptions {
    /// Endpoint of the user-provided remote service,
    /// e.g. https://us-east1-my_gcf_project.cloudfunctions.net/remote_add
    pub endpoint: Option<String>,
    /// Fully qualified name of the user-provided connection object which holds the authentication
    /// information to send requests to the remote service. Format: "projects/{projectId}/locations/{locationId}/connections/{connectionId}"
    pub connection: Option<String>,
    /// User-defined context as a set of key/value pairs, which will be sent as function invocation context together with batched arguments in the requests to the remote service. The total number of bytes of keys and values must be less than 8KB.
    /// An object containing a list of "key": value pairs.
    /// Example: { "name": "wrench", "mass": "1.3kg", "count": "3" }.
    pub user_defined_context: Option<HashMap<String, String>>,
    /// Max number of rows in each batch sent to the remote service.
    /// If absent or if 0, BigQuery dynamically decides the number of rows in a batch.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub max_batching_rows: Option<i64>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Argument {
    /// Optional. The name of this argument. Can be absent for function return argument.
    pub name: Option<String>,
    /// Optional. Defaults to FIXED_TYPE.
    pub argument_kind: Option<ArgumentKind>,
    /// Optional. Specifies whether the argument is input or output. Can be set for procedures only.
    pub mode: Option<Mode>,
    /// Required unless argumentKind = ANY_TYPE.
    pub data_type: StandardSqlDataType,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct SparkOptions {
    /// Fully qualified name of the user-provided Spark connection object.
    /// Format: "projects/{projectId}/locations/{locationId}/connections/{connectionId}"
    pub connection: Option<String>,
    /// Runtime version.
    /// If not specified, the default runtime version is used.
    pub runtime_version: Option<String>,
    /// Custom container image for the runtime environment.
    pub container_image: Option<String>,
    /// Configuration properties as a set of key/value pairs, which will be passed on to the Spark application.
    /// For more information, see Apache Spark and the procedure option list.
    /// An object containing a list of "key": value pairs.
    /// Example: { "name": "wrench", "mass": "1.3kg", "count": "3" }.
    pub properties: Option<HashMap<String, String>>,
    /// The main file/jar URI of the Spark application.
    /// Exactly one of the definitionBody field and the mainFileUri field must be set for Python.
    /// Exactly one of mainClass and mainFileUri field should be set for Java/Scala language type.
    pub main_file_uri: Option<String>,
    /// Python files to be placed on the PYTHONPATH for PySpark application.
    /// Supported file types: .py, .egg, and .zip. For more information about Apache Spark, see Apache Spark.
    pub py_file_uris: Option<Vec<String>>,
    /// JARs to include on the driver and executor CLASSPATH.
    /// For more information about Apache Spark, see Apache Spark.
    pub jar_uris: Option<Vec<String>>,
    /// Files to be placed in the working directory of each executor.
    /// For more information about Apache Spark, see Apache Spark.
    pub file_uris: Option<Vec<String>>,
    /// Archive files to be extracted into the working directory of each executor.
    /// For more information about Apache Spark, see Apache Spark.
    pub archive_uris: Option<Vec<String>>,
    /// The fully qualified name of a class in jarUris, for example, com.example.wordcount.
    /// Exactly one of mainClass and main_jar_uri field should be set for Java/Scala language type.
    pub main_class: Option<String>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Routine {
    /// Output only. A hash of this resource.
    pub etag: String,
    /// Required. Reference describing the ID of this routine.
    pub routine_reference: RoutineReference,
    /// Required. The type of routine.
    pub routine_type: RoutineType,
    /// Output only. The time when this routine was created, in milliseconds since the epoch.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub creation_time: Option<i64>,
    /// Output only. The time when this routine was last modified, in milliseconds since the epoch.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub last_modified_time: Option<i64>,
    /// Optional. Defaults to "SQL" if remoteFunctionOptions field is absent, not set otherwise.
    pub language: Option<Language>,
    /// Optional.
    pub arguments: Option<Vec<Argument>>,
    /// Optional if language = "SQL"; required otherwise. Cannot be set if routineType = "TABLE_VALUED_FUNCTION".
    /// If absent, the return type is inferred from definitionBody at query time in each query that references this routine. If present,
    /// then the evaluated result will be cast to the specified returned type at query time.
    /// For example, for the functions created with the following statements:
    /// CREATE FUNCTION Add(x FLOAT64, y FLOAT64) RETURNS FLOAT64 AS (x + y);
    /// CREATE FUNCTION Increment(x FLOAT64) AS (Add(x, 1));
    /// CREATE FUNCTION Decrement(x FLOAT64) RETURNS FLOAT64 AS (Add(x, -1));
    /// The returnType is {typeKind: "FLOAT64"} for Add and Decrement, and is absent for Increment (inferred as FLOAT64 at query time).
    /// Suppose the function Add is replaced by CREATE OR REPLACE FUNCTION Add(x INT64, y INT64) AS (x + y);
    /// Then the inferred return type of Increment is automatically changed to INT64 at query time, while the return type of Decrement remains FLOAT64.
    pub return_type: Option<StandardSqlDataType>,
    /// Optional. Can be set only if routineType = "TABLE_VALUED_FUNCTION".
    /// If absent, the return table type is inferred from definitionBody at query time in each query that references this routine.
    /// If present, then the columns in the evaluated table result will be cast to match the column types specified in return table type, at query time.
    pub return_table_type: Option<StandardSqlTableType>,
    /// Optional. If language = "JAVASCRIPT", this field stores the path of the imported JAVASCRIPT libraries.
    pub imported_libraries: Option<Vec<String>>,
    /// Required. The body of the routine.
    /// For functions, this is the expression in the AS clause.
    /// If language=SQL, it is the substring inside (but excluding) the parentheses.
    /// For example, for the function created with the following statement:
    /// CREATE FUNCTION JoinLines(x string, y string) as (concat(x, "\n", y))
    /// The definitionBody is concat(x, "\n", y) (\n is not replaced with linebreak).
    /// If language=JAVASCRIPT, it is the evaluated string in the AS clause.
    /// For example, for the function created with the following statement:
    /// CREATE FUNCTION f() RETURNS STRING LANGUAGE js AS 'return "\n";\n'
    /// The definitionBody is return "\n";\n
    /// Note that both \n are replaced with linebreaks.
    pub definition_body: String,
    /// Optional. The description of the routine, if defined.
    pub description: Option<String>,
    /// Optional. The determinism level of the JavaScript UDF, if defined.
    pub determinism_level: Option<DeterminismLevel>,
    /// Optional. Remote function specific options.
    pub remote_function_options: Option<RemoteFunctionOptions>,
    /// Optional. Spark specific options.
    pub spark_options: Option<SparkOptions>,
}
