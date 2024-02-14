use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::job::Job;

pub fn build(base_url: &str, client: &Client, data: &Job) -> RequestBuilder {
    let url = format!("{}/projects/{}/jobs", base_url, data.job_reference.project_id);
    client.post(url).json(data)
}
