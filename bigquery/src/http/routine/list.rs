use crate::http::routine::Routine;
use reqwest::{Client, RequestBuilder};

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListRoutinesRequest {
    /// The maximum number of results to return in a single response page.
    /// Leverage the page tokens to iterate through the entire collection.
    pub max_results: Option<i64>,
    /// If set, then only the Routine fields in the field mask, as well as projectId, datasetId and routineId, are returned in the response. If unset, then the following Routine fields are returned: etag, projectId, datasetId, routineId, routineType, creationTime, lastModifiedTime, and language.
    /// This is a comma-separated list of fully qualified names of fields. Example: "user.displayName,photo".
    pub read_mask: Option<String>,
    /// If set, then only the Routines matching this filter are returned.
    /// The current supported form is either "routineType:" or "routineType:",
    /// where is a RoutineType enum. Example: "routineType:SCALAR_FUNCTION".
    pub filter: Option<String>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListRoutinesResponse {
    /// Routines in the requested dataset. Unless readMask is set in the request,
    /// only the following fields are populated: etag, projectId, datasetId, routineId, routineType,
    /// creationTime, lastModifiedTime, language, and remoteFunctionOptions.
    pub routines: Vec<Routine>,
    /// A token to request the next page of results.
    pub next_page_token: Option<String>,
}

pub fn build(
    base_url: &str,
    client: &Client,
    project_id: &str,
    dataset_id: &str,
    data: &ListRoutinesRequest,
    page_token: Option<String>,
) -> RequestBuilder {
    let url = format!("{}/projects/{}/datasets/{}/routines", base_url, project_id, dataset_id);
    let builder = client.get(url).query(data);
    if let Some(page_token) = page_token {
        builder.query(&[("pageToken", page_token.as_str())])
    } else {
        builder
    }
}
