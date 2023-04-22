use reqwest::{Client, RequestBuilder};

use crate::http::objects::{Encryption, Object};
use crate::http::{object_access_controls::Projection, Escape};
/// Request message for GetObject.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct CopyObjectRequest {
    /// Required. Name of the bucket in which the object resides.
    /// Name of the bucket in which to look for objects.
    #[serde(skip_serializing)]
    pub source_bucket: String,

    pub destination_bucket: String,

    pub destination_object: String,

    pub source_object: String,

    pub if_generation_match: Option<i64>,

    pub if_generation_not_match: Option<i64>,
    pub if_metageneration_match: Option<i64>,
    pub if_metageneration_not_match: Option<i64>,
    pub if_source_generation_match: Option<i64>,
    pub if_source_generation_not_match: Option<i64>,
    pub if_source_metageneration_match: Option<i64>,
    pub if_source_metageneration_not_match: Option<i64>,
    pub projection: Option<Projection>,
    pub source_generation: Option<i64>,
    /// The Object metadata for updating.
    #[serde(skip_serializing)]
    pub metadata: Option<Object>,

    #[serde(skip_serializing)]
    pub encryption: Option<Encryption>,
}

pub(crate) fn build(base_url: &str, client: &Client, req: &CopyObjectRequest) -> RequestBuilder {
    let url = format!(
        "{}/b/{}/o/{}/copyTo/b/{}/o/{}",
        base_url,
        req.source_bucket.escape(),
        req.source_object.escape(),
        req.destination_bucket.escape(),
        req.destination_object.escape()
    );
    let builder = client.post(url).query(&req).json(&req.metadata);
    if let Some(e) = &req.encryption {
        e.with_headers(builder)
    } else {
        builder
    }
}
