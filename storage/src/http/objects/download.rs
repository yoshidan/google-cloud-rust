use crate::http::objects::get::GetObjectRequest;

use crate::http::{Escape, BASE_URL};
use reqwest::{Client, RequestBuilder};

pub(crate) fn build(client: &Client, req: &GetObjectRequest) -> RequestBuilder {
    let url = format!("{}/b/{}/o/{}?alt=media", BASE_URL, req.bucket.escape(), req.object.escape());
    let builder = client.get(url).query(&req);
    if let Some(e) = &req.encryption {
        e.with_headers(builder)
    } else {
        builder
    }
}
