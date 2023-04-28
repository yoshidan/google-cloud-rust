use crate::http::routine::{Language, RemoteFunctionOptions, Routine, RoutineReference, RoutineType};
use reqwest::{Client, RequestBuilder};

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListRoutinesRequest {
    /// The maximum number of results to return in a single response page.
    /// Leverage the page tokens to iterate through the entire collection.
    pub max_results: Option<i64>,
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
    pub routines: Vec<RoutineOverview>,
    /// A token to request the next page of results.
    pub next_page_token: Option<String>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RoutineOverview {
    /// A hash of this resource.
    pub etag: String,
    /// Reference describing the ID of this routine.
    pub routine_reference: RoutineReference,
    /// The type of routine.
    pub routine_type: RoutineType,
    /// The time when this routine was created, in milliseconds since the epoch.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub creation_time: Option<i64>,
    /// The time when this routine was last modified, in milliseconds since the epoch.
    #[serde(default, deserialize_with = "crate::http::from_str_option")]
    pub last_modified_time: Option<i64>,
    /// Defaults to "SQL" if remoteFunctionOptions field is absent, not set otherwise.
    pub language: Option<Language>,
    /// Remote function specific options.
    pub remote_function_options: Option<RemoteFunctionOptions>,
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
