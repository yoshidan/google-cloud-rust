use crate::http::dataset::Dataset;
use reqwest::{Client, RequestBuilder};

pub fn build(base_url: &str, client: &Client, data: &Dataset) -> RequestBuilder {
    let url = format!("{}/projects/{}/datasets", base_url, data.dataset_reference.project_id);
    client.post(url).json(data)
}
