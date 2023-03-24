use crate::http::table::Table;
use reqwest::{Client, RequestBuilder};

pub fn build(base_url: &str, client: &Client, data: &Table) -> RequestBuilder {
    let url = format!(
        "{}/projects/{}/datasets/{}/tables",
        base_url,
        data.table_reference.project_id.as_str(),
        data.table_reference.dataset_id.as_str()
    );
    client.post(url).json(data)
}
