use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::routine::Routine;

pub fn build(base_url: &str, client: &Client, data: &Routine) -> RequestBuilder {
    let url = format!(
        "{}/projects/{}/datasets/{}/routines/{}",
        base_url,
        data.routine_reference.project_id.as_str(),
        data.routine_reference.dataset_id.as_str(),
        data.routine_reference.routine_id.as_str()
    );
    client.put(url).json(data)
}
