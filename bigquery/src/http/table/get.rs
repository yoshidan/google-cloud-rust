use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

pub fn build(base_url: &str, client: &Client, project_id: &str, dataset_id: &str, table_id: &str) -> RequestBuilder {
    let url = format!(
        "{}/projects/{}/datasets/{}/tables/{}",
        base_url, project_id, dataset_id, table_id
    );
    client.get(url)
}
