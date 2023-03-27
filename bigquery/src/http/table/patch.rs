use crate::http::table::Table;
use reqwest::{Client, RequestBuilder};

pub fn build(base_url: &str, client: &Client, data: &Table) -> RequestBuilder {
    let url = format!(
        "{}/projects/{}/datasets/{}/tables/{}",
        base_url,
        data.table_reference.project_id.as_str(),
        data.table_reference.dataset_id.as_str(),
        data.table_reference.table_id.as_str()
    );
    let mut builder = client.patch(url);
    if !data.etag.is_empty() {
        builder = builder.header("If-Match", data.etag.as_str())
    }
    builder.json(data)
}
