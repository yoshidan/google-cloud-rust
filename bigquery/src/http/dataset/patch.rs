use crate::http::dataset::Dataset;
use reqwest::{Client, RequestBuilder};

pub fn build(base_url: &str, client: &Client, project_id: &str, dataset_id: &str, data: &Dataset) -> RequestBuilder {
    let url = format!("{}/projects/{}/datasets/{}", base_url, project_id, dataset_id);
    let mut builder = client.patch(url);
    if !data.etag.is_empty() {
        builder = builder.header("If-Match", data.etag.as_str())
    }
    builder.json(data)
}
