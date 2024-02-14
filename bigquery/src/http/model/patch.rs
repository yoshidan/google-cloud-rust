use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::model::Model;

pub fn build(base_url: &str, client: &Client, data: &Model) -> RequestBuilder {
    let url = format!(
        "{}/projects/{}/datasets/{}/models/{}",
        base_url,
        data.model_reference.project_id.as_str(),
        data.model_reference.dataset_id.as_str(),
        data.model_reference.model_id.as_str()
    );
    client.patch(url).json(data)
}
