use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

pub fn build(base_url: &str, client: &Client, project_id: &str, dataset_id: &str, model_id: &str) -> RequestBuilder {
    let url = format!(
        "{}/projects/{}/datasets/{}/models/{}",
        base_url, project_id, dataset_id, model_id
    );
    client.delete(url)
}
