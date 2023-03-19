use crate::http::dataset::Dataset;
use reqwest::{Client, RequestBuilder};

pub fn build(base_url: &str, client: &Client, project_id: &str, data: &Dataset) -> RequestBuilder {
    let url = format!("{}/projects/{}/datasets", base_url, project_id);
    client.post(url).json(data)
}
