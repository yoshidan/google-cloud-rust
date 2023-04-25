use std::collections::HashMap;

use reqwest::RequestBuilder;
use time::OffsetDateTime;

use crate::http::object_access_controls::ObjectAccessControl;

pub mod compose;
pub mod copy;
pub mod delete;
pub mod download;
pub mod get;
pub mod list;
pub mod patch;
pub mod rewrite;
pub mod upload;
pub mod watch_all;

/// An object.
#[derive(Clone, PartialEq, Eq, Default, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Object {
    /// The link to this object.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub self_link: String,
    /// The media link to this object.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub media_link: String,
    /// Content-Encoding of the object data, matching
    /// \[<https://tools.ietf.org/html/rfc7231#section-3.1.2.2\][RFC> 7231 §3.1.2.2]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_encoding: Option<String>,
    /// Content-Disposition of the object data, matching
    /// \[<https://tools.ietf.org/html/rfc6266\][RFC> 6266].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_disposition: Option<String>,
    /// Cache-Control directive for the object data, matching
    /// \[<https://tools.ietf.org/html/rfc7234#section-5.2"\][RFC> 7234 §5.2].
    /// If omitted, and the object is accessible to all anonymous users, the
    /// default will be `public, max-age=3600`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<String>,
    /// Access controls on the object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acl: Option<Vec<ObjectAccessControl>>,
    /// Content-Language of the object data, matching
    /// \[<https://tools.ietf.org/html/rfc7231#section-3.1.3.2\][RFC> 7231 §3.1.3.2].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_language: Option<String>,
    /// The version of the metadata for this object at this generation. Used for
    /// preconditions and for detecting changes in metadata. A metageneration
    /// number is only meaningful in the context of a particular generation of a
    /// particular object.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    #[serde(skip_serializing_if = "crate::http::is_i64_zero")]
    #[serde(deserialize_with = "crate::http::from_str")]
    pub metageneration: i64,
    /// The deletion time of the object. Will be returned if and only if this
    /// version of the object has been deleted.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub time_deleted: Option<OffsetDateTime>,
    /// Content-Type of the object data, matching
    /// \[<https://tools.ietf.org/html/rfc7231#section-3.1.1.5\][RFC> 7231 §3.1.1.5].
    /// If an object is stored without a Content-Type, it is served as
    /// `application/octet-stream`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    /// Content-Length of the object data in bytes, matching
    /// \[<https://tools.ietf.org/html/rfc7230#section-3.3.2\][RFC> 7230 §3.3.2].
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    #[serde(skip_serializing_if = "crate::http::is_i64_zero")]
    #[serde(deserialize_with = "crate::http::from_str")]
    pub size: i64,
    /// The creation time of the object.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub time_created: Option<OffsetDateTime>,
    /// CRC32c checksum. For more information about using the CRC32c
    /// checksum, see
    /// \[<https://cloud.google.com/storage/docs/hashes-etags#json-api\][Hashes> and
    /// ETags: Best Practices]. This is a server determined value and should not be
    /// supplied by the user when sending an Object. The server will ignore any
    /// value provided. Users should instead use the object_checksums field on the
    /// InsertObjectRequest when uploading an object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crc32c: Option<String>,
    /// MD5 hash of the data; encoded using base64 as per
    /// \[<https://tools.ietf.org/html/rfc4648#section-4\][RFC> 4648 §4]. For more
    /// information about using the MD5 hash, see
    /// \[<https://cloud.google.com/storage/docs/hashes-etags#json-api\][Hashes> and
    /// ETags: Best Practices]. This is a server determined value and should not be
    /// supplied by the user when sending an Object. The server will ignore any
    /// value provided. Users should instead use the object_checksums field on the
    /// InsertObjectRequest when uploading an object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub md5_hash: Option<String>,
    /// HTTP 1.1 Entity tag for the object. See
    /// \[<https://tools.ietf.org/html/rfc7232#section-2.3\][RFC> 7232 §2.3].
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    #[serde(skip_serializing_if = "String::is_empty")]
    pub etag: String,
    /// The modification time of the object metadata.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub updated: Option<OffsetDateTime>,
    /// Storage class of the object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_class: Option<String>,
    /// Cloud KMS Key used to encrypt this object, if the object is encrypted by
    /// such a key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kms_key_name: Option<String>,
    /// The time at which the object's storage class was last changed. When the
    /// object is initially created, it will be set to time_created.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub time_storage_class_updated: Option<OffsetDateTime>,
    /// Whether an object is under temporary hold. While this flag is set to true,
    /// the object is protected against deletion and overwrites.  A common use case
    /// of this flag is regulatory investigations where objects need to be retained
    /// while the investigation is ongoing. Note that unlike event-based hold,
    /// temporary hold does not impact retention expiration time of an object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temporary_hold: Option<bool>,
    /// A server-determined value that specifies the earliest time that the
    /// object's retention period expires. This value is in
    /// \[<https://tools.ietf.org/html/rfc3339\][RFC> 3339] format.
    /// Note 1: This field is not provided for objects with an active event-based
    /// hold, since retention expiration is unknown until the hold is removed.
    /// Note 2: This value can be provided even when temporary hold is set (so that
    /// the user can reason about policy without having to first unset the
    /// temporary hold).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub retention_expiration_time: Option<OffsetDateTime>,
    /// User-provided metadata, in key/value pairs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    /// Whether an object is under event-based hold. Event-based hold is a way to
    /// retain objects until an event occurs, which is signified by the
    /// hold's release (i.e. this value is set to false). After being released (set
    /// to false), such objects will be subject to bucket-level retention (if any).
    /// One sample use case of this flag is for banks to hold loan documents for at
    /// least 3 years after loan is paid in full. Here, bucket-level retention is 3
    /// years and the event is the loan being paid in full. In this example, these
    /// objects will be held intact for any number of years until the event has
    /// occurred (event-based hold on the object is released) and then 3 more years
    /// after that. That means retention duration of the objects begins from the
    /// moment event-based hold transitioned from true to false.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_based_hold: Option<bool>,
    /// The name of the object.
    /// Attempting to update this field after the object is created will result in
    /// an error.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// The ID of the object, including the bucket name, object name, and
    /// generation number.
    /// Attempting to update this field after the object is created will result in
    /// an error.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,
    /// The name of the bucket containing this object.
    /// Attempting to update this field after the object is created will result in
    /// an error.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub bucket: String,
    /// The content generation of this object. Used for object versioning.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    #[serde(skip_serializing_if = "crate::http::is_i64_zero")]
    #[serde(deserialize_with = "crate::http::from_str")]
    pub generation: i64,
    /// The owner of the object. This will always be the uploader of the object.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<Owner>,
    /// Metadata of customer-supplied encryption key, if the object is encrypted by
    /// such a key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_encryption: Option<CustomerEncryption>,
    /// A user-specified timestamp set on an object.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub custom_time: Option<OffsetDateTime>,
}

