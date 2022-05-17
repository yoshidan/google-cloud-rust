


use crate::http::object_access_controls::{PredefinedObjectAcl, Projection};

use crate::http::objects::{Encryption};
use crate::http::{Escape, UPLOAD_BASE_URL};
use reqwest::header::{CONTENT_LENGTH, CONTENT_TYPE};
use reqwest::{Client, RequestBuilder};


#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct UploadObjectRequest {
    #[serde(skip_serializing)]
    pub bucket: String,
    pub name: String,
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

pub(crate) fn build<T: Into<reqwest::Body>>(
    client: &Client,
    req: &UploadObjectRequest,
    content_length: usize,
    content_type: &str,
    body: T,
) -> RequestBuilder {
    let url = format!("{}/b/{}/o", UPLOAD_BASE_URL, req.bucket.escape());
    let builder = client
        .post(url)
        .query(&req)
        .body(body)
        .header(CONTENT_LENGTH, &content_length.to_string())
        .header(CONTENT_TYPE, content_type);
    if let Some(e) = &req.encryption {
        e.with_headers(builder)
    } else {
        builder
    }
}
