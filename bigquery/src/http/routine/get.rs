use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

pub fn build(base_url: &str, client: &Client, project_id: &str, dataset_id: &str, routine_id: &str) -> RequestBuilder {
    let url = format!("{base_url}/projects/{project_id}/datasets/{dataset_id}/routines/{routine_id}");
    client.get(url)
}