/// Describes the customer-specified mechanism used to store the data at rest.
#[derive(Clone, PartialEq, Eq, Default, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CustomerEncryption {
    /// The encryption algorithm.
    pub encryption_algorithm: String,
    /// SHA256 hash value of the encryption key.
    pub key_sha256: String,
}

/// The owner of a specific resource.
#[derive(Clone, PartialEq, Eq, Default, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Owner {
    /// The entity, in the form `user-`*userId*.
    #[serde(default)]
    pub entity: String,
    /// The ID for the entity.
    pub entity_id: Option<String>,
}

/// Description of a source object for a composition request.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct SourceObjects {
    /// The source object's name. All source objects must reside in the same
    /// bucket.
    pub name: String,
    /// The generation of this object to use as the source.
    pub generation: Option<i64>,
    /// Conditions that must be met for this operation to execute.
    pub object_preconditions: Option<ObjectPreconditions>,
}
/// Preconditions for a source object of a composition request.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ObjectPreconditions {
    /// Only perform the composition if the generation of the source object
    /// that would be used matches this value.  If this value and a generation
    /// are both specified, they must be the same value or the call will fail.
    pub if_generation_match: Option<i64>,
}

/// Parameters that can be passed to any object request.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Encryption {
    /// Encryption algorithm used with Customer-Supplied Encryption Keys feature.
    pub encryption_algorithm: String,
    /// Encryption key used with Customer-Supplied Encryption Keys feature.
    pub encryption_key: String,
    /// SHA256 hash of encryption key used with Customer-Supplied Encryption Keys
    /// feature.
    pub encryption_key_sha256: String,
}

impl Encryption {
    pub(crate) fn with_headers(&self, builder: RequestBuilder) -> RequestBuilder {
        builder
            .header("X-Goog-Encryption-Algorithm", &self.encryption_algorithm)
            .header("X-Goog-Encryption-Key", &self.encryption_key)
            .header("X-Goog-Encryption-Key-Sha256", &self.encryption_key_sha256)
    }
}
