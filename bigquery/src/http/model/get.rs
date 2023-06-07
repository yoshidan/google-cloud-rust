use reqwest::{Client, RequestBuilder};
use crate::http::model::Model;

use crate::http::table::{Clustering, RangePartitioning, TableReference, TimePartitioning};

pub fn build(
    base_url: &str,
    client: &Client,
    project_id: &str,
    dataset_id: &str,
    model_id: &str,
) -> RequestBuilder {
    let url = format!("{}/projects/{}/datasets/{}/models/{}", base_url, project_id, dataset_id, model_id);
    client.get(url)
}
