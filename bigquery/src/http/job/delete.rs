use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

pub fn build(base_url: &str, client: &Client, project_id: &str, job_id: &str) -> RequestBuilder {
    let url = format!("{base_url}/projects/{project_id}/jobs/{job_id}");
    client.delete(url)
}
