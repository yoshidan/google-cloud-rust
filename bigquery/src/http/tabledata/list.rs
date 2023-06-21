use std::fmt::Debug;

use reqwest::{Client, RequestBuilder};

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(untagged)]
pub enum Value {
    Null,
    String(String),
    Array(Vec<Cell>),
    Struct(Tuple),
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Cell {
    pub v: Value,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Tuple {
    pub f: Vec<Cell>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct FetchDataRequest {
    /// Start row index of the table.
    pub start_index: Option<i32>,
    /// Row limit of the table.
    pub max_results: Option<u32>,
    ///To retrieve the next page of table data, set
    /// this field to the string provided in the pageToken field of the response body from
    /// your previous call to tabledata.list.
    pub page_token: Option<String>,
    /// Subset of fields to return, supports select into sub fields. Example: selectedFields = "a,e.d.f";
    pub selected_fields: Option<String>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FetchDataResponse {
    /// Will be set to "bigquery#tableDataList".
    pub kind: String,
    /// Etag to the response.
    pub etag: String,
    /// Total rows of the entire table. In order to show default value "0", we have to present it as string.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub total_rows: u64,
    /// When this field is non-empty, it indicates that additional results are available.
    /// To request the next page of data, set the pageToken field of your next tabledata.
    /// list call to the string returned in this field.
    pub page_token: Option<String>,
    /// Repeated rows as result. The REST-based representation of this data leverages a series of JSON f,v objects for indicating fields and values.
    pub rows: Option<Vec<Tuple>>,
}

pub fn build(
    base_url: &str,
    client: &Client,
    project_id: &str,
    dataset_id: &str,
    table_id: &str,
    data: &FetchDataRequest,
) -> RequestBuilder {
    let url = format!(
        "{}/projects/{}/datasets/{}/tables/{}/data",
        base_url, project_id, dataset_id, table_id
    );
    client.get(url).query(&data)
}
