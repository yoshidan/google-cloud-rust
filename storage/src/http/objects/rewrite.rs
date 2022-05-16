use crate::http::bucket_access_controls::PredefinedBucketAcl;
use crate::http::channels::Channel;
use crate::http::notifications::Notification;
use crate::http::object_access_controls::insert::ObjectAccessControlCreationConfig;
use crate::http::object_access_controls::{PredefinedObjectAcl, Projection};
use crate::http::objects::{Encryption, Object};
use crate::http::{Escape, BASE_URL};
use reqwest::{Client, RequestBuilder};
use std::collections::HashMap;

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RewriteObjectRequest {
    #[serde(skip_serializing)]
    pub destination_bucket: String,
    #[serde(skip_serializing)]
    pub destination_object: String,
    #[serde(skip_serializing)]
    pub source_bucket: String,
    #[serde(skip_serializing)]
    pub source_object: String,
    /// If set, only deletes the bucket if its metageneration matches this value.
    pub if_destination_metageneration_match: Option<i64>,
    /// If set, only deletes the bucket if its metageneration does not match this
    /// value.
    pub if_destination_metageneration_not_match: Option<i64>,
    /// If set, only deletes the bucket if its metageneration matches this value.
    pub if_source_metageneration_match: Option<i64>,
    /// If set, only deletes the bucket if its metageneration does not match this
    /// value.
    pub if_source_metageneration_not_match: Option<i64>,
    pub destination_kms_key_name: Option<String>,
    pub destination_predefined_object_acl: Option<PredefinedObjectAcl>,
    pub max_bytes_rewritten_per_call: Option<i64>,
    pub projection: Option<Projection>,
    pub source_generation: Option<i64>,
    pub rewrite_token: Option<String>,
    #[serde(skip_serializing)]
    pub destination_metadata: Option<Object>,
    #[serde(skip_serializing)]
    pub source_encryption: Option<Encryption>,
    #[serde(skip_serializing)]
    pub destination_encryption: Option<Encryption>,
}

/// A rewrite response.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RewriteObjectResponse {
    /// The total bytes written so far, which can be used to provide a waiting user
    /// with a progress indicator. This property is always present in the response.
    pub total_bytes_rewritten: i64,
    /// The total size of the object being copied in bytes. This property is always
    /// present in the response.
    pub object_size: i64,
    /// `true` if the copy is finished; otherwise, `false` if
    /// the copy is in progress. This property is always present in the response.
    pub done: bool,
    /// A token to use in subsequent requests to continue copying data. This token
    /// is present in the response only when there is more data to copy.
    pub rewrite_token: String,
    /// A resource containing the metadata for the copied-to object. This property
    /// is present in the response only when copying completes.
    pub resource: Option<Object>,
}

pub(crate) fn build(client: &Client, req: &RewriteObjectRequest) -> RequestBuilder {
    let url = format!(
        "{}/b/{}/o/{}/rewriteTo/b/{}/o{}",
        BASE_URL,
        req.source_bucket.escape(),
        req.source_object.escape(),
        req.destination_bucket.escape(),
        req.destination_bucket.escape()
    );
    let mut builder = client.post(url).query(&req).json(&req.destination_metadata);
    if let Some(e) = &req.destination_encryption {
        builder = e.with_headers(builder)
    }
    if let Some(e) = &req.source_encryption {
        builder
            .header("X-Goog-Copy-Source-Encryption-Algorithm", &e.encryption_algorithm)
            .header("X-Goog-Copy-Source-Encryption-Key", &e.encryption_key)
            .header("X-Goog-Copy-Source-Encryption-Key-Sha256", &e.encryption_key_sha256)
    } else {
        builder
    }
}
