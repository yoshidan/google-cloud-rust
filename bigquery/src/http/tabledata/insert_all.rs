use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};
use serde::Serialize;

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Row<T: Serialize> {
    /// [Optional] A unique ID for each row. BigQuery uses this
    /// property to detect duplicate insertion requests on a best-effort basis.
    pub insert_id: Option<String>,

    /// [Required] A JSON object that contains a row of data. The
    /// object's properties and values must match the destination table's schema.
    pub json: T,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertAllRequest<T: Serialize> {
    /// Optional. Insert all valid rows of a request, even if invalid rows exist.
    /// The default value is false, which causes the entire request to fail if any invalid rows exist.
    pub skip_invalid_rows: Option<bool>,
    /// Optional. Accept rows that contain values that do not match the schema.
    /// The unknown values are ignored. Default is false, which treats unknown values as errors.
    pub ignore_unknown_values: Option<bool>,
    /// Optional. If specified, treats the destination table as a base template, and inserts the rows into an instance table named "{destination}{templateSuffix}". BigQuery will manage creation of the instance table, using the schema of the base template table.
    /// See https://cloud.google.com/bigquery/streaming-data-into-bigquery#template-tables for considerations when working with templates tables.
    pub template_suffix: Option<String>,
    /// Data to insert
    pub rows: Vec<Row<T>>,
    /// Optional. Unique request trace id. Used for debugging purposes only.
    /// It is case-sensitive, limited to up to 36 ASCII characters. A UUID is recommended.
    pub trace_id: Option<String>,
}

impl<T: Serialize> Default for InsertAllRequest<T> {
    fn default() -> Self {
        Self {
            skip_invalid_rows: None,
            ignore_unknown_values: None,
            template_suffix: None,
            rows: vec![],
            trace_id: None,
        }
    }
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ErrorMessage {
    /// A short error code that summarizes the error.
    pub reason: String,
    /// Specifies where the error occurred, if present.
    pub location: String,
    /// Debugging information. This property is internal to Google and should not be used.
    pub debug_info: String,
    /// A human-readable description of the error.
    pub message: String,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Error {
    pub index: i32,
    pub errors: Vec<ErrorMessage>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertAllResponse {
    #[serde(default)]
    pub kind: String,
    pub insert_errors: Option<Vec<Error>>,
}

pub fn build<T: Serialize>(
    base_url: &str,
    client: &Client,
    project_id: &str,
    dataset_id: &str,
    table_id: &str,
    data: &InsertAllRequest<T>,
) -> RequestBuilder {
    let url = format!("{base_url}/projects/{project_id}/datasets/{dataset_id}/tables/{table_id}/insertAll");
    client.post(url).json(data)
}
