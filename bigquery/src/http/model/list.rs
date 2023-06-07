use crate::http::model::{ModelReference, ModelType};
use reqwest::{Client, RequestBuilder};

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListModelsRequest {
    /// The maximum number of results to return in a single response page.
    /// Leverage the page tokens to iterate through the entire collection.
    pub max_results: Option<i64>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ModelOverview {
    /// Required. Unique identifier for this model.
    pub model_reference: ModelReference,
    /// Output only. The time when this model was created, in millisecs since the epoch.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub creation_time: i64,
    /// Output only. The time when this model was last modified, in millisecs since the epoch.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub last_modified_time: u64,
    /// Output only. Type of the model resource.
    pub model_type: Option<ModelType>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListModelsResponse {
    /// An array of the dataset resources in the project.
    /// Each resource contains basic information.
    /// For full information about a particular dataset resource, use the Datasets: get method.
    /// This property is omitted when there are no datasets in the project.
    pub models: Vec<ModelOverview>,
    /// A token that can be used to request the next results page.
    /// This property is omitted on the final results page.
    pub next_page_token: Option<String>,
}

pub fn build(
    base_url: &str,
    client: &Client,
    project_id: &str,
    dataset_id: &str,
    req: &ListModelsRequest,
    page_token: Option<String>,
) -> RequestBuilder {
    let url = format!("{}/projects/{}/datasets/{}/models", base_url, project_id, dataset_id);
    let builder = client.get(url).query(req).query(req);
    if let Some(page_token) = page_token {
        builder.query(&[("pageToken", page_token.as_str())])
    } else {
        builder
    }
}
