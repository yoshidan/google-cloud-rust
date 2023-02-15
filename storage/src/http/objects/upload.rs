use crate::http::object_access_controls::{PredefinedObjectAcl, Projection};

use crate::http::objects::{Encryption, Object};
use crate::http::Escape;
use reqwest::header::{CONTENT_LENGTH, CONTENT_TYPE};
use reqwest::{Client, RequestBuilder};

pub struct Multipart {
    pub boundary: String,
    pub metadata: Object,
}

pub enum UploadType {
    Simple(String),
    Multipart(Multipart),
}

impl UploadType {
    pub fn content_type(&self) -> String {
        match self {
            UploadType::Simple(v) => v.to_string(),
            UploadType::Multipart(v) => format!("multipart/related; boundary={}", v.boundary.as_str()),
        }
    }

    pub fn upload_type(&self) -> &'static str {
        match self {
            UploadType::Simple(_) => "media",
            UploadType::Multipart(_) => "multipart",
        }
    }

    pub fn data(&self, data: &[u8]) -> Result<Vec<u8>, serde_json::Error> {
        Ok(match self {
            UploadType::Simple(_) => Vec::from(data),
            UploadType::Multipart(metadata) => {
                let data_content_type = metadata
                    .metadata
                    .content_type
                    .clone()
                    .unwrap_or("application/octet-stream".to_string());
                let mut multipart_data = Vec::with_capacity(data.len());
                multipart_data.extend(
                    format!(
                        "--{}\r\nContent-Type: application/json; charset=UTF-8\r\n\r\n",
                        metadata.boundary
                    )
                    .into_bytes(),
                );
                multipart_data.extend(serde_json::to_vec(&metadata.metadata)?);
                multipart_data.extend(
                    format!("\r\n\r\n--{}\r\nContent-Type: {}\r\n\r\n", metadata.boundary, data_content_type)
                        .into_bytes(),
                );
                multipart_data.extend_from_slice(data);
                multipart_data.extend(format!("\r\n--{}--\r\n", metadata.boundary).into_bytes());
                let v = String::from_utf8_lossy(multipart_data.as_slice().clone()).to_string();
                tracing::info!("\n{:?}", v);
                multipart_data
            }
        })
    }
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct UploadObjectRequest {
    /// Name of the bucket in which to store the new object.
    /// Overrides the provided object metadata's bucket value, if any.
    #[serde(skip_serializing)]
    pub bucket: String,
    /// Name of the object. Not required if the request body contains object metadata
    /// that includes a name value. Overrides the object metadata's name value, if any.
    /// For information about how to URL encode object names to be path safe, see Encoding URI path parts.
    pub name: Option<String>,
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
    content_length: Option<usize>,
    upload_type: UploadType,
    body: T,
) -> RequestBuilder {
    let url = format!(
        "{}/b/{}/o?uploadType={}",
        base_url,
        req.bucket.escape(),
        upload_type.upload_type()
    );
    let mut builder = client
        .post(url)
        .query(&req)
        .body(body)
        .header(CONTENT_TYPE, upload_type.content_type());

    if let Some(len) = content_length {
        builder = builder.header(CONTENT_LENGTH, len.to_string())
    }
    if let Some(e) = &req.encryption {
        e.with_headers(builder)
    } else {
        builder
    }
}
