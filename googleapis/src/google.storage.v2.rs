/// Request message for DeleteBucket.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteBucketRequest {
    /// Required. Name of a BUCKET to delete.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// If set, only deletes the BUCKET if its metageneration matches this value.
    #[prost(int64, optional, tag = "2")]
    pub if_metageneration_match: ::core::option::Option<i64>,
    /// If set, only deletes the BUCKET if its metageneration does not match this
    /// value.
    #[prost(int64, optional, tag = "3")]
    pub if_metageneration_not_match: ::core::option::Option<i64>,
    /// A set of parameters common to all Storage API requests.
    #[prost(message, optional, tag = "5")]
    pub common_request_params: ::core::option::Option<CommonRequestParams>,
}
/// Request message for GetBucket.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetBucketRequest {
    /// Required. Name of a BUCKET.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// If set, and if the BUCKET's current metageneration does not match the
    /// specified value, the request will return an error.
    #[prost(int64, optional, tag = "2")]
    pub if_metageneration_match: ::core::option::Option<i64>,
    /// If set, and if the BUCKET's current metageneration matches the specified
    /// value, the request will return an error.
    #[prost(int64, optional, tag = "3")]
    pub if_metageneration_not_match: ::core::option::Option<i64>,
    /// A set of parameters common to all Storage API requests.
    #[prost(message, optional, tag = "4")]
    pub common_request_params: ::core::option::Option<CommonRequestParams>,
    /// Mask specifying which fields to read.
    /// A "*" field may be used to indicate all fields.
    /// If no mask is specified, will default to all fields.
    #[prost(message, optional, tag = "5")]
    pub read_mask: ::core::option::Option<::prost_types::FieldMask>,
}
/// Request message for CreateBucket.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateBucketRequest {
    /// Required. The project to which this BUCKET will belong.
    #[prost(string, tag = "1")]
    pub parent: ::prost::alloc::string::String,
    /// Required. Properties of the new BUCKET being inserted.
    /// The project and name of the BUCKET are specified in this request, not in
    /// the BUCKET resource.
    #[prost(message, optional, tag = "2")]
    pub bucket: ::core::option::Option<Bucket>,
    /// Required. The ID to use for this BUCKET, which will become the final component of
    /// the BUCKET's resource name. For example, the value `foo` might result in
    /// a BUCKET with the name `projects/123456/buckets/foo`.
    #[prost(string, tag = "3")]
    pub bucket_id: ::prost::alloc::string::String,
    /// Apply a predefined set of access controls to this BUCKET.
    /// Valid values are "authenticatedRead", "private", "projectPrivate",
    /// "publicRead", or "publicReadWrite".
    #[prost(string, tag = "6")]
    pub predefined_acl: ::prost::alloc::string::String,
    /// Apply a predefined set of default object access controls to this BUCKET.
    /// Valid values are "authenticatedRead", "bucketOwnerFullControl",
    /// "bucketOwnerRead", "private", "projectPrivate", or "publicRead".
    #[prost(string, tag = "7")]
    pub predefined_default_object_acl: ::prost::alloc::string::String,
}
/// Request message for ListBuckets.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListBucketsRequest {
    /// Required. The project whose buckets we are listing.
    #[prost(string, tag = "1")]
    pub parent: ::prost::alloc::string::String,
    /// Maximum number of buckets to return in a single response. The service will
    /// use this parameter or 1,000 items, whichever is smaller. If "acl" is
    /// present in the read_mask, the service will use this parameter of 200 items,
    /// whichever is smaller.
    #[prost(int32, tag = "2")]
    pub page_size: i32,
    /// A previously-returned page token representing part of the larger set of
    /// results to view.
    #[prost(string, tag = "3")]
    pub page_token: ::prost::alloc::string::String,
    /// Filter results to buckets whose names begin with this prefix.
    #[prost(string, tag = "4")]
    pub prefix: ::prost::alloc::string::String,
    /// Mask specifying which fields to read from each result.
    /// If no mask is specified, will default to all fields except items.owner,
    /// items.acl, and items.default_object_acl.
    /// * may be used to mean "all fields".
    #[prost(message, optional, tag = "5")]
    pub read_mask: ::core::option::Option<::prost_types::FieldMask>,
    /// A set of parameters common to all Storage API requests.
    #[prost(message, optional, tag = "6")]
    pub common_request_params: ::core::option::Option<CommonRequestParams>,
}
/// The result of a call to Buckets.ListBuckets
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListBucketsResponse {
    /// The list of items.
    #[prost(message, repeated, tag = "1")]
    pub buckets: ::prost::alloc::vec::Vec<Bucket>,
    /// The continuation token, used to page through large result sets. Provide
    /// this value in a subsequent request to return the next page of results.
    #[prost(string, tag = "2")]
    pub next_page_token: ::prost::alloc::string::String,
}
/// Request message for LockBucketRetentionPolicyRequest.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LockBucketRetentionPolicyRequest {
    /// Required. Name of a BUCKET.
    #[prost(string, tag = "1")]
    pub bucket: ::prost::alloc::string::String,
    /// Makes the operation conditional on whether BUCKET's current metageneration
    /// matches the given value. Must be positive.
    #[prost(int64, tag = "2")]
    pub if_metageneration_match: i64,
    /// A set of parameters common to all Storage API requests.
    #[prost(message, optional, tag = "3")]
    pub common_request_params: ::core::option::Option<CommonRequestParams>,
}
/// Request for UpdateBucket method.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateBucketRequest {
    /// The BUCKET to update.
    /// The BUCKET's `name` field will be used to identify the BUCKET.
    #[prost(message, optional, tag = "1")]
    pub bucket: ::core::option::Option<Bucket>,
    /// If set, will only modify the BUCKET if its metageneration matches this
    /// value.
    #[prost(int64, optional, tag = "2")]
    pub if_metageneration_match: ::core::option::Option<i64>,
    /// If set, will only modify the BUCKET if its metageneration does not match
    /// this value.
    #[prost(int64, optional, tag = "3")]
    pub if_metageneration_not_match: ::core::option::Option<i64>,
    /// Apply a predefined set of access controls to this BUCKET.
    /// Valid values are "authenticatedRead", "private", "projectPrivate",
    /// "publicRead", or "publicReadWrite".
    #[prost(string, tag = "8")]
    pub predefined_acl: ::prost::alloc::string::String,
    /// Apply a predefined set of default object access controls to this BUCKET.
    /// Valid values are "authenticatedRead", "bucketOwnerFullControl",
    /// "bucketOwnerRead", "private", "projectPrivate", or "publicRead".
    #[prost(string, tag = "9")]
    pub predefined_default_object_acl: ::prost::alloc::string::String,
    /// List of fields to be updated.
    ///
    /// To specify ALL fields, equivalent to the JSON API's "update" function,
    /// specify a single field with the value `*`. Note: not recommended. If a new
    /// field is introduced at a later time, an older client updating with the `*`
    /// may accidentally reset the new field's value.
    ///
    /// Not specifying any fields is an error.
    /// Not specifying a field while setting that field to a non-default value is
    /// an error.
    #[prost(message, optional, tag = "6")]
    pub update_mask: ::core::option::Option<::prost_types::FieldMask>,
    /// A set of parameters common to all Storage API requests.
    #[prost(message, optional, tag = "7")]
    pub common_request_params: ::core::option::Option<CommonRequestParams>,
}
/// Request message for DeleteNotification.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteNotificationRequest {
    /// Required. The parent BUCKET of the notification.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
