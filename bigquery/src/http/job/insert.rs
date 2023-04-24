use crate::http::job::Job;
use reqwest::{Client, RequestBuilder};

pub fn build(base_url: &str, client: &Client, data: &Job) -> RequestBuilder {
    let url = format!("{}/projects/{}/jobs", base_url, data.job_reference.project_id);
    println!("{:?}",serde_json::to_string(data));
    client.post(url).json(data)
}
