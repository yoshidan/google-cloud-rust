use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

pub fn build(base_url: &str, client: &Client, project_id: &str, job_id: &str) -> RequestBuilder {
    let url = format!("{}/projects/{}/jobs/{}", base_url, project_id, job_id);
    client.delete(url)
}