/// Request message for GetNotification.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetNotificationRequest {
    /// Required. The parent BUCKET of the notification.
    /// Format:
    /// `projects/{project}/buckets/{BUCKET}/notificationConfigs/{notification}`
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
/// Request message for CreateNotification.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateNotificationRequest {
    /// Required. The BUCKET to which this notification belongs.
    #[prost(string, tag = "1")]
    pub parent: ::prost::alloc::string::String,
    /// Required. Properties of the notification to be inserted.
    #[prost(message, optional, tag = "2")]
    pub notification: ::core::option::Option<Notification>,
}
/// Request message for ListNotifications.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListNotificationsRequest {
    /// Required. Name of a Google Cloud Storage BUCKET.
    #[prost(string, tag = "1")]
    pub parent: ::prost::alloc::string::String,
    /// The maximum number of notifications to return. The service may return fewer
    /// than this value.
    /// The maximum value is 100; values above 100 will be coerced to 100.
    #[prost(int32, tag = "2")]
    pub page_size: i32,
    /// A page token, received from a previous `ListNotifications` call.
    /// Provide this to retrieve the subsequent page.
    ///
    /// When paginating, all other parameters provided to `ListNotifications` must
    /// match the call that provided the page token.
    #[prost(string, tag = "3")]
    pub page_token: ::prost::alloc::string::String,
}
/// The result of a call to Notifications.ListNotifications
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListNotificationsResponse {
    /// The list of items.
    #[prost(message, repeated, tag = "1")]
    pub notifications: ::prost::alloc::vec::Vec<Notification>,
    /// A token, which can be sent as `page_token` to retrieve the next page.
    /// If this field is omitted, there are no subsequent pages.
    #[prost(string, tag = "2")]
    pub next_page_token: ::prost::alloc::string::String,
}
/// Request message for ComposeObject.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ComposeObjectRequest {
    /// Required. Properties of the resulting object.
    #[prost(message, optional, tag = "1")]
    pub destination: ::core::option::Option<Object>,
    /// The list of source objects that will be concatenated into a single object.
    #[prost(message, repeated, tag = "2")]
    pub source_objects: ::prost::alloc::vec::Vec<compose_object_request::SourceObject>,
    /// Apply a predefined set of access controls to the destination object.
    /// Valid values are "authenticatedRead", "bucketOwnerFullControl",
    /// "bucketOwnerRead", "private", "projectPrivate", or "publicRead".
    #[prost(string, tag = "9")]
    pub destination_predefined_acl: ::prost::alloc::string::String,
    /// Makes the operation conditional on whether the object's current generation
    /// matches the given value. Setting to 0 makes the operation succeed only if
    /// there are no live versions of the object.
    #[prost(int64, optional, tag = "4")]
    pub if_generation_match: ::core::option::Option<i64>,
    /// Makes the operation conditional on whether the object's current
    /// metageneration matches the given value.
    #[prost(int64, optional, tag = "5")]
    pub if_metageneration_match: ::core::option::Option<i64>,
    /// Resource name of the Cloud KMS key, of the form
    /// `projects/my-project/locations/my-location/keyRings/my-kr/cryptoKeys/my-key`,
    /// that will be used to encrypt the object. Overrides the object
    /// metadata's `kms_key_name` value, if any.
    #[prost(string, tag = "6")]
    pub kms_key: ::prost::alloc::string::String,
    /// A set of parameters common to Storage API requests concerning an object.
    #[prost(message, optional, tag = "7")]
    pub common_object_request_params: ::core::option::Option<CommonObjectRequestParams>,
    /// A set of parameters common to all Storage API requests.
    #[prost(message, optional, tag = "8")]
    pub common_request_params: ::core::option::Option<CommonRequestParams>,
}
/// Nested message and enum types in `ComposeObjectRequest`.
pub mod compose_object_request {
    /// Description of a source object for a composition request.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct SourceObject {
        /// Required. The source object's name. All source objects must reside in the same
        /// BUCKET.
        #[prost(string, tag = "1")]
        pub name: ::prost::alloc::string::String,
        /// The generation of this object to use as the source.
        #[prost(int64, tag = "2")]
        pub generation: i64,
        /// Conditions that must be met for this operation to execute.
        #[prost(message, optional, tag = "3")]
        pub object_preconditions: ::core::option::Option<source_object::ObjectPreconditions>,
    }
    /// Nested message and enum types in `SourceObject`.
    pub mod source_object {
        /// Preconditions for a source object of a composition request.
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct ObjectPreconditions {
            /// Only perform the composition if the generation of the source object
            /// that would be used matches this value.  If this value and a generation
            /// are both specified, they must be the same value or the call will fail.
            #[prost(int64, optional, tag = "1")]
            pub if_generation_match: ::core::option::Option<i64>,
        }
    }
}
/// Message for deleting an object.
/// Either `BUCKET` and `object` *or* `upload_id` **must** be set (but not both).
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteObjectRequest {
    /// Required. Name of the BUCKET in which the object resides.
    #[prost(string, tag = "1")]
    pub bucket: ::prost::alloc::string::String,
    /// Required. The name of the object to delete (when not using a resumable write).
    #[prost(string, tag = "2")]
    pub object: ::prost::alloc::string::String,
    /// The resumable upload_id of the object to delete (when deleting an
    /// in-progress resumable write). This should be copied from the `upload_id`
    /// field of `StartResumableWriteResponse`.
    #[prost(string, tag = "3")]
    pub upload_id: ::prost::alloc::string::String,
    /// If present, permanently deletes a specific revision of this object (as
    /// opposed to the latest version, the default).
    #[prost(int64, tag = "4")]
    pub generation: i64,
    /// Makes the operation conditional on whether the object's current generation
    /// matches the given value. Setting to 0 makes the operation succeed only if
    /// there are no live versions of the object.
    #[prost(int64, optional, tag = "5")]
    pub if_generation_match: ::core::option::Option<i64>,
    /// Makes the operation conditional on whether the object's current generation
    /// does not match the given value. If no live object exists, the precondition
    /// fails. Setting to 0 makes the operation succeed only if there is a live
    /// version of the object.
    #[prost(int64, optional, tag = "6")]
    pub if_generation_not_match: ::core::option::Option<i64>,
    /// Makes the operation conditional on whether the object's current
    /// metageneration matches the given value.
    #[prost(int64, optional, tag = "7")]
    pub if_metageneration_match: ::core::option::Option<i64>,
    /// Makes the operation conditional on whether the object's current
    /// metageneration does not match the given value.
    #[prost(int64, optional, tag = "8")]
    pub if_metageneration_not_match: ::core::option::Option<i64>,
    /// A set of parameters common to Storage API requests concerning an object.
    #[prost(message, optional, tag = "10")]
    pub common_object_request_params: ::core::option::Option<CommonObjectRequestParams>,
    /// A set of parameters common to all Storage API requests.
    #[prost(message, optional, tag = "11")]
    pub common_request_params: ::core::option::Option<CommonRequestParams>,
}
/// Request message for ReadObject.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReadObjectRequest {
    /// The name of the BUCKET containing the object to read.
    #[prost(string, tag = "1")]
    pub bucket: ::prost::alloc::string::String,
    /// The name of the object to read.
    #[prost(string, tag = "2")]
    pub object: ::prost::alloc::string::String,
    /// If present, selects a specific revision of this object (as opposed
    /// to the latest version, the default).
    #[prost(int64, tag = "3")]
    pub generation: i64,
    /// The offset for the first byte to return in the read, relative to the start
    /// of the object.
    ///
    /// A negative `read_offset` value will be interpreted as the number of bytes
    /// back from the end of the object to be returned. For example, if an object's
    /// length is 15 bytes, a ReadObjectRequest with `read_offset` = -5 and
    /// `read_limit` = 3 would return bytes 10 through 12 of the object. Requesting
    /// a negative offset whose magnitude is larger than the size of the object
    /// will result in an error.
    #[prost(int64, tag = "4")]
    pub read_offset: i64,
    /// The maximum number of `data` bytes the server is allowed to return in the
    /// sum of all `Object` messages. A `read_limit` of zero indicates that there
    /// is no limit, and a negative `read_limit` will cause an error.
    ///
    /// If the stream returns fewer bytes than allowed by the `read_limit` and no
    /// error occurred, the stream includes all data from the `read_offset` to the
    /// end of the resource.
    #[prost(int64, tag = "5")]
    pub read_limit: i64,
    /// Makes the operation conditional on whether the object's current generation
    /// matches the given value. Setting to 0 makes the operation succeed only if
    /// there are no live versions of the object.
    #[prost(int64, optional, tag = "6")]
    pub if_generation_match: ::core::option::Option<i64>,
    /// Makes the operation conditional on whether the object's current generation
    /// does not match the given value. If no live object exists, the precondition
    /// fails. Setting to 0 makes the operation succeed only if there is a live
    /// version of the object.
    #[prost(int64, optional, tag = "7")]
    pub if_generation_not_match: ::core::option::Option<i64>,
    /// Makes the operation conditional on whether the object's current
    /// metageneration matches the given value.
    #[prost(int64, optional, tag = "8")]
    pub if_metageneration_match: ::core::option::Option<i64>,
    /// Makes the operation conditional on whether the object's current
    /// metageneration does not match the given value.
    #[prost(int64, optional, tag = "9")]
    pub if_metageneration_not_match: ::core::option::Option<i64>,
    /// A set of parameters common to Storage API requests concerning an object.
    #[prost(message, optional, tag = "10")]
    pub common_object_request_params: ::core::option::Option<CommonObjectRequestParams>,
    /// A set of parameters common to all Storage API requests.
    #[prost(message, optional, tag = "11")]
    pub common_request_params: ::core::option::Option<CommonRequestParams>,
    /// Mask specifying which fields to read.
    /// The checksummed_data field and its children will always be present.
    /// If no mask is specified, will default to all fields except metadata.owner
    /// and metadata.acl.
    /// * may be used to mean "all fields".
    #[prost(message, optional, tag = "12")]
    pub read_mask: ::core::option::Option<::prost_types::FieldMask>,
}
/// Request message for GetObject.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetObjectRequest {
    /// Required. Name of the BUCKET in which the object resides.
    #[prost(string, tag = "1")]
    pub bucket: ::prost::alloc::string::String,
    /// Required. Name of the object.
    #[prost(string, tag = "2")]
    pub object: ::prost::alloc::string::String,
    /// If present, selects a specific revision of this object (as opposed to the
    /// latest version, the default).
    #[prost(int64, tag = "3")]
    pub generation: i64,
    /// Makes the operation conditional on whether the object's current generation
    /// matches the given value. Setting to 0 makes the operation succeed only if
    /// there are no live versions of the object.
    #[prost(int64, optional, tag = "4")]
    pub if_generation_match: ::core::option::Option<i64>,
    /// Makes the operation conditional on whether the object's current generation
    /// does not match the given value. If no live object exists, the precondition
    /// fails. Setting to 0 makes the operation succeed only if there is a live
    /// version of the object.
    #[prost(int64, optional, tag = "5")]
    pub if_generation_not_match: ::core::option::Option<i64>,
    /// Makes the operation conditional on whether the object's current
    /// metageneration matches the given value.
    #[prost(int64, optional, tag = "6")]
    pub if_metageneration_match: ::core::option::Option<i64>,
    /// Makes the operation conditional on whether the object's current
    /// metageneration does not match the given value.
    #[prost(int64, optional, tag = "7")]
    pub if_metageneration_not_match: ::core::option::Option<i64>,
    /// A set of parameters common to Storage API requests concerning an object.
    #[prost(message, optional, tag = "8")]
    pub common_object_request_params: ::core::option::Option<CommonObjectRequestParams>,
    /// A set of parameters common to all Storage API requests.
    #[prost(message, optional, tag = "9")]
    pub common_request_params: ::core::option::Option<CommonRequestParams>,
    /// Mask specifying which fields to read.
    /// If no mask is specified, will default to all fields except metadata.acl and
    /// metadata.owner.
    /// * may be used to mean "all fields".
    #[prost(message, optional, tag = "10")]
    pub read_mask: ::core::option::Option<::prost_types::FieldMask>,
}
/// Response message for ReadObject.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ReadObjectResponse {
    /// A portion of the data for the object. The service **may** leave `data`
    /// empty for any given `ReadResponse`. This enables the service to inform the
    /// client that the request is still live while it is running an operation to
    /// generate more data.
    #[prost(message, optional, tag = "1")]
    pub checksummed_data: ::core::option::Option<ChecksummedData>,
    /// The checksums of the complete object. The client should compute one of
    /// these checksums over the downloaded object and compare it against the value
    /// provided here.
    #[prost(message, optional, tag = "2")]
    pub object_checksums: ::core::option::Option<ObjectChecksums>,
    /// If read_offset and or read_limit was specified on the
    /// ReadObjectRequest, ContentRange will be populated on the first
    /// ReadObjectResponse message of the read stream.
    #[prost(message, optional, tag = "3")]
    pub content_range: ::core::option::Option<ContentRange>,
    /// Metadata of the object whose media is being returned.
    /// Only populated in the first response in the stream.
    #[prost(message, optional, tag = "4")]
    pub metadata: ::core::option::Option<Object>,
}
/// Describes an attempt to insert an object, possibly over multiple requests.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WriteObjectSpec {
    /// Destination object, including its name and its metadata.
    #[prost(message, optional, tag = "1")]
    pub resource: ::core::option::Option<Object>,
    /// Apply a predefined set of access controls to this object.
    /// Valid values are "authenticatedRead", "bucketOwnerFullControl",
    /// "bucketOwnerRead", "private", "projectPrivate", or "publicRead".
    #[prost(string, tag = "7")]
    pub predefined_acl: ::prost::alloc::string::String,
    /// Makes the operation conditional on whether the object's current
    /// generation matches the given value. Setting to 0 makes the operation
    /// succeed only if there are no live versions of the object.
    #[prost(int64, optional, tag = "3")]
    pub if_generation_match: ::core::option::Option<i64>,
    /// Makes the operation conditional on whether the object's current
    /// generation does not match the given value. If no live object exists, the
    /// precondition fails. Setting to 0 makes the operation succeed only if
    /// there is a live version of the object.
    #[prost(int64, optional, tag = "4")]
    pub if_generation_not_match: ::core::option::Option<i64>,
    /// Makes the operation conditional on whether the object's current
    /// metageneration matches the given value.
    #[prost(int64, optional, tag = "5")]
    pub if_metageneration_match: ::core::option::Option<i64>,
    /// Makes the operation conditional on whether the object's current
    /// metageneration does not match the given value.
    #[prost(int64, optional, tag = "6")]
    pub if_metageneration_not_match: ::core::option::Option<i64>,
}
/// Request message for WriteObject.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WriteObjectRequest {
    /// Required. The offset from the beginning of the object at which the data should be
    /// written.
    ///
    /// In the first `WriteObjectRequest` of a `WriteObject()` action, it
    /// indicates the initial offset for the `Write()` call. The value **must** be
    /// equal to the `persisted_size` that a call to `QueryWriteStatus()` would
    /// return (0 if this is the first write to the object).
    ///
    /// On subsequent calls, this value **must** be no larger than the sum of the
    /// first `write_offset` and the sizes of all `data` chunks sent previously on
    /// this stream.
    ///
    /// An incorrect value will cause an error.
    #[prost(int64, tag = "3")]
    pub write_offset: i64,
    /// Checksums for the complete object. If the checksums computed by the service
    /// don't match the specifified checksums the call will fail. May only be
    /// provided in the first or last request (either with first_message, or
    /// finish_write set).
    #[prost(message, optional, tag = "6")]
    pub object_checksums: ::core::option::Option<ObjectChecksums>,
    /// If `true`, this indicates that the write is complete. Sending any
    /// `WriteObjectRequest`s subsequent to one in which `finish_write` is `true`
    /// will cause an error.
    /// For a non-resumable write (where the upload_id was not set in the first
    /// message), it is an error not to set this field in the final message of the
    /// stream.
    #[prost(bool, tag = "7")]
    pub finish_write: bool,
    /// A set of parameters common to Storage API requests concerning an object.
    #[prost(message, optional, tag = "8")]
    pub common_object_request_params: ::core::option::Option<CommonObjectRequestParams>,
    /// A set of parameters common to all Storage API requests.
    #[prost(message, optional, tag = "9")]
    pub common_request_params: ::core::option::Option<CommonRequestParams>,
    /// The first message of each stream should set one of the following.
    #[prost(oneof = "write_object_request::FirstMessage", tags = "1, 2")]
    pub first_message: ::core::option::Option<write_object_request::FirstMessage>,
    /// A portion of the data for the object.
    #[prost(oneof = "write_object_request::Data", tags = "4")]
    pub data: ::core::option::Option<write_object_request::Data>,
}
/// Nested message and enum types in `WriteObjectRequest`.
pub mod write_object_request {
    /// The first message of each stream should set one of the following.
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum FirstMessage {
        /// For resumable uploads. This should be the `upload_id` returned from a
        /// call to `StartResumableWriteResponse`.
        #[prost(string, tag = "1")]
        UploadId(::prost::alloc::string::String),
        /// For non-resumable uploads. Describes the overall upload, including the
        /// destination BUCKET and object name, preconditions, etc.
        #[prost(message, tag = "2")]
        WriteObjectSpec(super::WriteObjectSpec),
    }
    /// A portion of the data for the object.
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Data {
        /// The data to insert. If a crc32c checksum is provided that doesn't match
        /// the checksum computed by the service, the request will fail.
        #[prost(message, tag = "4")]
        ChecksummedData(super::ChecksummedData),
    }
}
/// Response message for WriteObject.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct WriteObjectResponse {
    /// The response will set one of the following.
    #[prost(oneof = "write_object_response::WriteStatus", tags = "1, 2")]
    pub write_status: ::core::option::Option<write_object_response::WriteStatus>,
}
/// Nested message and enum types in `WriteObjectResponse`.
pub mod write_object_response {
    /// The response will set one of the following.
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum WriteStatus {
        /// The total number of bytes that have been processed for the given object
        /// from all `WriteObject` calls. Only set if the upload has not finalized.
        #[prost(int64, tag = "1")]
        PersistedSize(i64),
        /// A resource containing the metadata for the uploaded object. Only set if
        /// the upload has finalized.
        #[prost(message, tag = "2")]
        Resource(super::Object),
    }
}
/// Request message for ListObjects.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListObjectsRequest {
    /// Required. Name of the BUCKET in which to look for objects.
    #[prost(string, tag = "1")]
    pub parent: ::prost::alloc::string::String,
    /// Maximum number of `items` plus `prefixes` to return
    /// in a single page of responses. As duplicate `prefixes` are
    /// omitted, fewer total results may be returned than requested. The service
    /// will use this parameter or 1,000 items, whichever is smaller.
    #[prost(int32, tag = "2")]
    pub page_size: i32,
    /// A previously-returned page token representing part of the larger set of
    /// results to view.
    #[prost(string, tag = "3")]
    pub page_token: ::prost::alloc::string::String,
    /// If set, returns results in a directory-like mode. `items` will contain
    /// only objects whose names, aside from the `prefix`, do not
    /// contain `delimiter`. Objects whose names, aside from the
    /// `prefix`, contain `delimiter` will have their name,
    /// truncated after the `delimiter`, returned in
    /// `prefixes`. Duplicate `prefixes` are omitted.
    #[prost(string, tag = "4")]
    pub delimiter: ::prost::alloc::string::String,
    /// If true, objects that end in exactly one instance of `delimiter`
    /// will have their metadata included in `items` in addition to
    /// `prefixes`.
    #[prost(bool, tag = "5")]
    pub include_trailing_delimiter: bool,
    /// Filter results to objects whose names begin with this prefix.
    #[prost(string, tag = "6")]
    pub prefix: ::prost::alloc::string::String,
    /// If `true`, lists all versions of an object as distinct results.
    /// For more information, see
    /// [Object
    /// Versioning](<https://cloud.google.com/storage/docs/object-versioning>).
    #[prost(bool, tag = "7")]
    pub versions: bool,
    /// Mask specifying which fields to read from each result.
    /// If no mask is specified, will default to all fields except items.acl and
    /// items.owner.
    /// * may be used to mean "all fields".
    #[prost(message, optional, tag = "8")]
    pub read_mask: ::core::option::Option<::prost_types::FieldMask>,
    /// Filter results to objects whose names are lexicographically equal to or
    /// after lexicographic_start. If lexicographic_end is also set, the objects
    /// listed have names between lexicographic_start (inclusive) and
    /// lexicographic_end (exclusive).
    #[prost(string, tag = "10")]
    pub lexicographic_start: ::prost::alloc::string::String,
    /// Filter results to objects whose names are lexicographically before
    /// lexicographic_end. If lexicographic_start is also set, the objects listed
    /// have names between lexicographic_start (inclusive) and lexicographic_end
    /// (exclusive).
    #[prost(string, tag = "11")]
    pub lexicographic_end: ::prost::alloc::string::String,
    /// A set of parameters common to all Storage API requests.
    #[prost(message, optional, tag = "9")]
    pub common_request_params: ::core::option::Option<CommonRequestParams>,
}
/// Request object for `QueryWriteStatus`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryWriteStatusRequest {
    /// Required. The name of the resume token for the object whose write status is being
    /// requested.
    #[prost(string, tag = "1")]
    pub upload_id: ::prost::alloc::string::String,
    /// A set of parameters common to Storage API requests concerning an object.
    #[prost(message, optional, tag = "2")]
    pub common_object_request_params: ::core::option::Option<CommonObjectRequestParams>,
    /// A set of parameters common to all Storage API requests.
    #[prost(message, optional, tag = "3")]
    pub common_request_params: ::core::option::Option<CommonRequestParams>,
}
/// Response object for `QueryWriteStatus`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct QueryWriteStatusResponse {
    /// The response will set one of the following.
    #[prost(oneof = "query_write_status_response::WriteStatus", tags = "1, 2")]
    pub write_status: ::core::option::Option<query_write_status_response::WriteStatus>,
}
/// Nested message and enum types in `QueryWriteStatusResponse`.
pub mod query_write_status_response {
    /// The response will set one of the following.
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum WriteStatus {
        /// The total number of bytes that have been processed for the given object
        /// from all `WriteObject` calls. This is the correct value for the
        /// 'write_offset' field to use when resuming the `WriteObject` operation.
        /// Only set if the upload has not finalized.
        #[prost(int64, tag = "1")]
        PersistedSize(i64),
        /// A resource containing the metadata for the uploaded object. Only set if
        /// the upload has finalized.
        #[prost(message, tag = "2")]
        Resource(super::Object),
    }
}
/// Request message for RewriteObject.
/// If the source object is encrypted using a Customer-Supplied Encryption Key
/// the key information must be provided in the copy_source_encryption_algorithm,
/// copy_source_encryption_key_bytes, and copy_source_encryption_key_sha256_bytes
/// fields. If the destination object should be encrypted the keying information
/// should be provided in the encryption_algorithm, encryption_key_bytes, and
/// encryption_key_sha256_bytes fields of the
/// common_object_request_params.customer_encryption field.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RewriteObjectRequest {
    /// Immutable. The name of the destination object. Nearly any sequence of unicode
    /// characters is valid. See
    /// \[Guidelines\](<https://cloud.google.com/storage/docs/naming-objects>).
    /// Example: `test.txt`
    /// The `name` field by itself does not uniquely identify a Cloud Storage
    /// object. A Cloud Storage object is uniquely identified by the tuple of
    /// (BUCKET, object, generation).
    #[prost(string, tag = "24")]
    pub destination_name: ::prost::alloc::string::String,
    /// Immutable. The name of the BUCKET containing The name of the destination object.
    #[prost(string, tag = "25")]
    pub destination_bucket: ::prost::alloc::string::String,
    /// The name of the Cloud KMS key that will be used to encrypt the destination
    /// object. The Cloud KMS key must be located in same location as the object.
    /// If the parameter is not specified, the request uses the destination
    /// BUCKET's default encryption key, if any, or else the Google-managed
    /// encryption key.
    #[prost(string, tag = "27")]
    pub destination_kms_key: ::prost::alloc::string::String,
    /// Properties of the destination, post-rewrite object.
    /// The `name`, `BUCKET` and `kms_key` fields must not be populated (these
    /// values are specified in the `destination_name`, `destination_bucket`, and
    /// `destination_kms_key` fields).
    /// If `destination` is present it will be used to construct the destination
    /// object's metadata; otherwise the destination object's metadata will be
    /// copied from the source object.
    #[prost(message, optional, tag = "1")]
    pub destination: ::core::option::Option<Object>,
    /// Required. Name of the BUCKET in which to find the source object.
    #[prost(string, tag = "2")]
    pub source_bucket: ::prost::alloc::string::String,
    /// Required. Name of the source object.
    #[prost(string, tag = "3")]
    pub source_object: ::prost::alloc::string::String,
    /// If present, selects a specific revision of the source object (as opposed to
    /// the latest version, the default).
    #[prost(int64, tag = "4")]
    pub source_generation: i64,
    /// Include this field (from the previous rewrite response) on each rewrite
    /// request after the first one, until the rewrite response 'done' flag is
    /// true. Calls that provide a rewriteToken can omit all other request fields,
    /// but if included those fields must match the values provided in the first
    /// rewrite request.
    #[prost(string, tag = "5")]
    pub rewrite_token: ::prost::alloc::string::String,
    /// Apply a predefined set of access controls to the destination object.
    /// Valid values are "authenticatedRead", "bucketOwnerFullControl",
    /// "bucketOwnerRead", "private", "projectPrivate", or "publicRead".
    #[prost(string, tag = "28")]
    pub destination_predefined_acl: ::prost::alloc::string::String,
    /// Makes the operation conditional on whether the object's current generation
    /// matches the given value. Setting to 0 makes the operation succeed only if
    /// there are no live versions of the object.
    #[prost(int64, optional, tag = "7")]
    pub if_generation_match: ::core::option::Option<i64>,
    /// Makes the operation conditional on whether the object's current generation
    /// does not match the given value. If no live object exists, the precondition
    /// fails. Setting to 0 makes the operation succeed only if there is a live
    /// version of the object.
    #[prost(int64, optional, tag = "8")]
    pub if_generation_not_match: ::core::option::Option<i64>,
    /// Makes the operation conditional on whether the destination object's current
    /// metageneration matches the given value.
    #[prost(int64, optional, tag = "9")]
    pub if_metageneration_match: ::core::option::Option<i64>,
    /// Makes the operation conditional on whether the destination object's current
    /// metageneration does not match the given value.
    #[prost(int64, optional, tag = "10")]
    pub if_metageneration_not_match: ::core::option::Option<i64>,
    /// Makes the operation conditional on whether the source object's current
    /// generation matches the given value.
    #[prost(int64, optional, tag = "11")]
    pub if_source_generation_match: ::core::option::Option<i64>,
    /// Makes the operation conditional on whether the source object's current
    /// generation does not match the given value.
    #[prost(int64, optional, tag = "12")]
    pub if_source_generation_not_match: ::core::option::Option<i64>,
    /// Makes the operation conditional on whether the source object's current
    /// metageneration matches the given value.
    #[prost(int64, optional, tag = "13")]
    pub if_source_metageneration_match: ::core::option::Option<i64>,
    /// Makes the operation conditional on whether the source object's current
    /// metageneration does not match the given value.
    #[prost(int64, optional, tag = "14")]
    pub if_source_metageneration_not_match: ::core::option::Option<i64>,
    /// The maximum number of bytes that will be rewritten per rewrite request.
    /// Most callers
    /// shouldn't need to specify this parameter - it is primarily in place to
    /// support testing. If specified the value must be an integral multiple of
    /// 1 MiB (1048576). Also, this only applies to requests where the source and
    /// destination span locations and/or storage classes. Finally, this value must
    /// not change across rewrite calls else you'll get an error that the
    /// `rewriteToken` is invalid.
    #[prost(int64, tag = "15")]
    pub max_bytes_rewritten_per_call: i64,
    /// The algorithm used to encrypt the source object, if any. Used if the source
    /// object was encrypted with a Customer-Supplied Encryption Key.
    #[prost(string, tag = "16")]
    pub copy_source_encryption_algorithm: ::prost::alloc::string::String,
    /// The raw bytes (not base64-encoded) AES-256 encryption key used to encrypt
    /// the source object, if it was encrypted with a Customer-Supplied Encryption
    /// Key.
    #[prost(bytes = "vec", tag = "21")]
    pub copy_source_encryption_key_bytes: ::prost::alloc::vec::Vec<u8>,
    /// The raw bytes (not base64-encoded) SHA256 hash of the encryption key used
    /// to encrypt the source object, if it was encrypted with a Customer-Supplied
    /// Encryption Key.
    #[prost(bytes = "vec", tag = "22")]
    pub copy_source_encryption_key_sha256_bytes: ::prost::alloc::vec::Vec<u8>,
    /// A set of parameters common to Storage API requests concerning an object.
    #[prost(message, optional, tag = "19")]
    pub common_object_request_params: ::core::option::Option<CommonObjectRequestParams>,
    /// A set of parameters common to all Storage API requests.
    #[prost(message, optional, tag = "20")]
    pub common_request_params: ::core::option::Option<CommonRequestParams>,
}
/// A rewrite response.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RewriteResponse {
    /// The total bytes written so far, which can be used to provide a waiting user
    /// with a progress indicator. This property is always present in the response.
    #[prost(int64, tag = "1")]
    pub total_bytes_rewritten: i64,
    /// The total size of the object being copied in bytes. This property is always
    /// present in the response.
    #[prost(int64, tag = "2")]
    pub object_size: i64,
    /// `true` if the copy is finished; otherwise, `false` if
    /// the copy is in progress. This property is always present in the response.
    #[prost(bool, tag = "3")]
    pub done: bool,
    /// A token to use in subsequent requests to continue copying data. This token
    /// is present in the response only when there is more data to copy.
    #[prost(string, tag = "4")]
    pub rewrite_token: ::prost::alloc::string::String,
    /// A resource containing the metadata for the copied-to object. This property
    /// is present in the response only when copying completes.
    #[prost(message, optional, tag = "5")]
    pub resource: ::core::option::Option<Object>,
}
/// Request message StartResumableWrite.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StartResumableWriteRequest {
    /// The destination BUCKET, object, and metadata, as well as any preconditions.
    #[prost(message, optional, tag = "1")]
    pub write_object_spec: ::core::option::Option<WriteObjectSpec>,
    /// A set of parameters common to Storage API requests concerning an object.
    #[prost(message, optional, tag = "3")]
    pub common_object_request_params: ::core::option::Option<CommonObjectRequestParams>,
    /// A set of parameters common to all Storage API requests.
    #[prost(message, optional, tag = "4")]
    pub common_request_params: ::core::option::Option<CommonRequestParams>,
}
/// Response object for `StartResumableWrite`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StartResumableWriteResponse {
    /// The upload_id of the newly started resumable write operation. This
    /// value should be copied into the `WriteObjectRequest.upload_id` field.
    #[prost(string, tag = "1")]
    pub upload_id: ::prost::alloc::string::String,
}
/// Request message for UpdateObject.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateObjectRequest {
    /// The object to update.
    /// The object's BUCKET and name fields are used to identify the object to
    /// update. If present, the object's generation field selects a specific
    /// revision of this object whose metadata should be updated. Otherwise,
    /// assumes the current, live version of the object.
    #[prost(message, optional, tag = "1")]
    pub object: ::core::option::Option<Object>,
    /// Makes the operation conditional on whether the object's current generation
    /// matches the given value. Setting to 0 makes the operation succeed only if
    /// there are no live versions of the object.
    #[prost(int64, optional, tag = "2")]
    pub if_generation_match: ::core::option::Option<i64>,
    /// Makes the operation conditional on whether the object's current generation
    /// does not match the given value. If no live object exists, the precondition
    /// fails. Setting to 0 makes the operation succeed only if there is a live
    /// version of the object.
    #[prost(int64, optional, tag = "3")]
    pub if_generation_not_match: ::core::option::Option<i64>,
    /// Makes the operation conditional on whether the object's current
    /// metageneration matches the given value.
    #[prost(int64, optional, tag = "4")]
    pub if_metageneration_match: ::core::option::Option<i64>,
    /// Makes the operation conditional on whether the object's current
    /// metageneration does not match the given value.
    #[prost(int64, optional, tag = "5")]
    pub if_metageneration_not_match: ::core::option::Option<i64>,
    /// Apply a predefined set of access controls to this object.
    /// Valid values are "authenticatedRead", "bucketOwnerFullControl",
    /// "bucketOwnerRead", "private", "projectPrivate", or "publicRead".
    #[prost(string, tag = "10")]
    pub predefined_acl: ::prost::alloc::string::String,
    /// List of fields to be updated.
    ///
    /// To specify ALL fields, equivalent to the JSON API's "update" function,
    /// specify a single field with the value `*`. Note: not recommended. If a new
    /// field is introduced at a later time, an older client updating with the `*`
    /// may accidentally reset the new field's value.
    ///
    /// Not specifying any fields is an error.
    /// Not specifying a field while setting that field to a non-default value is
    /// an error.
    #[prost(message, optional, tag = "7")]
    pub update_mask: ::core::option::Option<::prost_types::FieldMask>,
    /// A set of parameters common to Storage API requests concerning an object.
    #[prost(message, optional, tag = "8")]
    pub common_object_request_params: ::core::option::Option<CommonObjectRequestParams>,
    /// A set of parameters common to all Storage API requests.
    #[prost(message, optional, tag = "9")]
    pub common_request_params: ::core::option::Option<CommonRequestParams>,
}
/// Request message for GetServiceAccount.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetServiceAccountRequest {
    /// Required. Project ID.
    #[prost(string, tag = "1")]
    pub project: ::prost::alloc::string::String,
    /// A set of parameters common to all Storage API requests.
    #[prost(message, optional, tag = "2")]
    pub common_request_params: ::core::option::Option<CommonRequestParams>,
}
/// Request message for CreateHmacKey.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateHmacKeyRequest {
    /// Required. The project that the HMAC-owning service account lives in.
    #[prost(string, tag = "1")]
    pub project: ::prost::alloc::string::String,
    /// Required. The service account to create the HMAC for.
    #[prost(string, tag = "2")]
    pub service_account_email: ::prost::alloc::string::String,
    /// A set of parameters common to all Storage API requests.
    #[prost(message, optional, tag = "3")]
    pub common_request_params: ::core::option::Option<CommonRequestParams>,
}
/// Create hmac response.  The only time the secret for an HMAC will be returned.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateHmacKeyResponse {
    /// Key metadata.
    #[prost(message, optional, tag = "1")]
    pub metadata: ::core::option::Option<HmacKeyMetadata>,
    /// HMAC key secret material.
    /// In raw bytes format (not base64-encoded).
    #[prost(bytes = "vec", tag = "3")]
    pub secret_key_bytes: ::prost::alloc::vec::Vec<u8>,
}
/// Request object to delete a given HMAC key.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteHmacKeyRequest {
    /// Required. The identifying key for the HMAC to delete.
    #[prost(string, tag = "1")]
    pub access_id: ::prost::alloc::string::String,
    /// Required. The project id the HMAC key lies in.
    #[prost(string, tag = "2")]
    pub project: ::prost::alloc::string::String,
    /// A set of parameters common to all Storage API requests.
    #[prost(message, optional, tag = "3")]
    pub common_request_params: ::core::option::Option<CommonRequestParams>,
}
/// Request object to get metadata on a given HMAC key.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetHmacKeyRequest {
    /// Required. The identifying key for the HMAC to delete.
    #[prost(string, tag = "1")]
    pub access_id: ::prost::alloc::string::String,
    /// Required. The project id the HMAC key lies in.
    #[prost(string, tag = "2")]
    pub project: ::prost::alloc::string::String,
    /// A set of parameters common to all Storage API requests.
    #[prost(message, optional, tag = "3")]
    pub common_request_params: ::core::option::Option<CommonRequestParams>,
}
/// Request to fetch a list of HMAC keys under a given project.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListHmacKeysRequest {
    /// Required. The project id to list HMAC keys for.
    #[prost(string, tag = "1")]
    pub project: ::prost::alloc::string::String,
    /// Optional. The maximum number of keys to return.
    #[prost(int32, tag = "2")]
    pub page_size: i32,
    /// Optional. A previously returned token from ListHmacKeysResponse to get the next page.
    #[prost(string, tag = "3")]
    pub page_token: ::prost::alloc::string::String,
    /// Optional. If set, filters to only return HMAC keys for specified service account.
    #[prost(string, tag = "4")]
    pub service_account_email: ::prost::alloc::string::String,
    /// Optional. If set, return deleted keys that have not yet been wiped out.
    #[prost(bool, tag = "5")]
    pub show_deleted_keys: bool,
    /// Optional. A set of parameters common to all Storage API requests.
    #[prost(message, optional, tag = "6")]
    pub common_request_params: ::core::option::Option<CommonRequestParams>,
}
/// Hmac key list response with next page information.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListHmacKeysResponse {
    /// The list of items.
    #[prost(message, repeated, tag = "1")]
    pub hmac_keys: ::prost::alloc::vec::Vec<HmacKeyMetadata>,
    /// The continuation token, used to page through large result sets. Provide
    /// this value in a subsequent request to return the next page of results.
    #[prost(string, tag = "2")]
    pub next_page_token: ::prost::alloc::string::String,
}
/// Request object to update an HMAC key state.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateHmacKeyRequest {
    /// Required. The HMAC key to update.
    /// If present, the hmac_key's `id` field will be used to identify the key.
    /// Otherwise, the hmac_key's access_id and project fields will be used to
    /// identify the key.
    #[prost(message, optional, tag = "1")]
    pub hmac_key: ::core::option::Option<HmacKeyMetadata>,
    /// A set of parameters common to all Storage API requests.
    #[prost(message, optional, tag = "2")]
    pub common_request_params: ::core::option::Option<CommonRequestParams>,
    /// Update mask for hmac_key.
    #[prost(message, optional, tag = "3")]
    pub update_mask: ::core::option::Option<::prost_types::FieldMask>,
}
/// Parameters that can be passed to any object request.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommonObjectRequestParams {
    /// Encryption algorithm used with the Customer-Supplied Encryption Keys
    /// feature.
    #[prost(string, tag = "1")]
    pub encryption_algorithm: ::prost::alloc::string::String,
    /// Encryption key used with the Customer-Supplied Encryption Keys feature.
    /// In raw bytes format (not base64-encoded).
    #[prost(bytes = "vec", tag = "4")]
    pub encryption_key_bytes: ::prost::alloc::vec::Vec<u8>,
    /// SHA256 hash of encryption key used with the Customer-Supplied Encryption
    /// Keys feature.
    #[prost(bytes = "vec", tag = "5")]
    pub encryption_key_sha256_bytes: ::prost::alloc::vec::Vec<u8>,
}
/// Parameters that can be passed to any request.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommonRequestParams {
    /// Required. Required when using buckets with Requestor Pays feature enabled.
    #[prost(string, tag = "1")]
    pub user_project: ::prost::alloc::string::String,
}
/// Shared constants.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ServiceConstants {}
/// Nested message and enum types in `ServiceConstants`.
pub mod service_constants {
    /// A collection of constant values meaningful to the Storage API.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Values {
        /// Unused. Proto3 requires first enum to be 0.
        Unspecified = 0,
        /// The maximum size chunk that can will be returned in a single
        /// ReadRequest.
        /// 2 MiB.
        MaxReadChunkBytes = 2097152,
        /// The maximum size of an object in MB - whether written in a single stream
        /// or composed from multiple other objects.
        /// 5 TiB.
        MaxObjectSizeMb = 5242880,
        /// The maximum length field name that can be sent in a single
        /// custom metadata field.
        /// 1 KiB.
        MaxCustomMetadataFieldNameBytes = 1024,
        /// The maximum length field value that can be sent in a single
        /// custom_metadata field.
        /// 4 KiB.
        MaxCustomMetadataFieldValueBytes = 4096,
        /// The maximum total bytes that can be populated into all field names and
        /// values of the custom_metadata for one object.
        /// 8 KiB.
        MaxCustomMetadataTotalSizeBytes = 8192,
        /// The maximum total bytes that can be populated into all BUCKET metadata
        /// fields.
        /// 20 KiB.
        MaxBucketMetadataTotalSizeBytes = 20480,
        /// The maximum number of NotificationConfigs that can be registered
        /// for a given BUCKET.
        MaxNotificationConfigsPerBucket = 100,
        /// The maximum number of custom attributes per NotificationConfigs.
        MaxNotificationCustomAttributes = 5,
        /// The maximum length of a custom attribute key included in
        /// NotificationConfig.
        MaxNotificationCustomAttributeKeyLength = 256,
        /// The maximum number of key/value entries per BUCKET label.
        MaxLabelsEntriesCount = 64,
        /// The maximum character length of the key or value in a BUCKET
        /// label map.
        MaxLabelsKeyValueLength = 63,
        /// The maximum byte size of the key or value in a BUCKET label
        /// map.
        MaxLabelsKeyValueBytes = 128,
        /// The maximum number of object IDs that can be included in a
        /// DeleteObjectsRequest.
        MaxObjectIdsPerDeleteObjectsRequest = 1000,
        /// The maximum number of days for which a token returned by the
        /// GetListObjectsSplitPoints RPC is valid.
        SplitTokenMaxValidDays = 14,
    }
}
/// A BUCKET.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Bucket {
    /// Immutable. The name of the BUCKET.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// Output only. The user-chosen part of the BUCKET name. The `{BUCKET}` portion of the
    /// `name` field. For globally unique buckets, this is equal to the "BUCKET
    /// name" of other Cloud Storage APIs. Example: "pub".
    #[prost(string, tag = "2")]
    pub bucket_id: ::prost::alloc::string::String,
    /// Immutable. The project which owns this BUCKET.
    #[prost(string, tag = "3")]
    pub project: ::prost::alloc::string::String,
    /// Output only. The metadata generation of this BUCKET.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    #[prost(int64, tag = "4")]
    pub metageneration: i64,
    /// Immutable. The location of the BUCKET. Object data for objects in the BUCKET resides
    /// in physical storage within this region.  Defaults to `US`. See the
    /// \[<https://developers.google.com/storage/docs/concepts-techniques#specifyinglocations"\][developer's>
    /// guide] for the authoritative list. Attempting to update this field after
    /// the BUCKET is created will result in an error.
    #[prost(string, tag = "5")]
    pub location: ::prost::alloc::string::String,
    /// Output only. The location type of the BUCKET (region, dual-region, multi-region, etc).
    #[prost(string, tag = "6")]
    pub location_type: ::prost::alloc::string::String,
    /// The BUCKET's default storage class, used whenever no storageClass is
    /// specified for a newly-created object. This defines how objects in the
    /// BUCKET are stored and determines the SLA and the cost of storage.
    /// If this value is not specified when the BUCKET is created, it will default
    /// to `STANDARD`. For more information, see
    /// <https://developers.google.com/storage/docs/storage-classes.>
    #[prost(string, tag = "7")]
    pub storage_class: ::prost::alloc::string::String,
    /// The recovery point objective for cross-region replication of the BUCKET.
    /// Applicable only for dual- and multi-region buckets. "DEFAULT" uses default
    /// replication. "ASYNC_TURBO" enables turbo replication, valid for dual-region
    /// buckets only. If rpo is not specified when the BUCKET is created, it
    /// defaults to "DEFAULT". For more information, see
    /// <https://cloud.google.com/storage/docs/turbo-replication.>
    #[prost(string, tag = "27")]
    pub rpo: ::prost::alloc::string::String,
    /// Access controls on the BUCKET.
    /// If iamConfig.uniformBucketLevelAccess is enabled on this BUCKET,
    /// requests to set, read, or modify acl is an error.
    #[prost(message, repeated, tag = "8")]
    pub acl: ::prost::alloc::vec::Vec<BucketAccessControl>,
    /// Default access controls to apply to new objects when no ACL is provided.
    /// If iamConfig.uniformBucketLevelAccess is enabled on this BUCKET,
    /// requests to set, read, or modify acl is an error.
    #[prost(message, repeated, tag = "9")]
    pub default_object_acl: ::prost::alloc::vec::Vec<ObjectAccessControl>,
    /// The BUCKET's lifecycle config. See
    /// \[<https://developers.google.com/storage/docs/lifecycle\]Lifecycle> Management]
    /// for more information.
    #[prost(message, optional, tag = "10")]
    pub lifecycle: ::core::option::Option<bucket::Lifecycle>,
    /// Output only. The creation time of the BUCKET in
    /// \[<https://tools.ietf.org/html/rfc3339\][RFC> 3339] format.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    #[prost(message, optional, tag = "11")]
    pub create_time: ::core::option::Option<::prost_types::Timestamp>,
    /// The BUCKET's \[<https://www.w3.org/TR/cors/\][Cross-Origin> Resource Sharing]
    /// (CORS) config.
    #[prost(message, repeated, tag = "12")]
    pub cors: ::prost::alloc::vec::Vec<bucket::Cors>,
    /// Output only. The modification time of the BUCKET.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    #[prost(message, optional, tag = "13")]
    pub update_time: ::core::option::Option<::prost_types::Timestamp>,
    /// The default value for event-based hold on newly created objects in this
    /// BUCKET.  Event-based hold is a way to retain objects indefinitely until an
    /// event occurs, signified by the
    /// hold's release. After being released, such objects will be subject to
    /// BUCKET-level retention (if any).  One sample use case of this flag is for
    /// banks to hold loan documents for at least 3 years after loan is paid in
    /// full. Here, BUCKET-level retention is 3 years and the event is loan being
    /// paid in full. In this example, these objects will be held intact for any
    /// number of years until the event has occurred (event-based hold on the
    /// object is released) and then 3 more years after that. That means retention
    /// duration of the objects begins from the moment event-based hold
    /// transitioned from true to false.  Objects under event-based hold cannot be
    /// deleted, overwritten or archived until the hold is removed.
    #[prost(bool, tag = "14")]
    pub default_event_based_hold: bool,
    /// User-provided labels, in key/value pairs.
    #[prost(map = "string, string", tag = "15")]
    pub labels: ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
    /// The BUCKET's website config, controlling how the service behaves
    /// when accessing BUCKET contents as a web site. See the
    /// \[<https://cloud.google.com/storage/docs/static-website\][Static> Website
    /// Examples] for more information.
    #[prost(message, optional, tag = "16")]
    pub website: ::core::option::Option<bucket::Website>,
    /// The BUCKET's versioning config.
    #[prost(message, optional, tag = "17")]
    pub versioning: ::core::option::Option<bucket::Versioning>,
    /// The BUCKET's logging config, which defines the destination BUCKET
    /// and name prefix (if any) for the current BUCKET's logs.
    #[prost(message, optional, tag = "18")]
    pub logging: ::core::option::Option<bucket::Logging>,
    /// Output only. The owner of the BUCKET. This is always the project team's owner group.
    #[prost(message, optional, tag = "19")]
    pub owner: ::core::option::Option<Owner>,
    /// Encryption config for a BUCKET.
    #[prost(message, optional, tag = "20")]
    pub encryption: ::core::option::Option<bucket::Encryption>,
    /// The BUCKET's billing config.
    #[prost(message, optional, tag = "21")]
    pub billing: ::core::option::Option<bucket::Billing>,
    /// The BUCKET's retention policy. The retention policy enforces a minimum
    /// retention time for all objects contained in the BUCKET, based on their
    /// creation time. Any attempt to overwrite or delete objects younger than the
    /// retention period will result in a PERMISSION_DENIED error.  An unlocked
    /// retention policy can be modified or removed from the BUCKET via a
    /// storage.buckets.update operation. A locked retention policy cannot be
    /// removed or shortened in duration for the lifetime of the BUCKET.
    /// Attempting to remove or decrease period of a locked retention policy will
    /// result in a PERMISSION_DENIED error.
    #[prost(message, optional, tag = "22")]
    pub retention_policy: ::core::option::Option<bucket::RetentionPolicy>,
    /// The BUCKET's IAM config.
    #[prost(message, optional, tag = "23")]
    pub iam_config: ::core::option::Option<bucket::IamConfig>,
    /// Reserved for future use.
    #[prost(bool, tag = "25")]
    pub satisfies_pzs: bool,
    /// The BUCKET's Autoclass configuration. If there is no configuration, the
    /// Autoclass feature will be disabled and have no effect on the BUCKET.
    #[prost(message, optional, tag = "28")]
    pub autoclass: ::core::option::Option<bucket::Autoclass>,
}
/// Nested message and enum types in `Bucket`.
pub mod bucket {
    /// Billing properties of a BUCKET.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Billing {
        /// When set to true, Requester Pays is enabled for this BUCKET.
        #[prost(bool, tag = "1")]
        pub requester_pays: bool,
    }
    /// Cross-Origin Response sharing (CORS) properties for a BUCKET.
    /// For more on Cloud Storage and CORS, see
    /// <https://cloud.google.com/storage/docs/cross-origin.>
    /// For more on CORS in general, see <https://tools.ietf.org/html/rfc6454.>
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Cors {
        /// The list of Origins eligible to receive CORS response headers. See
        /// \[<https://tools.ietf.org/html/rfc6454\][RFC> 6454] for more on origins.
        /// Note: "*" is permitted in the list of origins, and means "any Origin".
        #[prost(string, repeated, tag = "1")]
        pub origin: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
        /// The list of HTTP methods on which to include CORS response headers,
        /// (`GET`, `OPTIONS`, `POST`, etc) Note: "*" is permitted in the list of
        /// methods, and means "any method".
        #[prost(string, repeated, tag = "2")]
        pub method: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
        /// The list of HTTP headers other than the
        /// \[<https://www.w3.org/TR/cors/#simple-response-header\][simple> response
        /// headers] to give permission for the user-agent to share across domains.
        #[prost(string, repeated, tag = "3")]
        pub response_header: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
        /// The value, in seconds, to return in the
        /// \[<https://www.w3.org/TR/cors/#access-control-max-age-response-header\][Access-Control-Max-Age>
        /// header] used in preflight responses.
        #[prost(int32, tag = "4")]
        pub max_age_seconds: i32,
    }
    /// Encryption properties of a BUCKET.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Encryption {
        /// The name of the Cloud KMS key that will be used to encrypt objects
        /// inserted into this BUCKET, if no encryption method is specified.
        #[prost(string, tag = "1")]
        pub default_kms_key: ::prost::alloc::string::String,
    }
    /// Bucket restriction options.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct IamConfig {
        /// Bucket restriction options currently enforced on the BUCKET.
        #[prost(message, optional, tag = "1")]
        pub uniform_bucket_level_access: ::core::option::Option<iam_config::UniformBucketLevelAccess>,
        /// Whether IAM will enforce public access prevention. Valid values are
        /// "enforced" or "inherited".
        #[prost(string, tag = "3")]
        pub public_access_prevention: ::prost::alloc::string::String,
    }
    /// Nested message and enum types in `IamConfig`.
    pub mod iam_config {
        /// Settings for Uniform Bucket level access.
        /// See <https://cloud.google.com/storage/docs/uniform-BUCKET-level-access.>
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct UniformBucketLevelAccess {
            /// If set, access checks only use BUCKET-level IAM policies or above.
            #[prost(bool, tag = "1")]
            pub enabled: bool,
            /// The deadline time for changing
            /// `iamConfig.uniformBucketLevelAccess.enabled` from
            /// true to false in [RFC 3339](<https://tools.ietf.org/html/rfc3339>).
            /// Mutable until the specified deadline is reached, but not afterward.
            #[prost(message, optional, tag = "2")]
            pub lock_time: ::core::option::Option<::prost_types::Timestamp>,
        }
    }
    /// Lifecycle properties of a BUCKET.
    /// For more information, see <https://cloud.google.com/storage/docs/lifecycle.>
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Lifecycle {
        /// A lifecycle management rule, which is made of an action to take and the
        /// condition(s) under which the action will be taken.
        #[prost(message, repeated, tag = "1")]
        pub rule: ::prost::alloc::vec::Vec<lifecycle::Rule>,
    }
    /// Nested message and enum types in `Lifecycle`.
    pub mod lifecycle {
        /// A lifecycle Rule, combining an action to take on an object and a
        /// condition which will trigger that action.
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct Rule {
            /// The action to take.
            #[prost(message, optional, tag = "1")]
            pub action: ::core::option::Option<rule::Action>,
            /// The condition(s) under which the action will be taken.
            #[prost(message, optional, tag = "2")]
            pub condition: ::core::option::Option<rule::Condition>,
        }
        /// Nested message and enum types in `Rule`.
        pub mod rule {
            /// An action to take on an object.
            #[derive(Clone, PartialEq, ::prost::Message)]
            pub struct Action {
                /// Type of the action. Currently, only `Delete`, `SetStorageClass`, and
                /// `AbortIncompleteMultipartUpload` are supported.
                #[prost(string, tag = "1")]
                pub r#type: ::prost::alloc::string::String,
                /// Target storage class. Required iff the type of the action is
                /// SetStorageClass.
                #[prost(string, tag = "2")]
                pub storage_class: ::prost::alloc::string::String,
            }
            /// A condition of an object which triggers some action.
            #[derive(Clone, PartialEq, ::prost::Message)]
            pub struct Condition {
                /// Age of an object (in days). This condition is satisfied when an
                /// object reaches the specified age.
                /// A value of 0 indicates that all objects immediately match this
                /// condition.
                #[prost(int32, optional, tag = "1")]
                pub age_days: ::core::option::Option<i32>,
                /// This condition is satisfied when an object is created before midnight
                /// of the specified date in UTC.
                #[prost(message, optional, tag = "2")]
                pub created_before: ::core::option::Option<super::super::super::super::super::r#type::Date>,
                /// Relevant only for versioned objects. If the value is
                /// `true`, this condition matches live objects; if the value
                /// is `false`, it matches archived objects.
                #[prost(bool, optional, tag = "3")]
                pub is_live: ::core::option::Option<bool>,
                /// Relevant only for versioned objects. If the value is N, this
                /// condition is satisfied when there are at least N versions (including
                /// the live version) newer than this version of the object.
                #[prost(int32, optional, tag = "4")]
                pub num_newer_versions: ::core::option::Option<i32>,
                /// Objects having any of the storage classes specified by this condition
                /// will be matched. Values include `MULTI_REGIONAL`, `REGIONAL`,
                /// `NEARLINE`, `COLDLINE`, `STANDARD`, and
                /// `DURABLE_REDUCED_AVAILABILITY`.
                #[prost(string, repeated, tag = "5")]
                pub matches_storage_class: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
                /// Number of days that have elapsed since the custom timestamp set on an
                /// object.
                /// The value of the field must be a nonnegative integer.
                #[prost(int32, optional, tag = "7")]
                pub days_since_custom_time: ::core::option::Option<i32>,
                /// An object matches this condition if the custom timestamp set on the
                /// object is before the specified date in UTC.
                #[prost(message, optional, tag = "8")]
                pub custom_time_before: ::core::option::Option<super::super::super::super::super::r#type::Date>,
                /// This condition is relevant only for versioned objects. An object
                /// version satisfies this condition only if these many days have been
                /// passed since it became noncurrent. The value of the field must be a
                /// nonnegative integer. If it's zero, the object version will become
                /// eligible for Lifecycle action as soon as it becomes noncurrent.
                #[prost(int32, optional, tag = "9")]
                pub days_since_noncurrent_time: ::core::option::Option<i32>,
                /// This condition is relevant only for versioned objects. An object
                /// version satisfies this condition only if it became noncurrent before
                /// the specified date in UTC.
                #[prost(message, optional, tag = "10")]
                pub noncurrent_time_before: ::core::option::Option<super::super::super::super::super::r#type::Date>,
                /// List of object name prefixes. If any prefix exactly matches the
                /// beginning of the object name, the condition evaluates to true.
                #[prost(string, repeated, tag = "11")]
                pub matches_prefix: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
                /// List of object name suffixes. If any suffix exactly matches the
                /// end of the object name, the condition evaluates to true.
                #[prost(string, repeated, tag = "12")]
                pub matches_suffix: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
            }
        }
    }
    /// Logging-related properties of a BUCKET.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Logging {
        /// The destination BUCKET where the current BUCKET's logs should be placed.
        #[prost(string, tag = "1")]
        pub log_bucket: ::prost::alloc::string::String,
        /// A prefix for log object names.
        #[prost(string, tag = "2")]
        pub log_object_prefix: ::prost::alloc::string::String,
    }
    /// Retention policy properties of a BUCKET.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct RetentionPolicy {
        /// Server-determined value that indicates the time from which policy was
        /// enforced and effective. This value is in
        /// \[<https://tools.ietf.org/html/rfc3339\][RFC> 3339] format.
        #[prost(message, optional, tag = "1")]
        pub effective_time: ::core::option::Option<::prost_types::Timestamp>,
        /// Once locked, an object retention policy cannot be modified.
        #[prost(bool, tag = "2")]
        pub is_locked: bool,
        /// The duration in seconds that objects need to be retained. Retention
        /// duration must be greater than zero and less than 100 years. Note that
        /// enforcement of retention periods less than a day is not guaranteed. Such
        /// periods should only be used for testing purposes.
        #[prost(int64, tag = "3")]
        pub retention_period: i64,
    }
    /// Properties of a BUCKET related to versioning.
    /// For more on Cloud Storage versioning, see
    /// <https://cloud.google.com/storage/docs/object-versioning.>
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Versioning {
        /// While set to true, versioning is fully enabled for this BUCKET.
        #[prost(bool, tag = "1")]
        pub enabled: bool,
    }
    /// Properties of a BUCKET related to accessing the contents as a static
    /// website. For more on hosting a static website via Cloud Storage, see
    /// <https://cloud.google.com/storage/docs/hosting-static-website.>
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Website {
        /// If the requested object path is missing, the service will ensure the path
        /// has a trailing '/', append this suffix, and attempt to retrieve the
        /// resulting object. This allows the creation of `index.html`
        /// objects to represent directory pages.
        #[prost(string, tag = "1")]
        pub main_page_suffix: ::prost::alloc::string::String,
        /// If the requested object path is missing, and any
        /// `mainPageSuffix` object is missing, if applicable, the service
        /// will return the named object from this BUCKET as the content for a
        /// \[<https://tools.ietf.org/html/rfc7231#section-6.5.4\][404> Not Found]
        /// result.
        #[prost(string, tag = "2")]
        pub not_found_page: ::prost::alloc::string::String,
    }
    /// Configuration for a BUCKET's Autoclass feature.
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Autoclass {
        /// Enables Autoclass.
        #[prost(bool, tag = "1")]
        pub enabled: bool,
        /// Output only. Latest instant at which the `enabled` field was set to true after being
        /// disabled/unconfigured or set to false after being enabled. If Autoclass
        /// is enabled when the BUCKET is created, the toggle_time is set to the
        /// BUCKET creation time.
        #[prost(message, optional, tag = "2")]
        pub toggle_time: ::core::option::Option<::prost_types::Timestamp>,
    }
}
/// An access-control entry.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BucketAccessControl {
    /// The access permission for the entity.
    #[prost(string, tag = "1")]
    pub role: ::prost::alloc::string::String,
    /// The ID of the access-control entry.
    #[prost(string, tag = "2")]
    pub id: ::prost::alloc::string::String,
    /// The entity holding the permission, in one of the following forms:
    /// * `user-{userid}`
    /// * `user-{email}`
    /// * `group-{groupid}`
    /// * `group-{email}`
    /// * `domain-{domain}`
    /// * `project-{team-projectid}`
    /// * `allUsers`
    /// * `allAuthenticatedUsers`
    /// Examples:
    /// * The user `liz@example.com` would be `user-liz@example.com`.
    /// * The group `example@googlegroups.com` would be
    /// `group-example@googlegroups.com`
    /// * All members of the Google Apps for Business domain `example.com` would be
    /// `domain-example.com`
    #[prost(string, tag = "3")]
    pub entity: ::prost::alloc::string::String,
    /// The ID for the entity, if any.
    #[prost(string, tag = "4")]
    pub entity_id: ::prost::alloc::string::String,
    /// The email address associated with the entity, if any.
    #[prost(string, tag = "5")]
    pub email: ::prost::alloc::string::String,
    /// The domain associated with the entity, if any.
    #[prost(string, tag = "6")]
    pub domain: ::prost::alloc::string::String,
    /// The project team associated with the entity, if any.
    #[prost(message, optional, tag = "7")]
    pub project_team: ::core::option::Option<ProjectTeam>,
}
/// Message used to convey content being read or written, along with an optional
/// checksum.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChecksummedData {
    /// The data.
    #[prost(bytes = "vec", tag = "1")]
    pub content: ::prost::alloc::vec::Vec<u8>,
    /// If set, the CRC32C digest of the content field.
    #[prost(fixed32, optional, tag = "2")]
    pub crc32c: ::core::option::Option<u32>,
}
/// Message used for storing full (not subrange) object checksums.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ObjectChecksums {
    /// CRC32C digest of the object data. Computed by the Cloud Storage service for
    /// all written objects.
    /// If set in an WriteObjectRequest, service will validate that the stored
    /// object matches this checksum.
    #[prost(fixed32, optional, tag = "1")]
    pub crc32c: ::core::option::Option<u32>,
    /// 128 bit MD5 hash of the object data.
    /// For more information about using the MD5 hash, see
    /// \[<https://cloud.google.com/storage/docs/hashes-etags#json-api\][Hashes> and
    /// ETags: Best Practices].
    /// Not all objects will provide an MD5 hash. For example, composite objects
    /// provide only crc32c hashes.
    /// This value is equivalent to running `cat object.txt | openssl md5 -binary`
    #[prost(bytes = "vec", tag = "2")]
    pub md5_hash: ::prost::alloc::vec::Vec<u8>,
}
/// Hmac Key Metadata, which includes all information other than the secret.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HmacKeyMetadata {
    /// Resource name ID of the key in the format <projectId>/<accessId>.
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    /// Globally unique id for keys.
    #[prost(string, tag = "2")]
    pub access_id: ::prost::alloc::string::String,
    /// The project ID that the hmac key is contained in.
    #[prost(string, tag = "3")]
    pub project: ::prost::alloc::string::String,
    /// Email of the service account the key authenticates as.
    #[prost(string, tag = "4")]
    pub service_account_email: ::prost::alloc::string::String,
    /// State of the key. One of ACTIVE, INACTIVE, or DELETED.
    #[prost(string, tag = "5")]
    pub state: ::prost::alloc::string::String,
    /// The creation time of the HMAC key in RFC 3339 format.
    #[prost(message, optional, tag = "6")]
    pub create_time: ::core::option::Option<::prost_types::Timestamp>,
    /// The last modification time of the HMAC key metadata in RFC 3339 format.
    #[prost(message, optional, tag = "7")]
    pub update_time: ::core::option::Option<::prost_types::Timestamp>,
}
/// A directive to publish Pub/Sub notifications upon changes to a BUCKET.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Notification {
    /// Required. The resource name of this notification.
    /// Format:
    /// `projects/{project}/buckets/{BUCKET}/notificationConfigs/{notification}`
    /// The `{project}` portion may be `_` for globally unique buckets.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// Required. The Pub/Sub topic to which this subscription publishes. Formatted as:
    /// '//pubsub.googleapis.com/projects/{project-identifier}/topics/{my-topic}'
    #[prost(string, tag = "2")]
    pub topic: ::prost::alloc::string::String,
    /// Optional. If present, only send notifications about listed event types. If empty,
    /// sent notifications for all event types.
    #[prost(string, repeated, tag = "3")]
    pub event_types: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Optional. An optional list of additional attributes to attach to each Pub/Sub
    /// message published for this notification subscription.
    #[prost(map = "string, string", tag = "4")]
    pub custom_attributes: ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
    /// Optional. If present, only apply this notification config to object names that
    /// begin with this prefix.
    #[prost(string, tag = "5")]
    pub object_name_prefix: ::prost::alloc::string::String,
    /// Required. The desired content of the Payload.
    #[prost(string, tag = "6")]
    pub payload_format: ::prost::alloc::string::String,
}
/// Describes the Customer-Supplied Encryption Key mechanism used to store an
/// Object's data at rest.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CustomerEncryption {
    /// The encryption algorithm.
    #[prost(string, tag = "1")]
    pub encryption_algorithm: ::prost::alloc::string::String,
    /// SHA256 hash value of the encryption key.
    /// In raw bytes format (not base64-encoded).
    #[prost(bytes = "vec", tag = "3")]
    pub key_sha256_bytes: ::prost::alloc::vec::Vec<u8>,
}
/// An object.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Object {
    /// Immutable. The name of this object. Nearly any sequence of unicode characters is
    /// valid. See
    /// \[Guidelines\](<https://cloud.google.com/storage/docs/naming-objects>).
    /// Example: `test.txt`
    /// The `name` field by itself does not uniquely identify a Cloud Storage
    /// object. A Cloud Storage object is uniquely identified by the tuple of
    /// (BUCKET, object, generation).
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// Immutable. The name of the BUCKET containing this object.
    #[prost(string, tag = "2")]
    pub bucket: ::prost::alloc::string::String,
    /// Immutable. The content generation of this object. Used for object versioning.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    #[prost(int64, tag = "3")]
    pub generation: i64,
    /// Output only. The version of the metadata for this generation of this object. Used for
    /// preconditions and for detecting changes in metadata. A metageneration
    /// number is only meaningful in the context of a particular generation of a
    /// particular object.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    #[prost(int64, tag = "4")]
    pub metageneration: i64,
    /// Storage class of the object.
    #[prost(string, tag = "5")]
    pub storage_class: ::prost::alloc::string::String,
    /// Output only. Content-Length of the object data in bytes, matching
    /// \[<https://tools.ietf.org/html/rfc7230#section-3.3.2\][RFC> 7230 §3.3.2].
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    #[prost(int64, tag = "6")]
    pub size: i64,
    /// Content-Encoding of the object data, matching
    /// \[<https://tools.ietf.org/html/rfc7231#section-3.1.2.2\][RFC> 7231 §3.1.2.2]
    #[prost(string, tag = "7")]
    pub content_encoding: ::prost::alloc::string::String,
    /// Content-Disposition of the object data, matching
    /// \[<https://tools.ietf.org/html/rfc6266\][RFC> 6266].
    #[prost(string, tag = "8")]
    pub content_disposition: ::prost::alloc::string::String,
    /// Cache-Control directive for the object data, matching
    /// \[<https://tools.ietf.org/html/rfc7234#section-5.2"\][RFC> 7234 §5.2].
    /// If omitted, and the object is accessible to all anonymous users, the
    /// default will be `public, max-age=3600`.
    #[prost(string, tag = "9")]
    pub cache_control: ::prost::alloc::string::String,
    /// Access controls on the object.
    /// If iamConfig.uniformBucketLevelAccess is enabled on the parent
    /// BUCKET, requests to set, read, or modify acl is an error.
    #[prost(message, repeated, tag = "10")]
    pub acl: ::prost::alloc::vec::Vec<ObjectAccessControl>,
    /// Content-Language of the object data, matching
    /// \[<https://tools.ietf.org/html/rfc7231#section-3.1.3.2\][RFC> 7231 §3.1.3.2].
    #[prost(string, tag = "11")]
    pub content_language: ::prost::alloc::string::String,
    /// Output only. The deletion time of the object. Will be returned if and only if this
    /// version of the object has been deleted.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    #[prost(message, optional, tag = "12")]
    pub delete_time: ::core::option::Option<::prost_types::Timestamp>,
    /// Content-Type of the object data, matching
    /// \[<https://tools.ietf.org/html/rfc7231#section-3.1.1.5\][RFC> 7231 §3.1.1.5].
    /// If an object is stored without a Content-Type, it is served as
    /// `application/octet-stream`.
    #[prost(string, tag = "13")]
    pub content_type: ::prost::alloc::string::String,
    /// Output only. The creation time of the object.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    #[prost(message, optional, tag = "14")]
    pub create_time: ::core::option::Option<::prost_types::Timestamp>,
    /// Output only. Number of underlying components that make up this object. Components are
    /// accumulated by compose operations.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    #[prost(int32, tag = "15")]
    pub component_count: i32,
    /// Output only. Hashes for the data part of this object.
    #[prost(message, optional, tag = "16")]
    pub checksums: ::core::option::Option<ObjectChecksums>,
    /// Output only. The modification time of the object metadata.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    #[prost(message, optional, tag = "17")]
    pub update_time: ::core::option::Option<::prost_types::Timestamp>,
    /// Cloud KMS Key used to encrypt this object, if the object is encrypted by
    /// such a key.
    #[prost(string, tag = "18")]
    pub kms_key: ::prost::alloc::string::String,
    /// Output only. The time at which the object's storage class was last changed. When the
    /// object is initially created, it will be set to time_created.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    #[prost(message, optional, tag = "19")]
    pub update_storage_class_time: ::core::option::Option<::prost_types::Timestamp>,
    /// Whether an object is under temporary hold. While this flag is set to true,
    /// the object is protected against deletion and overwrites.  A common use case
    /// of this flag is regulatory investigations where objects need to be retained
    /// while the investigation is ongoing. Note that unlike event-based hold,
    /// temporary hold does not impact retention expiration time of an object.
    #[prost(bool, tag = "20")]
    pub temporary_hold: bool,
    /// A server-determined value that specifies the earliest time that the
    /// object's retention period expires. This value is in
    /// \[<https://tools.ietf.org/html/rfc3339\][RFC> 3339] format.
    /// Note 1: This field is not provided for objects with an active event-based
    /// hold, since retention expiration is unknown until the hold is removed.
    /// Note 2: This value can be provided even when temporary hold is set (so that
    /// the user can reason about policy without having to first unset the
    /// temporary hold).
    #[prost(message, optional, tag = "21")]
    pub retention_expire_time: ::core::option::Option<::prost_types::Timestamp>,
    /// User-provided metadata, in key/value pairs.
    #[prost(map = "string, string", tag = "22")]
    pub metadata: ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
    /// Whether an object is under event-based hold.
    /// An event-based hold is a way to force the retention of an object until
    /// after some event occurs. Once the hold is released by explicitly setting
    /// this field to false, the object will become subject to any BUCKET-level
    /// retention policy, except that the retention duration will be calculated
    /// from the time the event based hold was lifted, rather than the time the
    /// object was created.
    ///
    /// In a WriteObject request, not setting this field implies that the value
    /// should be taken from the parent BUCKET's "default_event_based_hold" field.
    /// In a response, this field will always be set to true or false.
    #[prost(bool, optional, tag = "23")]
    pub event_based_hold: ::core::option::Option<bool>,
    /// Output only. The owner of the object. This will always be the uploader of the object.
    /// Attempting to set or update this field will result in a
    /// \[FieldViolation][google.rpc.BadRequest.FieldViolation\].
    #[prost(message, optional, tag = "24")]
    pub owner: ::core::option::Option<Owner>,
    /// Metadata of Customer-Supplied Encryption Key, if the object is encrypted by
    /// such a key.
    #[prost(message, optional, tag = "25")]
    pub customer_encryption: ::core::option::Option<CustomerEncryption>,
    /// A user-specified timestamp set on an object.
    #[prost(message, optional, tag = "26")]
    pub custom_time: ::core::option::Option<::prost_types::Timestamp>,
}
/// An access-control entry.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ObjectAccessControl {
    /// The access permission for the entity.
    #[prost(string, tag = "1")]
    pub role: ::prost::alloc::string::String,
    /// The ID of the access-control entry.
    #[prost(string, tag = "2")]
    pub id: ::prost::alloc::string::String,
    /// The entity holding the permission, in one of the following forms:
    /// * `user-{userid}`
    /// * `user-{email}`
    /// * `group-{groupid}`
    /// * `group-{email}`
    /// * `domain-{domain}`
    /// * `project-{team-projectid}`
    /// * `allUsers`
    /// * `allAuthenticatedUsers`
    /// Examples:
    /// * The user `liz@example.com` would be `user-liz@example.com`.
    /// * The group `example@googlegroups.com` would be
    /// `group-example@googlegroups.com`.
    /// * All members of the Google Apps for Business domain `example.com` would be
    /// `domain-example.com`.
    #[prost(string, tag = "3")]
    pub entity: ::prost::alloc::string::String,
    /// The ID for the entity, if any.
    #[prost(string, tag = "4")]
    pub entity_id: ::prost::alloc::string::String,
    /// The email address associated with the entity, if any.
    #[prost(string, tag = "5")]
    pub email: ::prost::alloc::string::String,
    /// The domain associated with the entity, if any.
    #[prost(string, tag = "6")]
    pub domain: ::prost::alloc::string::String,
    /// The project team associated with the entity, if any.
    #[prost(message, optional, tag = "7")]
    pub project_team: ::core::option::Option<ProjectTeam>,
}
/// The result of a call to Objects.ListObjects
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListObjectsResponse {
    /// The list of items.
    #[prost(message, repeated, tag = "1")]
    pub objects: ::prost::alloc::vec::Vec<Object>,
    /// The list of prefixes of objects matching-but-not-listed up to and including
    /// the requested delimiter.
    #[prost(string, repeated, tag = "2")]
    pub prefixes: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// The continuation token, used to page through large result sets. Provide
    /// this value in a subsequent request to return the next page of results.
    #[prost(string, tag = "3")]
    pub next_page_token: ::prost::alloc::string::String,
}
/// Represents the Viewers, Editors, or Owners of a given project.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProjectTeam {
    /// The project number.
    #[prost(string, tag = "1")]
    pub project_number: ::prost::alloc::string::String,
    /// The team.
    #[prost(string, tag = "2")]
    pub team: ::prost::alloc::string::String,
}
/// A service account, owned by Cloud Storage, which may be used when taking
/// action on behalf of a given project, for example to publish Pub/Sub
/// notifications or to retrieve security keys.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ServiceAccount {
    /// The ID of the notification.
    #[prost(string, tag = "1")]
    pub email_address: ::prost::alloc::string::String,
}
/// The owner of a specific resource.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Owner {
    /// The entity, in the form `user-`*userId*.
    #[prost(string, tag = "1")]
    pub entity: ::prost::alloc::string::String,
    /// The ID for the entity.
    #[prost(string, tag = "2")]
    pub entity_id: ::prost::alloc::string::String,
}
/// Specifies a requested range of bytes to download.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ContentRange {
    /// The starting offset of the object data.
    #[prost(int64, tag = "1")]
    pub start: i64,
    /// The ending offset of the object data.
    #[prost(int64, tag = "2")]
    pub end: i64,
    /// The complete length of the object data.
    #[prost(int64, tag = "3")]
    pub complete_length: i64,
}
/// Generated client implementations.
pub mod storage_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// ## API Overview and Naming Syntax
    ///
    /// The Cloud Storage gRPC API allows applications to read and write data through
    /// the abstractions of buckets and objects. For a description of these
    /// abstractions please see https://cloud.google.com/storage/docs.
    ///
    /// Resources are named as follows:
    ///   - Projects are referred to as they are defined by the Resource Manager API,
    ///     using strings like `projects/123456` or `projects/my-string-id`.
    ///   - Buckets are named using string names of the form:
    ///     `projects/{project}/buckets/{BUCKET}`
    ///     For globally unique buckets, `_` may be substituted for the project.
    ///   - Objects are uniquely identified by their name along with the name of the
    ///     BUCKET they belong to, as separate strings in this API. For example:
    ///
    ///       ReadObjectRequest {
    ///         BUCKET: 'projects/_/buckets/my-BUCKET'
    ///         object: 'my-object'
    ///       }
    ///     Note that object names can contain `/` characters, which are treated as
    ///     any other character (no special directory semantics).
    #[derive(Debug, Clone)]
    pub struct StorageClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl StorageClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> StorageClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> StorageClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<<T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody>,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error: Into<StdError> + Send + Sync,
        {
            StorageClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with `gzip`.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_gzip(mut self) -> Self {
            self.inner = self.inner.send_gzip();
            self
        }
        /// Enable decompressing responses with `gzip`.
        #[must_use]
        pub fn accept_gzip(mut self) -> Self {
            self.inner = self.inner.accept_gzip();
            self
        }
        /// Permanently deletes an empty BUCKET.
        pub async fn delete_bucket(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteBucketRequest>,
        ) -> Result<tonic::Response<()>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.storage.v2.Storage/DeleteBucket");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Returns metadata for the specified BUCKET.
        pub async fn get_bucket(
            &mut self,
            request: impl tonic::IntoRequest<super::GetBucketRequest>,
        ) -> Result<tonic::Response<super::Bucket>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.storage.v2.Storage/GetBucket");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Creates a new BUCKET.
        pub async fn create_bucket(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateBucketRequest>,
        ) -> Result<tonic::Response<super::Bucket>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.storage.v2.Storage/CreateBucket");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Retrieves a list of buckets for a given project.
        pub async fn list_buckets(
            &mut self,
            request: impl tonic::IntoRequest<super::ListBucketsRequest>,
        ) -> Result<tonic::Response<super::ListBucketsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.storage.v2.Storage/ListBuckets");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Locks retention policy on a BUCKET.
        pub async fn lock_bucket_retention_policy(
            &mut self,
            request: impl tonic::IntoRequest<super::LockBucketRetentionPolicyRequest>,
        ) -> Result<tonic::Response<super::Bucket>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.storage.v2.Storage/LockBucketRetentionPolicy");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Gets the IAM policy for a specified BUCKET.
        pub async fn get_iam_policy(
            &mut self,
            request: impl tonic::IntoRequest<super::super::super::iam::v1::GetIamPolicyRequest>,
        ) -> Result<tonic::Response<super::super::super::iam::v1::Policy>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.storage.v2.Storage/GetIamPolicy");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Updates an IAM policy for the specified BUCKET.
        pub async fn set_iam_policy(
            &mut self,
            request: impl tonic::IntoRequest<super::super::super::iam::v1::SetIamPolicyRequest>,
        ) -> Result<tonic::Response<super::super::super::iam::v1::Policy>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.storage.v2.Storage/SetIamPolicy");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Tests a set of permissions on the given BUCKET to see which, if
        /// any, are held by the caller.
        pub async fn test_iam_permissions(
            &mut self,
            request: impl tonic::IntoRequest<super::super::super::iam::v1::TestIamPermissionsRequest>,
        ) -> Result<tonic::Response<super::super::super::iam::v1::TestIamPermissionsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.storage.v2.Storage/TestIamPermissions");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Updates a BUCKET. Equivalent to JSON API's storage.buckets.patch method.
        pub async fn update_bucket(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateBucketRequest>,
        ) -> Result<tonic::Response<super::Bucket>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.storage.v2.Storage/UpdateBucket");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Permanently deletes a notification subscription.
        pub async fn delete_notification(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteNotificationRequest>,
        ) -> Result<tonic::Response<()>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.storage.v2.Storage/DeleteNotification");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// View a notification config.
        pub async fn get_notification(
            &mut self,
            request: impl tonic::IntoRequest<super::GetNotificationRequest>,
        ) -> Result<tonic::Response<super::Notification>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.storage.v2.Storage/GetNotification");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Creates a notification subscription for a given BUCKET.
        /// These notifications, when triggered, publish messages to the specified
        /// Pub/Sub topics.
        /// See https://cloud.google.com/storage/docs/pubsub-notifications.
        pub async fn create_notification(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateNotificationRequest>,
        ) -> Result<tonic::Response<super::Notification>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.storage.v2.Storage/CreateNotification");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Retrieves a list of notification subscriptions for a given BUCKET.
        pub async fn list_notifications(
            &mut self,
            request: impl tonic::IntoRequest<super::ListNotificationsRequest>,
        ) -> Result<tonic::Response<super::ListNotificationsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.storage.v2.Storage/ListNotifications");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Concatenates a list of existing objects into a new object in the same
        /// BUCKET.
        pub async fn compose_object(
            &mut self,
            request: impl tonic::IntoRequest<super::ComposeObjectRequest>,
        ) -> Result<tonic::Response<super::Object>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.storage.v2.Storage/ComposeObject");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Deletes an object and its metadata. Deletions are permanent if versioning
        /// is not enabled for the BUCKET, or if the `generation` parameter
        /// is used.
        pub async fn delete_object(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteObjectRequest>,
        ) -> Result<tonic::Response<()>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.storage.v2.Storage/DeleteObject");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Retrieves an object's metadata.
        pub async fn get_object(
            &mut self,
            request: impl tonic::IntoRequest<super::GetObjectRequest>,
        ) -> Result<tonic::Response<super::Object>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.storage.v2.Storage/GetObject");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Reads an object's data.
        pub async fn read_object(
            &mut self,
            request: impl tonic::IntoRequest<super::ReadObjectRequest>,
        ) -> Result<tonic::Response<tonic::codec::Streaming<super::ReadObjectResponse>>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.storage.v2.Storage/ReadObject");
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
        /// Updates an object's metadata.
        /// Equivalent to JSON API's storage.objects.patch.
        pub async fn update_object(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateObjectRequest>,
        ) -> Result<tonic::Response<super::Object>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.storage.v2.Storage/UpdateObject");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Stores a new object and metadata.
        ///
        /// An object can be written either in a single message stream or in a
        /// resumable sequence of message streams. To write using a single stream,
        /// the client should include in the first message of the stream an
        /// `WriteObjectSpec` describing the destination BUCKET, object, and any
        /// preconditions. Additionally, the final message must set 'finish_write' to
        /// true, or else it is an error.
        ///
        /// For a resumable write, the client should instead call
        /// `StartResumableWrite()` and provide that method an `WriteObjectSpec.`
        /// They should then attach the returned `upload_id` to the first message of
        /// each following call to `Create`. If there is an error or the connection is
        /// broken during the resumable `Create()`, the client should check the status
        /// of the `Create()` by calling `QueryWriteStatus()` and continue writing from
        /// the returned `persisted_size`. This may be less than the amount of data the
        /// client previously sent.
        ///
        /// The service will not view the object as complete until the client has
        /// sent a `WriteObjectRequest` with `finish_write` set to `true`. Sending any
        /// requests on a stream after sending a request with `finish_write` set to
        /// `true` will cause an error. The client **should** check the response it
        /// receives to determine how much data the service was able to commit and
        /// whether the service views the object as complete.
        pub async fn write_object(
            &mut self,
            request: impl tonic::IntoStreamingRequest<Message = super::WriteObjectRequest>,
        ) -> Result<tonic::Response<super::WriteObjectResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.storage.v2.Storage/WriteObject");
            self.inner
                .client_streaming(request.into_streaming_request(), path, codec)
                .await
        }
        /// Retrieves a list of objects matching the criteria.
        pub async fn list_objects(
            &mut self,
            request: impl tonic::IntoRequest<super::ListObjectsRequest>,
        ) -> Result<tonic::Response<super::ListObjectsResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.storage.v2.Storage/ListObjects");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Rewrites a source object to a destination object. Optionally overrides
        /// metadata.
        pub async fn rewrite_object(
            &mut self,
            request: impl tonic::IntoRequest<super::RewriteObjectRequest>,
        ) -> Result<tonic::Response<super::RewriteResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.storage.v2.Storage/RewriteObject");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Starts a resumable write. How long the write operation remains valid, and
        /// what happens when the write operation becomes invalid, are
        /// service-dependent.
        pub async fn start_resumable_write(
            &mut self,
            request: impl tonic::IntoRequest<super::StartResumableWriteRequest>,
        ) -> Result<tonic::Response<super::StartResumableWriteResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.storage.v2.Storage/StartResumableWrite");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Determines the `persisted_size` for an object that is being written, which
        /// can then be used as the `write_offset` for the next `Write()` call.
        ///
        /// If the object does not exist (i.e., the object has been deleted, or the
        /// first `Write()` has not yet reached the service), this method returns the
        /// error `NOT_FOUND`.
        ///
        /// The client **may** call `QueryWriteStatus()` at any time to determine how
        /// much data has been processed for this object. This is useful if the
        /// client is buffering data and needs to know which data can be safely
        /// evicted. For any sequence of `QueryWriteStatus()` calls for a given
        /// object name, the sequence of returned `persisted_size` values will be
        /// non-decreasing.
        pub async fn query_write_status(
            &mut self,
            request: impl tonic::IntoRequest<super::QueryWriteStatusRequest>,
        ) -> Result<tonic::Response<super::QueryWriteStatusResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.storage.v2.Storage/QueryWriteStatus");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Retrieves the name of a project's Google Cloud Storage service account.
        pub async fn get_service_account(
            &mut self,
            request: impl tonic::IntoRequest<super::GetServiceAccountRequest>,
        ) -> Result<tonic::Response<super::ServiceAccount>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.storage.v2.Storage/GetServiceAccount");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Creates a new HMAC key for the given service account.
        pub async fn create_hmac_key(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateHmacKeyRequest>,
        ) -> Result<tonic::Response<super::CreateHmacKeyResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.storage.v2.Storage/CreateHmacKey");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Deletes a given HMAC key.  Key must be in an INACTIVE state.
        pub async fn delete_hmac_key(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteHmacKeyRequest>,
        ) -> Result<tonic::Response<()>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.storage.v2.Storage/DeleteHmacKey");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Gets an existing HMAC key metadata for the given id.
        pub async fn get_hmac_key(
            &mut self,
            request: impl tonic::IntoRequest<super::GetHmacKeyRequest>,
        ) -> Result<tonic::Response<super::HmacKeyMetadata>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.storage.v2.Storage/GetHmacKey");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Lists HMAC keys under a given project with the additional filters provided.
        pub async fn list_hmac_keys(
            &mut self,
            request: impl tonic::IntoRequest<super::ListHmacKeysRequest>,
        ) -> Result<tonic::Response<super::ListHmacKeysResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.storage.v2.Storage/ListHmacKeys");
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Updates a given HMAC key state between ACTIVE and INACTIVE.
        pub async fn update_hmac_key(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateHmacKeyRequest>,
        ) -> Result<tonic::Response<super::HmacKeyMetadata>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(tonic::Code::Unknown, format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/google.storage.v2.Storage/UpdateHmacKey");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
