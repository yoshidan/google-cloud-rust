use crate::http::channels::Channel;
use crate::http::notifications::Notification;
use crate::http::object_access_controls::insert::ObjectAccessControlCreationConfig;
use crate::http::object_access_controls::{PredefinedObjectAcl, Projection};
use crate::http::objects::get::GetObjectRequest;
use crate::http::objects::{Encryption, Object};
use crate::http::{Escape, BASE_URL};
use reqwest::{Client, RequestBuilder};
use std::collections::HashMap;

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UploadObjectRequest {
    #[serde(skip_serializing)]
    pub bucket: String,
    #[serde(skip_serializing)]
    pub name: String,
    #[serde(skip_serializing)]
    pub generation: Option<i64>,
    /// Makes the operation conditional on whether the object's current generation
    /// matches the given value. Setting to 0 makes the operation succeed only if
    /// there are no live versions of the object.
    pub if_generation_match: Option<i64>,
    /// Makes the operation conditional on whether the object's current generation
    /// does not match the given value. If no live object exists, the precondition
    /// fails. Setting to 0 makes the operation succeed only if there is a live
    /// version of the object.
    pub if_generation_not_match: Option<i64>,
    /// Makes the operation conditional on whether the object's current
    /// metageneration matches the given value.
    pub if_metageneration_match: Option<i64>,
    /// Makes the operation conditional on whether the object's current
    /// metageneration does not match the given value.
    pub if_metageneration_not_match: Option<i64>,
    pub content_encoding: Option<String>,
    pub kms_key_name: Option<String>,
    pub predefined_acl: Option<PredefinedObjectAcl>,
    pub projection: Option<Projection>,
    #[serde(skip_serializing)]
    pub encryption: Option<Encryption>,
}

pub(crate) fn build(client: &Client, req: &UploadObjectRequest, body: Vec<u8>) -> RequestBuilder {
    let url = format!("{}/b/{}/o", BASE_URL, req.bucket.escape());
    let mut builder = client.post(url).query(&req).body(body);
    if let Some(e) = &req.encryption {
        e.with_headers(builder)
    } else {
        builder
    }
}
