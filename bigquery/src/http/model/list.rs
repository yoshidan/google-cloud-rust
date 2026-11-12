use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::model::{ModelReference, ModelType};

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
    /// Models in the requested dataset.
    /// This property is omitted when there are no models in the dataset.
    #[serde(default)]
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
    let url = format!("{base_url}/projects/{project_id}/datasets/{dataset_id}/models");
    let builder = client.get(url).query(req);
    if let Some(page_token) = page_token {
        builder.query(&[("pageToken", page_token.as_str())])
    } else {
        builder
    }
}

#[cfg(test)]
mod test {
    use crate::http::model::list::{build, ListModelsRequest, ListModelsResponse};

    #[test]
    fn test_deserialize_empty_response() {
        let response: ListModelsResponse = serde_json::from_str("{}").unwrap();
        assert!(response.models.is_empty());
    }

    #[test]
    fn test_build_query() {
        let client = reqwest_middleware::ClientBuilder::new(reqwest::Client::new()).build();
        let req = ListModelsRequest { max_results: Some(10) };
        let request = build(
            "https://localhost",
            &client,
            "project",
            "dataset",
            &req,
            Some("token".to_string()),
        )
        .build()
        .unwrap();
        assert_eq!(request.url().query(), Some("maxResults=10&pageToken=token"));
    }
}
