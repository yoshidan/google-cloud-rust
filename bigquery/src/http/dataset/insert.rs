use reqwest::{Client, RequestBuilder};

use crate::http::dataset::Dataset;

pub fn build(base_url: &str, client: &Client, data: &Dataset) -> RequestBuilder {
    let url = format!("{}/projects/{}/datasets", base_url, data.dataset_reference.project_id);
    client.post(url).json(data)
}
