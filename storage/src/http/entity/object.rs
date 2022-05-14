use crate::http::entity::acl::{ObjectAccessControl};
use crate::http::entity::common::{PredefinedBucketAcl, PredefinedObjectAcl, Projection};
use crate::http::entity::{MaxResults, PageToken, Prefix};

/// Message for deleting an object.
/// Either `bucket` and `object` *or* `upload_id` **must** be set (but not both).
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteObjectRequest {
    /// Required. Name of the bucket in which the object resides.
    pub bucket: String,
    /// Required. The name of the object to delete (when not using a resumable write).
    pub object: String,
    /// If present, permanently deletes a specific revision of this object (as
    /// opposed to the latest version, the default).
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
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertSimpleObjectRequest {
    pub bucket: String,
    pub object: String,
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
    pub body: Vec<u8>
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PatchObjectRequest {
    pub bucket: String,
    pub object: String,
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
    pub predefined_acl: Option<PredefinedObjectAcl>,
    pub projection: Option<Projection>,
    pub resource: Object,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListObjectsRequest {
    pub bucket: String,
    pub delimiter: Option<String>,
    pub end_offset: Option<String>,
    pub include_trailing_delimiter: Option<bool>,
    pub max_results: Option<i32>,
    pub page_token: Option<String>,
    pub prefix: Option<String>,
    pub projection: Option<Projection>,
    pub start_offset: Option<String>,
    pub versions: Option<bool>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RewriteObjectRequest {
    pub destination_bucket: String,
    pub destination_object: String,
    pub source_bucket: String,
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
}

/// Request message for ComposeObject.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ComposeObjectRequest {
    /// Required. Name of the bucket containing the source objects. The destination object is
    /// stored in this bucket.
    pub bucket: String,
    /// Required. Name of the new object.
    pub destination_object: String,
    /// Apply a predefined set of access controls to the destination object.
    pub destination_predefined_acl: Option<PredefinedObjectAcl>,
    /// Properties of the resulting object.
    pub destination: Option<Object>,
    /// The list of source objects that will be concatenated into a single object.
    pub source_objects: Vec<SourceObjects>,
    /// Makes the operation conditional on whether the object's current generation
    /// matches the given value. Setting to 0 makes the operation succeed only if
    /// there are no live versions of the object.
    pub if_generation_match: Option<i64>,
    /// Makes the operation conditional on whether the object's current
    /// metageneration matches the given value.
    pub if_metageneration_match: Option<i64>,
    /// Resource name of the Cloud KMS key, of the form
    /// `projects/my-project/locations/my-location/keyRings/my-kr/cryptoKeys/my-key`,
    /// that will be used to encrypt the object. Overrides the object
    /// metadata's `kms_key_name` value, if any.
    pub kms_key_name: String,
}

/// The result of a call to Objects.ListObjects
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListObjectsResponse {
    /// The list of prefixes of objects matching-but-not-listed up to and including
    /// the requested delimiter.
    pub prefixes: Vec<String>,
    /// The list of items.
    pub items: Vec<Object>,
    /// The continuation token, used to page through large result sets. Provide
    /// this value in a subsequent request to return the next page of results.
    pub next_page_token: String,
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

/// An object.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Object {
    /// Content-Encoding of the object data, matching
    /// \[<https://tools.ietf.org/html/rfc7231#section-3.1.2.2\][RFC> 7231 §3.1.2.2]
    pub content_encoding: String,
    /// Content-Disposition of the object data, matching
    /// \[<https://tools.ietf.org/html/rfc6266\][RFC> 6266].
    pub content_disposition: String,
    /// Cache-Control directive for the object data, matching
    /// \[<https://tools.ietf.org/html/rfc7234#section-5.2"\][RFC> 7234 §5.2].
    /// If omitted, and the object is accessible to all anonymous users, the
    /// default will be `public, max-age=3600`.
    pub cache_control: String,
    /// Access controls on the object.
    pub acl: Vec<ObjectAccessControl>,
    /// Content-Language of the object data, matching
    /// \[<https://tools.ietf.org/html/rfc7231#section-3.1.3.2\][RFC> 7231 §3.1.3.2].
    pub content_language: String,
    /// The version of the metadata for this object at this generation. Used for
    /// preconditions and for detecting changes in metadata. A metageneration
    /// number is only meaningful in the context of a particular generation of a
    /// particular object.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    pub metageneration: i64,
    /// The deletion time of the object. Will be returned if and only if this
    /// version of the object has been deleted.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    pub time_deleted: Option<chrono::DateTime<chrono::Utc>>,
    /// Content-Type of the object data, matching
    /// \[<https://tools.ietf.org/html/rfc7231#section-3.1.1.5\][RFC> 7231 §3.1.1.5].
    /// If an object is stored without a Content-Type, it is served as
    /// `application/octet-stream`.
    pub content_type: String,
    /// Content-Length of the object data in bytes, matching
    /// \[<https://tools.ietf.org/html/rfc7230#section-3.3.2\][RFC> 7230 §3.3.2].
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    pub size: i64,
    /// The creation time of the object.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    pub time_created: Option<chrono::DateTime<chrono::Utc>>,
    /// CRC32c checksum. For more information about using the CRC32c
    /// checksum, see
    /// \[<https://cloud.google.com/storage/docs/hashes-etags#json-api\][Hashes> and
    /// ETags: Best Practices]. This is a server determined value and should not be
    /// supplied by the user when sending an Object. The server will ignore any
    /// value provided. Users should instead use the object_checksums field on the
    /// InsertObjectRequest when uploading an object.
    pub crc32c: Option<u32>,
    /// Number of underlying components that make up this object. Components are
    /// accumulated by compose operations.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    pub component_count: i32,
    /// MD5 hash of the data; encoded using base64 as per
    /// \[<https://tools.ietf.org/html/rfc4648#section-4\][RFC> 4648 §4]. For more
    /// information about using the MD5 hash, see
    /// \[<https://cloud.google.com/storage/docs/hashes-etags#json-api\][Hashes> and
    /// ETags: Best Practices]. This is a server determined value and should not be
    /// supplied by the user when sending an Object. The server will ignore any
    /// value provided. Users should instead use the object_checksums field on the
    /// InsertObjectRequest when uploading an object.
    pub md5_hash: String,
    /// HTTP 1.1 Entity tag for the object. See
    /// \[<https://tools.ietf.org/html/rfc7232#section-2.3\][RFC> 7232 §2.3].
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    pub etag: String,
    /// The modification time of the object metadata.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    pub updated: Option<chrono::DateTime<chrono::Utc>>,
    /// Storage class of the object.
    pub storage_class: String,
    /// Cloud KMS Key used to encrypt this object, if the object is encrypted by
    /// such a key.
    pub kms_key_name: String,
    /// The time at which the object's storage class was last changed. When the
    /// object is initially created, it will be set to time_created.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    pub time_storage_class_updated: Option<chrono::DateTime<chrono::Utc>>,
    /// Whether an object is under temporary hold. While this flag is set to true,
    /// the object is protected against deletion and overwrites.  A common use case
    /// of this flag is regulatory investigations where objects need to be retained
    /// while the investigation is ongoing. Note that unlike event-based hold,
    /// temporary hold does not impact retention expiration time of an object.
    pub temporary_hold: bool,
    /// A server-determined value that specifies the earliest time that the
    /// object's retention period expires. This value is in
    /// \[<https://tools.ietf.org/html/rfc3339\][RFC> 3339] format.
    /// Note 1: This field is not provided for objects with an active event-based
    /// hold, since retention expiration is unknown until the hold is removed.
    /// Note 2: This value can be provided even when temporary hold is set (so that
    /// the user can reason about policy without having to first unset the
    /// temporary hold).
    pub retention_expiration_time: Option<chrono::DateTime<chrono::Utc>>,
    /// User-provided metadata, in key/value pairs.
    pub metadata: ::std::collections::HashMap<String, String>,
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
    pub event_based_hold: Option<bool>,
    /// The name of the object.
    /// Attempting to update this field after the object is created will result in
    /// an error.
    pub name: String,
    /// The ID of the object, including the bucket name, object name, and
    /// generation number.
    /// Attempting to update this field after the object is created will result in
    /// an error.
    pub id: String,
    /// The name of the bucket containing this object.
    /// Attempting to update this field after the object is created will result in
    /// an error.
    pub bucket: String,
    /// The content generation of this object. Used for object versioning.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    pub generation: i64,
    /// The owner of the object. This will always be the uploader of the object.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    pub owner: Option<Owner>,
    /// Metadata of customer-supplied encryption key, if the object is encrypted by
    /// such a key.
    pub customer_encryption: Option<object::CustomerEncryption>,
    /// A user-specified timestamp set on an object.
    pub custom_time: Option<chrono::DateTime<chrono::Utc>>,
}
/// Nested message and enum types in `Object`.
pub mod object {
    /// Describes the customer-specified mechanism used to store the data at rest.
    #[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct CustomerEncryption {
        /// The encryption algorithm.
        pub encryption_algorithm: String,
        /// SHA256 hash value of the encryption key.
        pub key_sha256: String,
    }
}

/// The owner of a specific resource.
#[derive(Clone, PartialEq, Default, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Owner {
    /// The entity, in the form `user-`*userId*.
    #[serde(default)]
    pub entity: String,
    /// The ID for the entity.
    pub entity_id: Option<String>,
}

/// Description of a source object for a composition request.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SourceObjects {
    /// The source object's name. All source objects must reside in the same
    /// bucket.
    pub name: String,
    /// The generation of this object to use as the source.
    pub generation: Generation,
    /// Conditions that must be met for this operation to execute.
    pub object_preconditions: Option<source_objects::ObjectPreconditions>,
}
/// Nested message and enum types in `SourceObjects`.
pub mod source_objects {
    /// Preconditions for a source object of a composition request.
    #[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct ObjectPreconditions {
        /// Only perform the composition if the generation of the source object
        /// that would be used matches this value.  If this value and a generation
        /// are both specified, they must be the same value or the call will fail.
        pub if_generation_match: Option<i64>,
    }
}