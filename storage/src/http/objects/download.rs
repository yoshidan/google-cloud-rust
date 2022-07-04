use crate::http::objects::get::GetObjectRequest;

use crate::http::Escape;
use reqwest::{Client, RequestBuilder};

pub(crate) fn build(base_url: &str, client: &Client, req: &GetObjectRequest) -> RequestBuilder {
    let url = format!("{}/b/{}/o/{}?alt=media", base_url, req.bucket.escape(), req.object.escape());
    let builder = client.get(url).query(&req);
    if let Some(e) = &req.encryption {
        e.with_headers(builder)
    } else {
        builder
    }
}
