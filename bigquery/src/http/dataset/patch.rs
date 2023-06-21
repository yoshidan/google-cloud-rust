use reqwest::{Client, RequestBuilder};

use crate::http::dataset::Dataset;

pub fn build(base_url: &str, client: &Client, data: &Dataset) -> RequestBuilder {
    let url = format!(
        "{}/projects/{}/datasets/{}",
        base_url,
        data.dataset_reference.project_id.as_str(),
        data.dataset_reference.dataset_id.as_str()
    );
    let mut builder = client.patch(url);
    if !data.etag.is_empty() {
        builder = builder.header("If-Match", data.etag.as_str())
    }
    builder.json(data)
}
