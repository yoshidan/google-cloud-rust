use crate::http::channels::Channel;
use crate::http::notifications::Notification;
use crate::http::object_access_controls::insert::ObjectAccessControlCreationConfig;
use crate::http::object_access_controls::Projection;
use crate::http::objects::get::GetObjectRequest;
use crate::http::objects::{Encryption, Object};
use crate::http::{Escape, BASE_URL};
use reqwest::{Client, RequestBuilder};
use std::collections::HashMap;

pub(crate) fn build(client: &Client, req: &GetObjectRequest) -> RequestBuilder {
    let url = format!("{}/b/{}/o/{}?alt=media", BASE_URL, req.bucket.escape(), req.object.escape());
    let mut builder = client.get(url).query(&req);
    if let Some(e) = &req.encryption {
        e.with_headers(builder)
    } else {
        builder
    }
}
