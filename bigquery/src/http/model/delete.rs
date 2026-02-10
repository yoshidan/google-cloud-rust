use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

pub fn build(base_url: &str, client: &Client, project_id: &str, dataset_id: &str, model_id: &str) -> RequestBuilder {
    let url = format!("{base_url}/projects/{project_id}/datasets/{dataset_id}/models/{model_id}");
    client.delete(url)
}
