use std::borrow::Cow;

use reqwest::header::{CONTENT_LENGTH, CONTENT_TYPE};
use reqwest::multipart::{Form, Part};
use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::object_access_controls::{PredefinedObjectAcl, Projection};
use crate::http::objects::{Encryption, Object};
use crate::http::{Error, Escape};

#[derive(Clone, Debug)]
pub struct Media {
    pub name: Cow<'static, str>,
    pub content_type: Cow<'static, str>,
    pub content_length: Option<u64>,
}

impl Media {
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            name: name.into(),
            content_type: "application/octet-stream".into(),
            content_length: None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum UploadType {
    Simple(Media),
    Multipart(Box<Object>),
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct UploadObjectRequest {
    /// Name of the bucket in which to store the new object.
    /// Overrides the provided object metadata's bucket value, if any.
    #[serde(skip_serializing)]
    pub bucket: String,
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
    /// Resource name of the Cloud KMS key that will be used to encrypt the object.
    /// If not specified, the request uses the bucket's default Cloud KMS key, if any,
    /// or a Google-managed encryption key.
    pub kms_key_name: Option<String>,
    ///Apply a predefined set of access controls to this object.
    /// Acceptable values are:
    /// authenticatedRead: Object owner gets OWNER access, and allAuthenticatedUsers get READER access.
    /// bucketOwnerFullControl: Object owner gets OWNER access, and project team owners get OWNER access.
    /// bucketOwnerRead: Object owner gets OWNER access, and project team owners get READER access.
    /// private: Object owner gets OWNER access.
    /// projectPrivate: Object owner gets OWNER access, and project team members get access according to their roles.
    /// publicRead: Object owner gets OWNER access, and allUsers get READER access.
    /// If iamConfiguration.uniformBucketLevelAccess.enabled is set to true,
    /// requests that include this parameter fail with a 400 Bad Request response.
    pub predefined_acl: Option<PredefinedObjectAcl>,
    /// Set of properties to return. Defaults to noAcl,
    /// unless the object resource specifies the acl property, when it defaults to full.
    /// Acceptable values are:
    /// full: Include all properties.
    /// noAcl: Omit the owner, acl property.
    pub projection: Option<Projection>,
    #[serde(skip_serializing)]
    pub encryption: Option<Encryption>,
}

pub(crate) fn build<T: Into<reqwest::Body>>(
    base_url: &str,
    client: &Client,
    req: &UploadObjectRequest,
    media: &Media,
    body: T,
) -> RequestBuilder {
    let url = format!("{}/b/{}/o?uploadType=media", base_url, req.bucket.escape(),);
    let mut builder = client
        .post(url)
        .query(&req)
        .query(&[("name", &media.name)])
        .body(body)
        .header(CONTENT_TYPE, media.content_type.to_string());

    if let Some(len) = media.content_length {
        builder = builder.header(CONTENT_LENGTH, len.to_string())
    }
    if let Some(e) = &req.encryption {
        e.with_headers(builder)
    } else {
        builder
    }
}

pub(crate) fn build_multipart<T: Into<reqwest::Body>>(
    base_url: &str,
    client: &Client,
    req: &UploadObjectRequest,
    metadata: &Object,
    body: T,
) -> Result<RequestBuilder, Error> {
    let url = format!("{}/b/{}/o?uploadType=multipart", base_url, req.bucket.escape(),);
    let form = Form::new();
    let metadata_part = Part::text(serde_json::to_string(metadata).expect("object serialize failed"))
        .mime_str("application/json; charset=UTF-8")?;
    let data_part = Part::stream(body);
    let form = form.part("metadata", metadata_part).part("data", data_part);

    // Content-Length is automatically set by multipart
    let builder = client.post(url).query(&req).multipart(form);

    Ok(if let Some(e) = &req.encryption {
        e.with_headers(builder)
    } else {
        builder
    })
}

pub(crate) fn build_resumable_session_simple(
    base_url: &str,
    client: &Client,
    req: &UploadObjectRequest,
    media: &Media,
) -> RequestBuilder {
    let url = format!("{}/b/{}/o?uploadType=resumable", base_url, req.bucket.escape(),);
    let mut builder = client
        .post(url)
        .query(&req)
        .query(&[("name", &media.name)])
        .header(CONTENT_LENGTH, 0)
        .header("X-Upload-Content-Type", media.content_type.to_string());
    if let Some(len) = media.content_length {
        builder = builder.header("X-Upload-Content-Length", len)
    }
    if let Some(e) = &req.encryption {
        e.with_headers(builder)
    } else {
        builder
    }
}

pub(crate) fn build_resumable_session_metadata(
    base_url: &str,
    client: &Client,
    req: &UploadObjectRequest,
    metadata: &Object,
) -> RequestBuilder {
    let url = format!("{}/b/{}/o?uploadType=resumable", base_url, req.bucket.escape(),);
    let builder = client.post(url).query(&req).json(&metadata);
    if let Some(e) = &req.encryption {
        e.with_headers(builder)
    } else {
        builder
    }
}
