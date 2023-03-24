use reqwest::{Client, RequestBuilder};

pub fn build(base_url: &str, client: &Client, project_id: &str, dataset_id: &str) -> RequestBuilder {
    let url = format!("{}/projects/{}/datasets/{}", base_url, project_id, dataset_id);
    client.get(url)
}
