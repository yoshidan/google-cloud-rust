use crate::http::dataset::DatasetReference;
use reqwest::{Client, RequestBuilder};
use std::collections::HashMap;

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListDatasetsRequest {
    /// The maximum number of results to return in a single response page.
    /// Leverage the page tokens to iterate through the entire collection.
    pub max_results: Option<i32>,
    /// Whether to list all datasets, including hidden ones.
    pub all: bool,
    /// An expression for filtering the results of the request by label.
    /// The syntax is "labels.<name>[:<value>]".
    /// Multiple filters can be ANDed together by connecting with a space.
    /// Example: "labels.department:receiving labels.active".
    /// See Filtering datasets using labels for details.
    pub filter: String,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DatasetOverview {
    /// The resource type.
    /// This property always returns the value "bigquery#dataset"
    pub kind: String,
    /// The fully-qualified, unique, opaque ID of the dataset.
    pub id: String,
    /// The dataset reference.
    /// Use this property to access specific parts of the dataset's ID, such as project ID or dataset ID.
    pub dataset_reference: DatasetReference,
    /// The labels associated with this dataset. You can use these to organize and group your datasets.
    /// An object containing a list of "key": value pairs. Example: { "name": "wrench", "mass": "1.3kg", "count": "3" }.
    pub labels: Option<HashMap<String, String>>,
    /// An alternate name for the dataset. The friendly name is purely decorative in nature.
    pub friendly_name: Option<String>,
    /// The geographic location where the dataset resides.
    pub location: Option<String>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListDatasetsResponse {
    /// Output only. The resource type. This property always returns the value "bigquery#datasetList"
    pub kind: String,
    /// Output only. A hash value of the results page.
    /// You can use this property to determine if the page has changed since the last request.
    pub etag: String,
    /// An array of the dataset resources in the project.
    /// Each resource contains basic information.
    /// For full information about a particular dataset resource, use the Datasets: get method.
    /// This property is omitted when there are no datasets in the project.
    pub datasets: Vec<DatasetOverview>,
    /// A token that can be used to request the next results page.
    /// This property is omitted on the final results page.
    pub next_page_token: Option<String>,
}

pub fn build(
    base_url: &str,
    client: &Client,
    project_id: &str,
    req: Option<&ListDatasetsRequest>,
    page_token: Option<String>,
) -> RequestBuilder {
    let url = format!("{}/projects/{}/datasets", base_url, project_id);
    let mut builder = client.get(url);
    builder = if let Some(req) = req {
        builder.query(req)
    } else {
        builder
    };
    if let Some(page_token) = page_token {
        builder.query(&[("pageToken", page_token.as_str())])
    } else {
        builder
    }
}
