use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::routine::Routine;

pub fn build(base_url: &str, client: &Client, data: &Routine) -> RequestBuilder {
    let url = format!(
        "{}/projects/{}/datasets/{}/routines",
        base_url, data.routine_reference.project_id, data.routine_reference.dataset_id
    );
    client.post(url).json(data)
}
