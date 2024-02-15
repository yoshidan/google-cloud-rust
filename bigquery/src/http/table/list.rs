use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::table::{Clustering, RangePartitioning, TableReference, TimePartitioning};

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListTablesRequest {
    /// The maximum number of results to return in a single response page.
    /// Leverage the page tokens to iterate through the entire collection.
    pub max_results: Option<i64>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct View {
    /// True if view is defined in legacy SQL dialect, false if in GoogleSQL.
    pub use_legacy_sql: bool,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct TableOverview {
    /// The resource type.
    pub kind: String,
    /// An opaque ID of the table.
    pub id: String,
    /// A reference uniquely identifying table..
    pub table_reference: TableReference,
    /// The user-friendly name for this table.
    pub friendly_name: Option<String>,
    /// The labels associated with this table. You can use these to organize and group your tables.
    /// An object containing a list of "key": value pairs. Example: { "name": "wrench", "mass": "1.3kg", "count": "3" }.
    pub labels: Option<std::collections::HashMap<String, String>>,
    /// The time-based partitioning for this table.
    pub time_partitioning: Option<TimePartitioning>,
    /// The range partitioning for this table..
    pub range_partitioning: Option<RangePartitioning>,
    /// Clustering specification for this table, if configured.
    pub clustering: Option<Clustering>,
    /// Output only. The time when this table was created, in milliseconds since the epoch.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub creation_time: i64,
    /// OThe time when this table expires, in milliseconds since the epoch. If not present, the table will persist indefinitely. Expired tables will be deleted and their storage reclaimed.
    #[serde(deserialize_with = "crate::http::from_str_option")]
    #[serde(default)]
    pub expiration_time: Option<i64>,
    /// The type of table.
    #[serde(rename(deserialize = "type"))]
    pub table_type: String,
    /// Additional details for a view.
    pub view: Option<View>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListTablesResponse {
    /// Output only. The resource type. This property always returns the value "bigquery#datasetList"
    pub kind: String,
    /// Output only. A hash value of the results page.
    /// You can use this property to determine if the page has changed since the last request.
    pub etag: String,
    /// An array of the dataset resources in the project.
    /// Each resource contains basic information.
    /// For full information about a particular dataset resource, use the Datasets: get method.
    /// This property is omitted when there are no datasets in the project.
    pub tables: Vec<TableOverview>,
    /// A token that can be used to request the next results page.
    /// This property is omitted on the final results page.
    pub next_page_token: Option<String>,
    /// The total number of tables in the dataset.
    pub total_items: i32,
}

pub fn build(
    base_url: &str,
    client: &Client,
    project_id: &str,
    dataset_id: &str,
    req: &ListTablesRequest,
    page_token: Option<String>,
) -> RequestBuilder {
    let url = format!("{}/projects/{}/datasets/{}/tables", base_url, project_id, dataset_id);
    let builder = client.get(url).query(req).query(req);
    if let Some(page_token) = page_token {
        builder.query(&[("pageToken", page_token.as_str())])
    } else {
        builder
    }
}
