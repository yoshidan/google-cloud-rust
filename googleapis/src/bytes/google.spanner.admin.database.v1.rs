/// Encapsulates progress related information for a Cloud Spanner long
/// running operation.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OperationProgress {
    /// Percent completion of the operation.
    /// Values are between 0 and 100 inclusive.
    #[prost(int32, tag = "1")]
    pub progress_percent: i32,
    /// Time the request was received.
    #[prost(message, optional, tag = "2")]
    pub start_time: ::core::option::Option<::prost_types::Timestamp>,
    /// If set, the time at which this operation failed or was completed
    /// successfully.
    #[prost(message, optional, tag = "3")]
    pub end_time: ::core::option::Option<::prost_types::Timestamp>,
}
/// Encryption configuration for a Cloud Spanner database.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EncryptionConfig {
    /// The Cloud KMS key to be used for encrypting and decrypting
    /// the database. Values are of the form
    /// `projects/<project>/locations/<location>/keyRings/<key_ring>/cryptoKeys/<kms_key_name>`.
    #[prost(string, tag = "2")]
    pub kms_key_name: ::prost::alloc::string::String,
}
/// Encryption information for a Cloud Spanner database or backup.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EncryptionInfo {
    /// Output only. The type of encryption.
    #[prost(enumeration = "encryption_info::Type", tag = "3")]
    pub encryption_type: i32,
    /// Output only. If present, the status of a recent encrypt/decrypt call on underlying data
    /// for this database or backup. Regardless of status, data is always encrypted
    /// at rest.
    #[prost(message, optional, tag = "4")]
    pub encryption_status: ::core::option::Option<
        super::super::super::super::rpc::Status,
    >,
    /// Output only. A Cloud KMS key version that is being used to protect the database or
    /// backup.
    #[prost(string, tag = "2")]
    pub kms_key_version: ::prost::alloc::string::String,
}
/// Nested message and enum types in `EncryptionInfo`.
pub mod encryption_info {
    /// Possible encryption types.
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum Type {
        /// Encryption type was not specified, though data at rest remains encrypted.
        Unspecified = 0,
        /// The data is encrypted at rest with a key that is
        /// fully managed by Google. No key version or status will be populated.
        /// This is the default state.
        GoogleDefaultEncryption = 1,
        /// The data is encrypted at rest with a key that is
        /// managed by the customer. The active version of the key. `kms_key_version`
        /// will be populated, and `encryption_status` may be populated.
        CustomerManagedEncryption = 2,
    }
    impl Type {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Type::Unspecified => "TYPE_UNSPECIFIED",
                Type::GoogleDefaultEncryption => "GOOGLE_DEFAULT_ENCRYPTION",
                Type::CustomerManagedEncryption => "CUSTOMER_MANAGED_ENCRYPTION",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "TYPE_UNSPECIFIED" => Some(Self::Unspecified),
                "GOOGLE_DEFAULT_ENCRYPTION" => Some(Self::GoogleDefaultEncryption),
                "CUSTOMER_MANAGED_ENCRYPTION" => Some(Self::CustomerManagedEncryption),
                _ => None,
            }
        }
    }
}
/// Indicates the dialect type of a database.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum DatabaseDialect {
    /// Default value. This value will create a database with the
    /// GOOGLE_STANDARD_SQL dialect.
    Unspecified = 0,
    /// Google standard SQL.
    GoogleStandardSql = 1,
    /// PostgreSQL supported SQL.
    Postgresql = 2,
}
impl DatabaseDialect {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            DatabaseDialect::Unspecified => "DATABASE_DIALECT_UNSPECIFIED",
            DatabaseDialect::GoogleStandardSql => "GOOGLE_STANDARD_SQL",
            DatabaseDialect::Postgresql => "POSTGRESQL",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "DATABASE_DIALECT_UNSPECIFIED" => Some(Self::Unspecified),
            "GOOGLE_STANDARD_SQL" => Some(Self::GoogleStandardSql),
            "POSTGRESQL" => Some(Self::Postgresql),
            _ => None,
        }
    }
}
/// A backup of a Cloud Spanner database.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Backup {
    /// Required for the \[CreateBackup][google.spanner.admin.database.v1.DatabaseAdmin.CreateBackup\] operation.
    /// Name of the database from which this backup was
    /// created. This needs to be in the same instance as the backup.
    /// Values are of the form
    /// `projects/<project>/instances/<instance>/databases/<database>`.
    #[prost(string, tag = "2")]
    pub database: ::prost::alloc::string::String,
    /// The backup will contain an externally consistent copy of the database at
    /// the timestamp specified by `version_time`. If `version_time` is not
    /// specified, the system will set `version_time` to the `create_time` of the
    /// backup.
    #[prost(message, optional, tag = "9")]
    pub version_time: ::core::option::Option<::prost_types::Timestamp>,
    /// Required for the \[CreateBackup][google.spanner.admin.database.v1.DatabaseAdmin.CreateBackup\]
    /// operation. The expiration time of the backup, with microseconds
    /// granularity that must be at least 6 hours and at most 366 days
    /// from the time the CreateBackup request is processed. Once the `expire_time`
    /// has passed, the backup is eligible to be automatically deleted by Cloud
    /// Spanner to free the resources used by the backup.
    #[prost(message, optional, tag = "3")]
    pub expire_time: ::core::option::Option<::prost_types::Timestamp>,
    /// Output only for the \[CreateBackup][google.spanner.admin.database.v1.DatabaseAdmin.CreateBackup\] operation.
    /// Required for the \[UpdateBackup][google.spanner.admin.database.v1.DatabaseAdmin.UpdateBackup\] operation.
    ///
    /// A globally unique identifier for the backup which cannot be
    /// changed. Values are of the form
    /// `projects/<project>/instances/<instance>/backups/\[a-z][a-z0-9_\-]*[a-z0-9\]`
    /// The final segment of the name must be between 2 and 60 characters
    /// in length.
    ///
    /// The backup is stored in the location(s) specified in the instance
    /// configuration of the instance containing the backup, identified
    /// by the prefix of the backup name of the form
    /// `projects/<project>/instances/<instance>`.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// Output only. The time the \[CreateBackup][google.spanner.admin.database.v1.DatabaseAdmin.CreateBackup\]
    /// request is received. If the request does not specify `version_time`, the
    /// `version_time` of the backup will be equivalent to the `create_time`.
    #[prost(message, optional, tag = "4")]
    pub create_time: ::core::option::Option<::prost_types::Timestamp>,
    /// Output only. Size of the backup in bytes.
    #[prost(int64, tag = "5")]
    pub size_bytes: i64,
    /// Output only. The current state of the backup.
    #[prost(enumeration = "backup::State", tag = "6")]
    pub state: i32,
    /// Output only. The names of the restored databases that reference the backup.
    /// The database names are of
    /// the form `projects/<project>/instances/<instance>/databases/<database>`.
    /// Referencing databases may exist in different instances. The existence of
    /// any referencing database prevents the backup from being deleted. When a
    /// restored database from the backup enters the `READY` state, the reference
    /// to the backup is removed.
    #[prost(string, repeated, tag = "7")]
    pub referencing_databases: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Output only. The encryption information for the backup.
    #[prost(message, optional, tag = "8")]
    pub encryption_info: ::core::option::Option<EncryptionInfo>,
    /// Output only. The database dialect information for the backup.
    #[prost(enumeration = "DatabaseDialect", tag = "10")]
    pub database_dialect: i32,
    /// Output only. The names of the destination backups being created by copying
    /// this source backup. The backup names are of the form
    /// `projects/<project>/instances/<instance>/backups/<backup>`.
    /// Referencing backups may exist in different instances. The existence of
    /// any referencing backup prevents the backup from being deleted. When the
    /// copy operation is done (either successfully completed or cancelled or the
    /// destination backup is deleted), the reference to the backup is removed.
    #[prost(string, repeated, tag = "11")]
    pub referencing_backups: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Output only. The max allowed expiration time of the backup, with
    /// microseconds granularity. A backup's expiration time can be configured in
    /// multiple APIs: CreateBackup, UpdateBackup, CopyBackup. When updating or
    /// copying an existing backup, the expiration time specified must be
    /// less than `Backup.max_expire_time`.
    #[prost(message, optional, tag = "12")]
    pub max_expire_time: ::core::option::Option<::prost_types::Timestamp>,
}
/// Nested message and enum types in `Backup`.
pub mod backup {
    /// Indicates the current state of the backup.
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum State {
        /// Not specified.
        Unspecified = 0,
        /// The pending backup is still being created. Operations on the
        /// backup may fail with `FAILED_PRECONDITION` in this state.
        Creating = 1,
        /// The backup is complete and ready for use.
        Ready = 2,
    }
    impl State {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                State::Unspecified => "STATE_UNSPECIFIED",
                State::Creating => "CREATING",
                State::Ready => "READY",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "STATE_UNSPECIFIED" => Some(Self::Unspecified),
                "CREATING" => Some(Self::Creating),
                "READY" => Some(Self::Ready),
                _ => None,
            }
        }
    }
}
/// The request for \[CreateBackup][google.spanner.admin.database.v1.DatabaseAdmin.CreateBackup\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateBackupRequest {
    /// Required. The name of the instance in which the backup will be
    /// created. This must be the same instance that contains the database the
    /// backup will be created from. The backup will be stored in the
    /// location(s) specified in the instance configuration of this
    /// instance. Values are of the form
    /// `projects/<project>/instances/<instance>`.
    #[prost(string, tag = "1")]
    pub parent: ::prost::alloc::string::String,
    /// Required. The id of the backup to be created. The `backup_id` appended to
    /// `parent` forms the full backup name of the form
    /// `projects/<project>/instances/<instance>/backups/<backup_id>`.
    #[prost(string, tag = "2")]
    pub backup_id: ::prost::alloc::string::String,
    /// Required. The backup to create.
    #[prost(message, optional, tag = "3")]
    pub backup: ::core::option::Option<Backup>,
    /// Optional. The encryption configuration used to encrypt the backup. If this field is
    /// not specified, the backup will use the same
    /// encryption configuration as the database by default, namely
    /// \[encryption_type][google.spanner.admin.database.v1.CreateBackupEncryptionConfig.encryption_type\] =
    /// `USE_DATABASE_ENCRYPTION`.
    #[prost(message, optional, tag = "4")]
    pub encryption_config: ::core::option::Option<CreateBackupEncryptionConfig>,
}
/// Metadata type for the operation returned by
/// \[CreateBackup][google.spanner.admin.database.v1.DatabaseAdmin.CreateBackup\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateBackupMetadata {
    /// The name of the backup being created.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// The name of the database the backup is created from.
    #[prost(string, tag = "2")]
    pub database: ::prost::alloc::string::String,
    /// The progress of the
    /// \[CreateBackup][google.spanner.admin.database.v1.DatabaseAdmin.CreateBackup\] operation.
    #[prost(message, optional, tag = "3")]
    pub progress: ::core::option::Option<OperationProgress>,
    /// The time at which cancellation of this operation was received.
    /// \[Operations.CancelOperation][google.longrunning.Operations.CancelOperation\]
    /// starts asynchronous cancellation on a long-running operation. The server
    /// makes a best effort to cancel the operation, but success is not guaranteed.
    /// Clients can use
    /// \[Operations.GetOperation][google.longrunning.Operations.GetOperation\] or
    /// other methods to check whether the cancellation succeeded or whether the
    /// operation completed despite cancellation. On successful cancellation,
    /// the operation is not deleted; instead, it becomes an operation with
    /// an \[Operation.error][google.longrunning.Operation.error\] value with a
    /// \[google.rpc.Status.code][google.rpc.Status.code\] of 1,
    /// corresponding to `Code.CANCELLED`.
    #[prost(message, optional, tag = "4")]
    pub cancel_time: ::core::option::Option<::prost_types::Timestamp>,
}
/// The request for \[CopyBackup][google.spanner.admin.database.v1.DatabaseAdmin.CopyBackup\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CopyBackupRequest {
    /// Required. The name of the destination instance that will contain the backup copy.
    /// Values are of the form: `projects/<project>/instances/<instance>`.
    #[prost(string, tag = "1")]
    pub parent: ::prost::alloc::string::String,
    /// Required. The id of the backup copy.
    /// The `backup_id` appended to `parent` forms the full backup_uri of the form
    /// `projects/<project>/instances/<instance>/backups/<backup>`.
    #[prost(string, tag = "2")]
    pub backup_id: ::prost::alloc::string::String,
    /// Required. The source backup to be copied.
    /// The source backup needs to be in READY state for it to be copied.
    /// Once CopyBackup is in progress, the source backup cannot be deleted or
    /// cleaned up on expiration until CopyBackup is finished.
    /// Values are of the form:
    /// `projects/<project>/instances/<instance>/backups/<backup>`.
    #[prost(string, tag = "3")]
    pub source_backup: ::prost::alloc::string::String,
    /// Required. The expiration time of the backup in microsecond granularity.
    /// The expiration time must be at least 6 hours and at most 366 days
    /// from the `create_time` of the source backup. Once the `expire_time` has
    /// passed, the backup is eligible to be automatically deleted by Cloud Spanner
    /// to free the resources used by the backup.
    #[prost(message, optional, tag = "4")]
    pub expire_time: ::core::option::Option<::prost_types::Timestamp>,
    /// Optional. The encryption configuration used to encrypt the backup. If this field is
    /// not specified, the backup will use the same
    /// encryption configuration as the source backup by default, namely
    /// \[encryption_type][google.spanner.admin.database.v1.CopyBackupEncryptionConfig.encryption_type\] =
    /// `USE_CONFIG_DEFAULT_OR_BACKUP_ENCRYPTION`.
    #[prost(message, optional, tag = "5")]
    pub encryption_config: ::core::option::Option<CopyBackupEncryptionConfig>,
}
/// Metadata type for the google.longrunning.Operation returned by
/// \[CopyBackup][google.spanner.admin.database.v1.DatabaseAdmin.CopyBackup\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CopyBackupMetadata {
    /// The name of the backup being created through the copy operation.
    /// Values are of the form
    /// `projects/<project>/instances/<instance>/backups/<backup>`.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// The name of the source backup that is being copied.
    /// Values are of the form
    /// `projects/<project>/instances/<instance>/backups/<backup>`.
    #[prost(string, tag = "2")]
    pub source_backup: ::prost::alloc::string::String,
    /// The progress of the
    /// \[CopyBackup][google.spanner.admin.database.v1.DatabaseAdmin.CopyBackup\] operation.
    #[prost(message, optional, tag = "3")]
    pub progress: ::core::option::Option<OperationProgress>,
    /// The time at which cancellation of CopyBackup operation was received.
    /// \[Operations.CancelOperation][google.longrunning.Operations.CancelOperation\]
    /// starts asynchronous cancellation on a long-running operation. The server
    /// makes a best effort to cancel the operation, but success is not guaranteed.
    /// Clients can use
    /// \[Operations.GetOperation][google.longrunning.Operations.GetOperation\] or
    /// other methods to check whether the cancellation succeeded or whether the
    /// operation completed despite cancellation. On successful cancellation,
    /// the operation is not deleted; instead, it becomes an operation with
    /// an \[Operation.error][google.longrunning.Operation.error\] value with a
    /// \[google.rpc.Status.code][google.rpc.Status.code\] of 1,
    /// corresponding to `Code.CANCELLED`.
    #[prost(message, optional, tag = "4")]
    pub cancel_time: ::core::option::Option<::prost_types::Timestamp>,
}
/// The request for \[UpdateBackup][google.spanner.admin.database.v1.DatabaseAdmin.UpdateBackup\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateBackupRequest {
    /// Required. The backup to update. `backup.name`, and the fields to be updated
    /// as specified by `update_mask` are required. Other fields are ignored.
    /// Update is only supported for the following fields:
    ///   * `backup.expire_time`.
    #[prost(message, optional, tag = "1")]
    pub backup: ::core::option::Option<Backup>,
    /// Required. A mask specifying which fields (e.g. `expire_time`) in the
    /// Backup resource should be updated. This mask is relative to the Backup
    /// resource, not to the request message. The field mask must always be
    /// specified; this prevents any future fields from being erased accidentally
    /// by clients that do not know about them.
    #[prost(message, optional, tag = "2")]
    pub update_mask: ::core::option::Option<::prost_types::FieldMask>,
}
/// The request for \[GetBackup][google.spanner.admin.database.v1.DatabaseAdmin.GetBackup\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetBackupRequest {
    /// Required. Name of the backup.
    /// Values are of the form
    /// `projects/<project>/instances/<instance>/backups/<backup>`.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
/// The request for \[DeleteBackup][google.spanner.admin.database.v1.DatabaseAdmin.DeleteBackup\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeleteBackupRequest {
    /// Required. Name of the backup to delete.
    /// Values are of the form
    /// `projects/<project>/instances/<instance>/backups/<backup>`.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
/// The request for \[ListBackups][google.spanner.admin.database.v1.DatabaseAdmin.ListBackups\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListBackupsRequest {
    /// Required. The instance to list backups from.  Values are of the
    /// form `projects/<project>/instances/<instance>`.
    #[prost(string, tag = "1")]
    pub parent: ::prost::alloc::string::String,
    /// An expression that filters the list of returned backups.
    ///
    /// A filter expression consists of a field name, a comparison operator, and a
    /// value for filtering.
    /// The value must be a string, a number, or a boolean. The comparison operator
    /// must be one of: `<`, `>`, `<=`, `>=`, `!=`, `=`, or `:`.
    /// Colon `:` is the contains operator. Filter rules are not case sensitive.
    ///
    /// The following fields in the \[Backup][google.spanner.admin.database.v1.Backup\] are eligible for filtering:
    ///
    ///    * `name`
    ///    * `database`
    ///    * `state`
    ///    * `create_time`  (and values are of the format YYYY-MM-DDTHH:MM:SSZ)
    ///    * `expire_time`  (and values are of the format YYYY-MM-DDTHH:MM:SSZ)
    ///    * `version_time` (and values are of the format YYYY-MM-DDTHH:MM:SSZ)
    ///    * `size_bytes`
    ///
    /// You can combine multiple expressions by enclosing each expression in
    /// parentheses. By default, expressions are combined with AND logic, but
    /// you can specify AND, OR, and NOT logic explicitly.
    ///
    /// Here are a few examples:
    ///
    ///    * `name:Howl` - The backup's name contains the string "howl".
    ///    * `database:prod`
    ///           - The database's name contains the string "prod".
    ///    * `state:CREATING` - The backup is pending creation.
    ///    * `state:READY` - The backup is fully created and ready for use.
    ///    * `(name:howl) AND (create_time < \"2018-03-28T14:50:00Z\")`
    ///           - The backup name contains the string "howl" and `create_time`
    ///               of the backup is before 2018-03-28T14:50:00Z.
    ///    * `expire_time < \"2018-03-28T14:50:00Z\"`
    ///           - The backup `expire_time` is before 2018-03-28T14:50:00Z.
    ///    * `size_bytes > 10000000000` - The backup's size is greater than 10GB
    #[prost(string, tag = "2")]
    pub filter: ::prost::alloc::string::String,
    /// Number of backups to be returned in the response. If 0 or
    /// less, defaults to the server's maximum allowed page size.
    #[prost(int32, tag = "3")]
    pub page_size: i32,
    /// If non-empty, `page_token` should contain a
    /// \[next_page_token][google.spanner.admin.database.v1.ListBackupsResponse.next_page_token\] from a
    /// previous \[ListBackupsResponse][google.spanner.admin.database.v1.ListBackupsResponse\] to the same `parent` and with the same
    /// `filter`.
    #[prost(string, tag = "4")]
    pub page_token: ::prost::alloc::string::String,
}
/// The response for \[ListBackups][google.spanner.admin.database.v1.DatabaseAdmin.ListBackups\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListBackupsResponse {
    /// The list of matching backups. Backups returned are ordered by `create_time`
    /// in descending order, starting from the most recent `create_time`.
    #[prost(message, repeated, tag = "1")]
    pub backups: ::prost::alloc::vec::Vec<Backup>,
    /// `next_page_token` can be sent in a subsequent
    /// \[ListBackups][google.spanner.admin.database.v1.DatabaseAdmin.ListBackups\] call to fetch more
    /// of the matching backups.
    #[prost(string, tag = "2")]
    pub next_page_token: ::prost::alloc::string::String,
}
/// The request for
/// \[ListBackupOperations][google.spanner.admin.database.v1.DatabaseAdmin.ListBackupOperations\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListBackupOperationsRequest {
    /// Required. The instance of the backup operations. Values are of
    /// the form `projects/<project>/instances/<instance>`.
    #[prost(string, tag = "1")]
    pub parent: ::prost::alloc::string::String,
    /// An expression that filters the list of returned backup operations.
    ///
    /// A filter expression consists of a field name, a
    /// comparison operator, and a value for filtering.
    /// The value must be a string, a number, or a boolean. The comparison operator
    /// must be one of: `<`, `>`, `<=`, `>=`, `!=`, `=`, or `:`.
    /// Colon `:` is the contains operator. Filter rules are not case sensitive.
    ///
    /// The following fields in the \[operation][google.longrunning.Operation\]
    /// are eligible for filtering:
    ///
    ///    * `name` - The name of the long-running operation
    ///    * `done` - False if the operation is in progress, else true.
    ///    * `metadata.@type` - the type of metadata. For example, the type string
    ///       for \[CreateBackupMetadata][google.spanner.admin.database.v1.CreateBackupMetadata\] is
    ///       `type.googleapis.com/google.spanner.admin.database.v1.CreateBackupMetadata`.
    ///    * `metadata.<field_name>` - any field in metadata.value.
    ///       `metadata.@type` must be specified first if filtering on metadata
    ///       fields.
    ///    * `error` - Error associated with the long-running operation.
    ///    * `response.@type` - the type of response.
    ///    * `response.<field_name>` - any field in response.value.
    ///
    /// You can combine multiple expressions by enclosing each expression in
    /// parentheses. By default, expressions are combined with AND logic, but
    /// you can specify AND, OR, and NOT logic explicitly.
    ///
    /// Here are a few examples:
    ///
    ///    * `done:true` - The operation is complete.
    ///    * `(metadata.@type=type.googleapis.com/google.spanner.admin.database.v1.CreateBackupMetadata) AND` \
    ///       `metadata.database:prod` - Returns operations where:
    ///       * The operation's metadata type is \[CreateBackupMetadata][google.spanner.admin.database.v1.CreateBackupMetadata\].
    ///       * The database the backup was taken from has a name containing the
    ///       string "prod".
    ///    * `(metadata.@type=type.googleapis.com/google.spanner.admin.database.v1.CreateBackupMetadata) AND` \
    ///      `(metadata.name:howl) AND` \
    ///      `(metadata.progress.start_time < \"2018-03-28T14:50:00Z\") AND` \
    ///      `(error:*)` - Returns operations where:
    ///      * The operation's metadata type is \[CreateBackupMetadata][google.spanner.admin.database.v1.CreateBackupMetadata\].
    ///      * The backup name contains the string "howl".
    ///      * The operation started before 2018-03-28T14:50:00Z.
    ///      * The operation resulted in an error.
    ///    * `(metadata.@type=type.googleapis.com/google.spanner.admin.database.v1.CopyBackupMetadata) AND` \
    ///      `(metadata.source_backup:test) AND` \
    ///      `(metadata.progress.start_time < \"2022-01-18T14:50:00Z\") AND` \
    ///      `(error:*)` - Returns operations where:
    ///      * The operation's metadata type is \[CopyBackupMetadata][google.spanner.admin.database.v1.CopyBackupMetadata\].
    ///      * The source backup of the copied backup name contains the string
    ///      "test".
    ///      * The operation started before 2022-01-18T14:50:00Z.
    ///      * The operation resulted in an error.
    ///    * `((metadata.@type=type.googleapis.com/google.spanner.admin.database.v1.CreateBackupMetadata) AND` \
    ///      `(metadata.database:test_db)) OR` \
    ///      `((metadata.@type=type.googleapis.com/google.spanner.admin.database.v1.CopyBackupMetadata)
    ///      AND` \
    ///      `(metadata.source_backup:test_bkp)) AND` \
    ///      `(error:*)` - Returns operations where:
    ///      * The operation's metadata matches either of criteria:
    ///        * The operation's metadata type is \[CreateBackupMetadata][google.spanner.admin.database.v1.CreateBackupMetadata\] AND the
    ///        database the backup was taken from has name containing string
    ///        "test_db"
    ///        * The operation's metadata type is \[CopyBackupMetadata][google.spanner.admin.database.v1.CopyBackupMetadata\] AND the
    ///        backup the backup was copied from has name containing string
    ///        "test_bkp"
    ///      * The operation resulted in an error.
    #[prost(string, tag = "2")]
    pub filter: ::prost::alloc::string::String,
    /// Number of operations to be returned in the response. If 0 or
    /// less, defaults to the server's maximum allowed page size.
    #[prost(int32, tag = "3")]
    pub page_size: i32,
    /// If non-empty, `page_token` should contain a
    /// \[next_page_token][google.spanner.admin.database.v1.ListBackupOperationsResponse.next_page_token\]
    /// from a previous \[ListBackupOperationsResponse][google.spanner.admin.database.v1.ListBackupOperationsResponse\] to the
    /// same `parent` and with the same `filter`.
    #[prost(string, tag = "4")]
    pub page_token: ::prost::alloc::string::String,
}
/// The response for
/// \[ListBackupOperations][google.spanner.admin.database.v1.DatabaseAdmin.ListBackupOperations\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListBackupOperationsResponse {
    /// The list of matching backup [long-running
    /// operations]\[google.longrunning.Operation\]. Each operation's name will be
    /// prefixed by the backup's name. The operation's
    /// \[metadata][google.longrunning.Operation.metadata\] field type
    /// `metadata.type_url` describes the type of the metadata. Operations returned
    /// include those that are pending or have completed/failed/canceled within the
    /// last 7 days. Operations returned are ordered by
    /// `operation.metadata.value.progress.start_time` in descending order starting
    /// from the most recently started operation.
    #[prost(message, repeated, tag = "1")]
    pub operations: ::prost::alloc::vec::Vec<
        super::super::super::super::longrunning::Operation,
    >,
    /// `next_page_token` can be sent in a subsequent
    /// \[ListBackupOperations][google.spanner.admin.database.v1.DatabaseAdmin.ListBackupOperations\]
    /// call to fetch more of the matching metadata.
    #[prost(string, tag = "2")]
    pub next_page_token: ::prost::alloc::string::String,
}
/// Information about a backup.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BackupInfo {
    /// Name of the backup.
    #[prost(string, tag = "1")]
    pub backup: ::prost::alloc::string::String,
    /// The backup contains an externally consistent copy of `source_database` at
    /// the timestamp specified by `version_time`. If the
    /// \[CreateBackup][google.spanner.admin.database.v1.DatabaseAdmin.CreateBackup\] request did not specify
    /// `version_time`, the `version_time` of the backup is equivalent to the
    /// `create_time`.
    #[prost(message, optional, tag = "4")]
    pub version_time: ::core::option::Option<::prost_types::Timestamp>,
    /// The time the \[CreateBackup][google.spanner.admin.database.v1.DatabaseAdmin.CreateBackup\] request was
    /// received.
    #[prost(message, optional, tag = "2")]
    pub create_time: ::core::option::Option<::prost_types::Timestamp>,
    /// Name of the database the backup was created from.
    #[prost(string, tag = "3")]
    pub source_database: ::prost::alloc::string::String,
}
/// Encryption configuration for the backup to create.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateBackupEncryptionConfig {
    /// Required. The encryption type of the backup.
    #[prost(enumeration = "create_backup_encryption_config::EncryptionType", tag = "1")]
    pub encryption_type: i32,
    /// Optional. The Cloud KMS key that will be used to protect the backup.
    /// This field should be set only when
    /// \[encryption_type][google.spanner.admin.database.v1.CreateBackupEncryptionConfig.encryption_type\] is
    /// `CUSTOMER_MANAGED_ENCRYPTION`. Values are of the form
    /// `projects/<project>/locations/<location>/keyRings/<key_ring>/cryptoKeys/<kms_key_name>`.
    #[prost(string, tag = "2")]
    pub kms_key_name: ::prost::alloc::string::String,
}
/// Nested message and enum types in `CreateBackupEncryptionConfig`.
pub mod create_backup_encryption_config {
    /// Encryption types for the backup.
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum EncryptionType {
        /// Unspecified. Do not use.
        Unspecified = 0,
        /// Use the same encryption configuration as the database. This is the
        /// default option when
        /// \[encryption_config][google.spanner.admin.database.v1.CreateBackupEncryptionConfig\] is empty.
        /// For example, if the database is using `Customer_Managed_Encryption`, the
        /// backup will be using the same Cloud KMS key as the database.
        UseDatabaseEncryption = 1,
        /// Use Google default encryption.
        GoogleDefaultEncryption = 2,
        /// Use customer managed encryption. If specified, `kms_key_name`
        /// must contain a valid Cloud KMS key.
        CustomerManagedEncryption = 3,
    }
    impl EncryptionType {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                EncryptionType::Unspecified => "ENCRYPTION_TYPE_UNSPECIFIED",
                EncryptionType::UseDatabaseEncryption => "USE_DATABASE_ENCRYPTION",
                EncryptionType::GoogleDefaultEncryption => "GOOGLE_DEFAULT_ENCRYPTION",
                EncryptionType::CustomerManagedEncryption => {
                    "CUSTOMER_MANAGED_ENCRYPTION"
                }
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "ENCRYPTION_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
                "USE_DATABASE_ENCRYPTION" => Some(Self::UseDatabaseEncryption),
                "GOOGLE_DEFAULT_ENCRYPTION" => Some(Self::GoogleDefaultEncryption),
                "CUSTOMER_MANAGED_ENCRYPTION" => Some(Self::CustomerManagedEncryption),
                _ => None,
            }
        }
    }
}
/// Encryption configuration for the copied backup.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CopyBackupEncryptionConfig {
    /// Required. The encryption type of the backup.
    #[prost(enumeration = "copy_backup_encryption_config::EncryptionType", tag = "1")]
    pub encryption_type: i32,
    /// Optional. The Cloud KMS key that will be used to protect the backup.
    /// This field should be set only when
    /// \[encryption_type][google.spanner.admin.database.v1.CopyBackupEncryptionConfig.encryption_type\] is
    /// `CUSTOMER_MANAGED_ENCRYPTION`. Values are of the form
    /// `projects/<project>/locations/<location>/keyRings/<key_ring>/cryptoKeys/<kms_key_name>`.
    #[prost(string, tag = "2")]
    pub kms_key_name: ::prost::alloc::string::String,
}
/// Nested message and enum types in `CopyBackupEncryptionConfig`.
pub mod copy_backup_encryption_config {
    /// Encryption types for the backup.
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum EncryptionType {
        /// Unspecified. Do not use.
        Unspecified = 0,
        /// This is the default option for \[CopyBackup][google.spanner.admin.database.v1.DatabaseAdmin.CopyBackup\]
        /// when \[encryption_config][google.spanner.admin.database.v1.CopyBackupEncryptionConfig\] is not specified.
        /// For example, if the source backup is using `Customer_Managed_Encryption`,
        /// the backup will be using the same Cloud KMS key as the source backup.
        UseConfigDefaultOrBackupEncryption = 1,
        /// Use Google default encryption.
        GoogleDefaultEncryption = 2,
        /// Use customer managed encryption. If specified, `kms_key_name`
        /// must contain a valid Cloud KMS key.
        CustomerManagedEncryption = 3,
    }
    impl EncryptionType {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                EncryptionType::Unspecified => "ENCRYPTION_TYPE_UNSPECIFIED",
                EncryptionType::UseConfigDefaultOrBackupEncryption => {
                    "USE_CONFIG_DEFAULT_OR_BACKUP_ENCRYPTION"
                }
                EncryptionType::GoogleDefaultEncryption => "GOOGLE_DEFAULT_ENCRYPTION",
                EncryptionType::CustomerManagedEncryption => {
                    "CUSTOMER_MANAGED_ENCRYPTION"
                }
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "ENCRYPTION_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
                "USE_CONFIG_DEFAULT_OR_BACKUP_ENCRYPTION" => {
                    Some(Self::UseConfigDefaultOrBackupEncryption)
                }
                "GOOGLE_DEFAULT_ENCRYPTION" => Some(Self::GoogleDefaultEncryption),
                "CUSTOMER_MANAGED_ENCRYPTION" => Some(Self::CustomerManagedEncryption),
                _ => None,
            }
        }
    }
}
/// Information about the database restore.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RestoreInfo {
    /// The type of the restore source.
    #[prost(enumeration = "RestoreSourceType", tag = "1")]
    pub source_type: i32,
    /// Information about the source used to restore the database.
    #[prost(oneof = "restore_info::SourceInfo", tags = "2")]
    pub source_info: ::core::option::Option<restore_info::SourceInfo>,
}
/// Nested message and enum types in `RestoreInfo`.
pub mod restore_info {
    /// Information about the source used to restore the database.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum SourceInfo {
        /// Information about the backup used to restore the database. The backup
        /// may no longer exist.
        #[prost(message, tag = "2")]
        BackupInfo(super::BackupInfo),
    }
}
/// A Cloud Spanner database.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Database {
    /// Required. The name of the database. Values are of the form
    /// `projects/<project>/instances/<instance>/databases/<database>`,
    /// where `<database>` is as specified in the `CREATE DATABASE`
    /// statement. This name can be passed to other API methods to
    /// identify the database.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// Output only. The current database state.
    #[prost(enumeration = "database::State", tag = "2")]
    pub state: i32,
    /// Output only. If exists, the time at which the database creation started.
    #[prost(message, optional, tag = "3")]
    pub create_time: ::core::option::Option<::prost_types::Timestamp>,
    /// Output only. Applicable only for restored databases. Contains information
    /// about the restore source.
    #[prost(message, optional, tag = "4")]
    pub restore_info: ::core::option::Option<RestoreInfo>,
    /// Output only. For databases that are using customer managed encryption, this
    /// field contains the encryption configuration for the database.
    /// For databases that are using Google default or other types of encryption,
    /// this field is empty.
    #[prost(message, optional, tag = "5")]
    pub encryption_config: ::core::option::Option<EncryptionConfig>,
    /// Output only. For databases that are using customer managed encryption, this
    /// field contains the encryption information for the database, such as
    /// encryption state and the Cloud KMS key versions that are in use.
    ///
    /// For databases that are using Google default or other types of encryption,
    /// this field is empty.
    ///
    /// This field is propagated lazily from the backend. There might be a delay
    /// from when a key version is being used and when it appears in this field.
    #[prost(message, repeated, tag = "8")]
    pub encryption_info: ::prost::alloc::vec::Vec<EncryptionInfo>,
    /// Output only. The period in which Cloud Spanner retains all versions of data
    /// for the database. This is the same as the value of version_retention_period
    /// database option set using
    /// \[UpdateDatabaseDdl][google.spanner.admin.database.v1.DatabaseAdmin.UpdateDatabaseDdl\]. Defaults to 1 hour,
    /// if not set.
    #[prost(string, tag = "6")]
    pub version_retention_period: ::prost::alloc::string::String,
    /// Output only. Earliest timestamp at which older versions of the data can be
    /// read. This value is continuously updated by Cloud Spanner and becomes stale
    /// the moment it is queried. If you are using this value to recover data, make
    /// sure to account for the time from the moment when the value is queried to
    /// the moment when you initiate the recovery.
    #[prost(message, optional, tag = "7")]
    pub earliest_version_time: ::core::option::Option<::prost_types::Timestamp>,
    /// Output only. The read-write region which contains the database's leader
    /// replicas.
    ///
    /// This is the same as the value of default_leader
    /// database option set using DatabaseAdmin.CreateDatabase or
    /// DatabaseAdmin.UpdateDatabaseDdl. If not explicitly set, this is empty.
    #[prost(string, tag = "9")]
    pub default_leader: ::prost::alloc::string::String,
    /// Output only. The dialect of the Cloud Spanner Database.
    #[prost(enumeration = "DatabaseDialect", tag = "10")]
    pub database_dialect: i32,
}
/// Nested message and enum types in `Database`.
pub mod database {
    /// Indicates the current state of the database.
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum State {
        /// Not specified.
        Unspecified = 0,
        /// The database is still being created. Operations on the database may fail
        /// with `FAILED_PRECONDITION` in this state.
        Creating = 1,
        /// The database is fully created and ready for use.
        Ready = 2,
        /// The database is fully created and ready for use, but is still
        /// being optimized for performance and cannot handle full load.
        ///
        /// In this state, the database still references the backup
        /// it was restore from, preventing the backup
        /// from being deleted. When optimizations are complete, the full performance
        /// of the database will be restored, and the database will transition to
        /// `READY` state.
        ReadyOptimizing = 3,
    }
    impl State {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                State::Unspecified => "STATE_UNSPECIFIED",
                State::Creating => "CREATING",
                State::Ready => "READY",
                State::ReadyOptimizing => "READY_OPTIMIZING",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "STATE_UNSPECIFIED" => Some(Self::Unspecified),
                "CREATING" => Some(Self::Creating),
                "READY" => Some(Self::Ready),
                "READY_OPTIMIZING" => Some(Self::ReadyOptimizing),
                _ => None,
            }
        }
    }
}
/// The request for \[ListDatabases][google.spanner.admin.database.v1.DatabaseAdmin.ListDatabases\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListDatabasesRequest {
    /// Required. The instance whose databases should be listed.
    /// Values are of the form `projects/<project>/instances/<instance>`.
    #[prost(string, tag = "1")]
    pub parent: ::prost::alloc::string::String,
    /// Number of databases to be returned in the response. If 0 or less,
    /// defaults to the server's maximum allowed page size.
    #[prost(int32, tag = "3")]
    pub page_size: i32,
    /// If non-empty, `page_token` should contain a
    /// \[next_page_token][google.spanner.admin.database.v1.ListDatabasesResponse.next_page_token\] from a
    /// previous \[ListDatabasesResponse][google.spanner.admin.database.v1.ListDatabasesResponse\].
    #[prost(string, tag = "4")]
    pub page_token: ::prost::alloc::string::String,
}
/// The response for \[ListDatabases][google.spanner.admin.database.v1.DatabaseAdmin.ListDatabases\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListDatabasesResponse {
    /// Databases that matched the request.
    #[prost(message, repeated, tag = "1")]
    pub databases: ::prost::alloc::vec::Vec<Database>,
    /// `next_page_token` can be sent in a subsequent
    /// \[ListDatabases][google.spanner.admin.database.v1.DatabaseAdmin.ListDatabases\] call to fetch more
    /// of the matching databases.
    #[prost(string, tag = "2")]
    pub next_page_token: ::prost::alloc::string::String,
}
/// The request for \[CreateDatabase][google.spanner.admin.database.v1.DatabaseAdmin.CreateDatabase\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateDatabaseRequest {
    /// Required. The name of the instance that will serve the new database.
    /// Values are of the form `projects/<project>/instances/<instance>`.
    #[prost(string, tag = "1")]
    pub parent: ::prost::alloc::string::String,
    /// Required. A `CREATE DATABASE` statement, which specifies the ID of the
    /// new database.  The database ID must conform to the regular expression
    /// `\[a-z][a-z0-9_\-]*[a-z0-9\]` and be between 2 and 30 characters in length.
    /// If the database ID is a reserved word or if it contains a hyphen, the
    /// database ID must be enclosed in backticks (`` ` ``).
    #[prost(string, tag = "2")]
    pub create_statement: ::prost::alloc::string::String,
    /// Optional. A list of DDL statements to run inside the newly created
    /// database. Statements can create tables, indexes, etc. These
    /// statements execute atomically with the creation of the database:
    /// if there is an error in any statement, the database is not created.
    #[prost(string, repeated, tag = "3")]
    pub extra_statements: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Optional. The encryption configuration for the database. If this field is not
    /// specified, Cloud Spanner will encrypt/decrypt all data at rest using
    /// Google default encryption.
    #[prost(message, optional, tag = "4")]
    pub encryption_config: ::core::option::Option<EncryptionConfig>,
    /// Optional. The dialect of the Cloud Spanner Database.
    #[prost(enumeration = "DatabaseDialect", tag = "5")]
    pub database_dialect: i32,
}
/// Metadata type for the operation returned by
/// \[CreateDatabase][google.spanner.admin.database.v1.DatabaseAdmin.CreateDatabase\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateDatabaseMetadata {
    /// The database being created.
    #[prost(string, tag = "1")]
    pub database: ::prost::alloc::string::String,
}
/// The request for \[GetDatabase][google.spanner.admin.database.v1.DatabaseAdmin.GetDatabase\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetDatabaseRequest {
    /// Required. The name of the requested database. Values are of the form
    /// `projects/<project>/instances/<instance>/databases/<database>`.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
/// Enqueues the given DDL statements to be applied, in order but not
/// necessarily all at once, to the database schema at some point (or
/// points) in the future. The server checks that the statements
/// are executable (syntactically valid, name tables that exist, etc.)
/// before enqueueing them, but they may still fail upon
/// later execution (e.g., if a statement from another batch of
/// statements is applied first and it conflicts in some way, or if
/// there is some data-related problem like a `NULL` value in a column to
/// which `NOT NULL` would be added). If a statement fails, all
/// subsequent statements in the batch are automatically cancelled.
///
/// Each batch of statements is assigned a name which can be used with
/// the \[Operations][google.longrunning.Operations\] API to monitor
/// progress. See the
/// \[operation_id][google.spanner.admin.database.v1.UpdateDatabaseDdlRequest.operation_id\] field for more
/// details.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateDatabaseDdlRequest {
    /// Required. The database to update.
    #[prost(string, tag = "1")]
    pub database: ::prost::alloc::string::String,
    /// Required. DDL statements to be applied to the database.
    #[prost(string, repeated, tag = "2")]
    pub statements: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// If empty, the new update request is assigned an
    /// automatically-generated operation ID. Otherwise, `operation_id`
    /// is used to construct the name of the resulting
    /// \[Operation][google.longrunning.Operation\].
    ///
    /// Specifying an explicit operation ID simplifies determining
    /// whether the statements were executed in the event that the
    /// \[UpdateDatabaseDdl][google.spanner.admin.database.v1.DatabaseAdmin.UpdateDatabaseDdl\] call is replayed,
    /// or the return value is otherwise lost: the \[database][google.spanner.admin.database.v1.UpdateDatabaseDdlRequest.database\] and
    /// `operation_id` fields can be combined to form the
    /// \[name][google.longrunning.Operation.name\] of the resulting
    /// \[longrunning.Operation][google.longrunning.Operation\]: `<database>/operations/<operation_id>`.
    ///
    /// `operation_id` should be unique within the database, and must be
    /// a valid identifier: `\[a-z][a-z0-9_\]*`. Note that
    /// automatically-generated operation IDs always begin with an
    /// underscore. If the named operation already exists,
    /// \[UpdateDatabaseDdl][google.spanner.admin.database.v1.DatabaseAdmin.UpdateDatabaseDdl\] returns
    /// `ALREADY_EXISTS`.
    #[prost(string, tag = "3")]
    pub operation_id: ::prost::alloc::string::String,
}
/// Metadata type for the operation returned by
/// \[UpdateDatabaseDdl][google.spanner.admin.database.v1.DatabaseAdmin.UpdateDatabaseDdl\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateDatabaseDdlMetadata {
    /// The database being modified.
    #[prost(string, tag = "1")]
    pub database: ::prost::alloc::string::String,
    /// For an update this list contains all the statements. For an
    /// individual statement, this list contains only that statement.
    #[prost(string, repeated, tag = "2")]
    pub statements: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    /// Reports the commit timestamps of all statements that have
    /// succeeded so far, where `commit_timestamps\[i\]` is the commit
    /// timestamp for the statement `statements\[i\]`.
    #[prost(message, repeated, tag = "3")]
    pub commit_timestamps: ::prost::alloc::vec::Vec<::prost_types::Timestamp>,
    /// Output only. When true, indicates that the operation is throttled e.g
    /// due to resource constraints. When resources become available the operation
    /// will resume and this field will be false again.
    #[prost(bool, tag = "4")]
    pub throttled: bool,
    /// The progress of the
    /// \[UpdateDatabaseDdl][google.spanner.admin.database.v1.DatabaseAdmin.UpdateDatabaseDdl\] operations.
    /// Currently, only index creation statements will have a continuously
    /// updating progress.
    /// For non-index creation statements, `progress\[i\]` will have start time
    /// and end time populated with commit timestamp of operation,
    /// as well as a progress of 100% once the operation has completed.
    /// `progress\[i\]` is the operation progress for `statements\[i\]`.
    #[prost(message, repeated, tag = "5")]
    pub progress: ::prost::alloc::vec::Vec<OperationProgress>,
}
/// The request for \[DropDatabase][google.spanner.admin.database.v1.DatabaseAdmin.DropDatabase\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DropDatabaseRequest {
    /// Required. The database to be dropped.
    #[prost(string, tag = "1")]
    pub database: ::prost::alloc::string::String,
}
/// The request for \[GetDatabaseDdl][google.spanner.admin.database.v1.DatabaseAdmin.GetDatabaseDdl\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetDatabaseDdlRequest {
    /// Required. The database whose schema we wish to get.
    /// Values are of the form
    /// `projects/<project>/instances/<instance>/databases/<database>`
    #[prost(string, tag = "1")]
    pub database: ::prost::alloc::string::String,
}
/// The response for \[GetDatabaseDdl][google.spanner.admin.database.v1.DatabaseAdmin.GetDatabaseDdl\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetDatabaseDdlResponse {
    /// A list of formatted DDL statements defining the schema of the database
    /// specified in the request.
    #[prost(string, repeated, tag = "1")]
    pub statements: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// The request for
/// \[ListDatabaseOperations][google.spanner.admin.database.v1.DatabaseAdmin.ListDatabaseOperations\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListDatabaseOperationsRequest {
    /// Required. The instance of the database operations.
    /// Values are of the form `projects/<project>/instances/<instance>`.
    #[prost(string, tag = "1")]
    pub parent: ::prost::alloc::string::String,
    /// An expression that filters the list of returned operations.
    ///
    /// A filter expression consists of a field name, a
    /// comparison operator, and a value for filtering.
    /// The value must be a string, a number, or a boolean. The comparison operator
    /// must be one of: `<`, `>`, `<=`, `>=`, `!=`, `=`, or `:`.
    /// Colon `:` is the contains operator. Filter rules are not case sensitive.
    ///
    /// The following fields in the \[Operation][google.longrunning.Operation\]
    /// are eligible for filtering:
    ///
    ///    * `name` - The name of the long-running operation
    ///    * `done` - False if the operation is in progress, else true.
    ///    * `metadata.@type` - the type of metadata. For example, the type string
    ///       for \[RestoreDatabaseMetadata][google.spanner.admin.database.v1.RestoreDatabaseMetadata\] is
    ///       `type.googleapis.com/google.spanner.admin.database.v1.RestoreDatabaseMetadata`.
    ///    * `metadata.<field_name>` - any field in metadata.value.
    ///       `metadata.@type` must be specified first, if filtering on metadata
    ///       fields.
    ///    * `error` - Error associated with the long-running operation.
    ///    * `response.@type` - the type of response.
    ///    * `response.<field_name>` - any field in response.value.
    ///
    /// You can combine multiple expressions by enclosing each expression in
    /// parentheses. By default, expressions are combined with AND logic. However,
    /// you can specify AND, OR, and NOT logic explicitly.
    ///
    /// Here are a few examples:
    ///
    ///    * `done:true` - The operation is complete.
    ///    * `(metadata.@type=type.googleapis.com/google.spanner.admin.database.v1.RestoreDatabaseMetadata) AND` \
    ///      `(metadata.source_type:BACKUP) AND` \
    ///      `(metadata.backup_info.backup:backup_howl) AND` \
    ///      `(metadata.name:restored_howl) AND` \
    ///      `(metadata.progress.start_time < \"2018-03-28T14:50:00Z\") AND` \
    ///      `(error:*)` - Return operations where:
    ///      * The operation's metadata type is \[RestoreDatabaseMetadata][google.spanner.admin.database.v1.RestoreDatabaseMetadata\].
    ///      * The database is restored from a backup.
    ///      * The backup name contains "backup_howl".
    ///      * The restored database's name contains "restored_howl".
    ///      * The operation started before 2018-03-28T14:50:00Z.
    ///      * The operation resulted in an error.
    #[prost(string, tag = "2")]
    pub filter: ::prost::alloc::string::String,
    /// Number of operations to be returned in the response. If 0 or
    /// less, defaults to the server's maximum allowed page size.
    #[prost(int32, tag = "3")]
    pub page_size: i32,
    /// If non-empty, `page_token` should contain a
    /// \[next_page_token][google.spanner.admin.database.v1.ListDatabaseOperationsResponse.next_page_token\]
    /// from a previous \[ListDatabaseOperationsResponse][google.spanner.admin.database.v1.ListDatabaseOperationsResponse\] to the
    /// same `parent` and with the same `filter`.
    #[prost(string, tag = "4")]
    pub page_token: ::prost::alloc::string::String,
}
/// The response for
/// \[ListDatabaseOperations][google.spanner.admin.database.v1.DatabaseAdmin.ListDatabaseOperations\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListDatabaseOperationsResponse {
    /// The list of matching database [long-running
    /// operations]\[google.longrunning.Operation\]. Each operation's name will be
    /// prefixed by the database's name. The operation's
    /// \[metadata][google.longrunning.Operation.metadata\] field type
    /// `metadata.type_url` describes the type of the metadata.
    #[prost(message, repeated, tag = "1")]
    pub operations: ::prost::alloc::vec::Vec<
        super::super::super::super::longrunning::Operation,
    >,
    /// `next_page_token` can be sent in a subsequent
    /// \[ListDatabaseOperations][google.spanner.admin.database.v1.DatabaseAdmin.ListDatabaseOperations\]
    /// call to fetch more of the matching metadata.
    #[prost(string, tag = "2")]
    pub next_page_token: ::prost::alloc::string::String,
}
/// The request for
/// \[RestoreDatabase][google.spanner.admin.database.v1.DatabaseAdmin.RestoreDatabase\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RestoreDatabaseRequest {
    /// Required. The name of the instance in which to create the
    /// restored database. This instance must be in the same project and
    /// have the same instance configuration as the instance containing
    /// the source backup. Values are of the form
    /// `projects/<project>/instances/<instance>`.
    #[prost(string, tag = "1")]
    pub parent: ::prost::alloc::string::String,
    /// Required. The id of the database to create and restore to. This
    /// database must not already exist. The `database_id` appended to
    /// `parent` forms the full database name of the form
    /// `projects/<project>/instances/<instance>/databases/<database_id>`.
    #[prost(string, tag = "2")]
    pub database_id: ::prost::alloc::string::String,
    /// Optional. An encryption configuration describing the encryption type and key
    /// resources in Cloud KMS used to encrypt/decrypt the database to restore to.
    /// If this field is not specified, the restored database will use
    /// the same encryption configuration as the backup by default, namely
    /// \[encryption_type][google.spanner.admin.database.v1.RestoreDatabaseEncryptionConfig.encryption_type\] =
    /// `USE_CONFIG_DEFAULT_OR_BACKUP_ENCRYPTION`.
    #[prost(message, optional, tag = "4")]
    pub encryption_config: ::core::option::Option<RestoreDatabaseEncryptionConfig>,
    /// Required. The source from which to restore.
    #[prost(oneof = "restore_database_request::Source", tags = "3")]
    pub source: ::core::option::Option<restore_database_request::Source>,
}
/// Nested message and enum types in `RestoreDatabaseRequest`.
pub mod restore_database_request {
    /// Required. The source from which to restore.
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Source {
        /// Name of the backup from which to restore.  Values are of the form
        /// `projects/<project>/instances/<instance>/backups/<backup>`.
        #[prost(string, tag = "3")]
        Backup(::prost::alloc::string::String),
    }
}
/// Encryption configuration for the restored database.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RestoreDatabaseEncryptionConfig {
    /// Required. The encryption type of the restored database.
    #[prost(
        enumeration = "restore_database_encryption_config::EncryptionType",
        tag = "1"
    )]
    pub encryption_type: i32,
    /// Optional. The Cloud KMS key that will be used to encrypt/decrypt the restored
    /// database. This field should be set only when
    /// \[encryption_type][google.spanner.admin.database.v1.RestoreDatabaseEncryptionConfig.encryption_type\] is
    /// `CUSTOMER_MANAGED_ENCRYPTION`. Values are of the form
    /// `projects/<project>/locations/<location>/keyRings/<key_ring>/cryptoKeys/<kms_key_name>`.
    #[prost(string, tag = "2")]
    pub kms_key_name: ::prost::alloc::string::String,
}
/// Nested message and enum types in `RestoreDatabaseEncryptionConfig`.
pub mod restore_database_encryption_config {
    /// Encryption types for the database to be restored.
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum EncryptionType {
        /// Unspecified. Do not use.
        Unspecified = 0,
        /// This is the default option when
        /// \[encryption_config][google.spanner.admin.database.v1.RestoreDatabaseEncryptionConfig\] is not specified.
        UseConfigDefaultOrBackupEncryption = 1,
        /// Use Google default encryption.
        GoogleDefaultEncryption = 2,
        /// Use customer managed encryption. If specified, `kms_key_name` must
        /// must contain a valid Cloud KMS key.
        CustomerManagedEncryption = 3,
    }
    impl EncryptionType {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                EncryptionType::Unspecified => "ENCRYPTION_TYPE_UNSPECIFIED",
                EncryptionType::UseConfigDefaultOrBackupEncryption => {
                    "USE_CONFIG_DEFAULT_OR_BACKUP_ENCRYPTION"
                }
                EncryptionType::GoogleDefaultEncryption => "GOOGLE_DEFAULT_ENCRYPTION",
                EncryptionType::CustomerManagedEncryption => {
                    "CUSTOMER_MANAGED_ENCRYPTION"
                }
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "ENCRYPTION_TYPE_UNSPECIFIED" => Some(Self::Unspecified),
                "USE_CONFIG_DEFAULT_OR_BACKUP_ENCRYPTION" => {
                    Some(Self::UseConfigDefaultOrBackupEncryption)
                }
                "GOOGLE_DEFAULT_ENCRYPTION" => Some(Self::GoogleDefaultEncryption),
                "CUSTOMER_MANAGED_ENCRYPTION" => Some(Self::CustomerManagedEncryption),
                _ => None,
            }
        }
    }
}
/// Metadata type for the long-running operation returned by
/// \[RestoreDatabase][google.spanner.admin.database.v1.DatabaseAdmin.RestoreDatabase\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RestoreDatabaseMetadata {
    /// Name of the database being created and restored to.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// The type of the restore source.
    #[prost(enumeration = "RestoreSourceType", tag = "2")]
    pub source_type: i32,
    /// The progress of the
    /// \[RestoreDatabase][google.spanner.admin.database.v1.DatabaseAdmin.RestoreDatabase\]
    /// operation.
    #[prost(message, optional, tag = "4")]
    pub progress: ::core::option::Option<OperationProgress>,
    /// The time at which cancellation of this operation was received.
    /// \[Operations.CancelOperation][google.longrunning.Operations.CancelOperation\]
    /// starts asynchronous cancellation on a long-running operation. The server
    /// makes a best effort to cancel the operation, but success is not guaranteed.
    /// Clients can use
    /// \[Operations.GetOperation][google.longrunning.Operations.GetOperation\] or
    /// other methods to check whether the cancellation succeeded or whether the
    /// operation completed despite cancellation. On successful cancellation,
    /// the operation is not deleted; instead, it becomes an operation with
    /// an \[Operation.error][google.longrunning.Operation.error\] value with a
    /// \[google.rpc.Status.code][google.rpc.Status.code\] of 1, corresponding to `Code.CANCELLED`.
    #[prost(message, optional, tag = "5")]
    pub cancel_time: ::core::option::Option<::prost_types::Timestamp>,
    /// If exists, the name of the long-running operation that will be used to
    /// track the post-restore optimization process to optimize the performance of
    /// the restored database, and remove the dependency on the restore source.
    /// The name is of the form
    /// `projects/<project>/instances/<instance>/databases/<database>/operations/<operation>`
    /// where the <database> is the name of database being created and restored to.
    /// The metadata type of the  long-running operation is
    /// \[OptimizeRestoredDatabaseMetadata][google.spanner.admin.database.v1.OptimizeRestoredDatabaseMetadata\]. This long-running operation will be
    /// automatically created by the system after the RestoreDatabase long-running
    /// operation completes successfully. This operation will not be created if the
    /// restore was not successful.
    #[prost(string, tag = "6")]
    pub optimize_database_operation_name: ::prost::alloc::string::String,
    /// Information about the source used to restore the database, as specified by
    /// `source` in \[RestoreDatabaseRequest][google.spanner.admin.database.v1.RestoreDatabaseRequest\].
    #[prost(oneof = "restore_database_metadata::SourceInfo", tags = "3")]
    pub source_info: ::core::option::Option<restore_database_metadata::SourceInfo>,
}
/// Nested message and enum types in `RestoreDatabaseMetadata`.
pub mod restore_database_metadata {
    /// Information about the source used to restore the database, as specified by
    /// `source` in \[RestoreDatabaseRequest][google.spanner.admin.database.v1.RestoreDatabaseRequest\].
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum SourceInfo {
        /// Information about the backup used to restore the database.
        #[prost(message, tag = "3")]
        BackupInfo(super::BackupInfo),
    }
}
/// Metadata type for the long-running operation used to track the progress
/// of optimizations performed on a newly restored database. This long-running
/// operation is automatically created by the system after the successful
/// completion of a database restore, and cannot be cancelled.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OptimizeRestoredDatabaseMetadata {
    /// Name of the restored database being optimized.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    /// The progress of the post-restore optimizations.
    #[prost(message, optional, tag = "2")]
    pub progress: ::core::option::Option<OperationProgress>,
}
/// A Cloud Spanner database role.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DatabaseRole {
    /// Required. The name of the database role. Values are of the form
    /// `projects/<project>/instances/<instance>/databases/<database>/databaseRoles/
    /// {role}`, where `<role>` is as specified in the `CREATE ROLE`
    /// DDL statement. This name can be passed to Get/Set IAMPolicy methods to
    /// identify the database role.
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
/// The request for \[ListDatabaseRoles][google.spanner.admin.database.v1.DatabaseAdmin.ListDatabaseRoles\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListDatabaseRolesRequest {
    /// Required. The database whose roles should be listed.
    /// Values are of the form
    /// `projects/<project>/instances/<instance>/databases/<database>/databaseRoles`.
    #[prost(string, tag = "1")]
    pub parent: ::prost::alloc::string::String,
    /// Number of database roles to be returned in the response. If 0 or less,
    /// defaults to the server's maximum allowed page size.
    #[prost(int32, tag = "2")]
    pub page_size: i32,
    /// If non-empty, `page_token` should contain a
    /// \[next_page_token][google.spanner.admin.database.v1.ListDatabaseRolesResponse.next_page_token\] from a
    /// previous \[ListDatabaseRolesResponse][google.spanner.admin.database.v1.ListDatabaseRolesResponse\].
    #[prost(string, tag = "3")]
    pub page_token: ::prost::alloc::string::String,
}
/// The response for \[ListDatabaseRoles][google.spanner.admin.database.v1.DatabaseAdmin.ListDatabaseRoles\].
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ListDatabaseRolesResponse {
    /// Database roles that matched the request.
    #[prost(message, repeated, tag = "1")]
    pub database_roles: ::prost::alloc::vec::Vec<DatabaseRole>,
    /// `next_page_token` can be sent in a subsequent
    /// \[ListDatabaseRoles][google.spanner.admin.database.v1.DatabaseAdmin.ListDatabaseRoles\]
    /// call to fetch more of the matching roles.
    #[prost(string, tag = "2")]
    pub next_page_token: ::prost::alloc::string::String,
}
/// Indicates the type of the restore source.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum RestoreSourceType {
    /// No restore associated.
    TypeUnspecified = 0,
    /// A backup was used as the source of the restore.
    Backup = 1,
}
impl RestoreSourceType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            RestoreSourceType::TypeUnspecified => "TYPE_UNSPECIFIED",
            RestoreSourceType::Backup => "BACKUP",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "TYPE_UNSPECIFIED" => Some(Self::TypeUnspecified),
            "BACKUP" => Some(Self::Backup),
            _ => None,
        }
    }
}
/// Generated client implementations.
pub mod database_admin_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Cloud Spanner Database Admin API
    ///
    /// The Cloud Spanner Database Admin API can be used to:
    ///   * create, drop, and list databases
    ///   * update the schema of pre-existing databases
    ///   * create, delete and list backups for a database
    ///   * restore a database from an existing backup
    #[derive(Debug, Clone)]
    pub struct DatabaseAdminClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl DatabaseAdminClient<tonic::transport::Channel> {
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
    impl<T> DatabaseAdminClient<T>
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
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> DatabaseAdminClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            DatabaseAdminClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Lists Cloud Spanner databases.
        pub async fn list_databases(
            &mut self,
            request: impl tonic::IntoRequest<super::ListDatabasesRequest>,
        ) -> Result<tonic::Response<super::ListDatabasesResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.admin.database.v1.DatabaseAdmin/ListDatabases",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Creates a new Cloud Spanner database and starts to prepare it for serving.
        /// The returned [long-running operation][google.longrunning.Operation] will
        /// have a name of the format `<database_name>/operations/<operation_id>` and
        /// can be used to track preparation of the database. The
        /// [metadata][google.longrunning.Operation.metadata] field type is
        /// [CreateDatabaseMetadata][google.spanner.admin.database.v1.CreateDatabaseMetadata]. The
        /// [response][google.longrunning.Operation.response] field type is
        /// [Database][google.spanner.admin.database.v1.Database], if successful.
        pub async fn create_database(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateDatabaseRequest>,
        ) -> Result<
            tonic::Response<super::super::super::super::super::longrunning::Operation>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.admin.database.v1.DatabaseAdmin/CreateDatabase",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Gets the state of a Cloud Spanner database.
        pub async fn get_database(
            &mut self,
            request: impl tonic::IntoRequest<super::GetDatabaseRequest>,
        ) -> Result<tonic::Response<super::Database>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.admin.database.v1.DatabaseAdmin/GetDatabase",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Updates the schema of a Cloud Spanner database by
        /// creating/altering/dropping tables, columns, indexes, etc. The returned
        /// [long-running operation][google.longrunning.Operation] will have a name of
        /// the format `<database_name>/operations/<operation_id>` and can be used to
        /// track execution of the schema change(s). The
        /// [metadata][google.longrunning.Operation.metadata] field type is
        /// [UpdateDatabaseDdlMetadata][google.spanner.admin.database.v1.UpdateDatabaseDdlMetadata].  The operation has no response.
        pub async fn update_database_ddl(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateDatabaseDdlRequest>,
        ) -> Result<
            tonic::Response<super::super::super::super::super::longrunning::Operation>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.admin.database.v1.DatabaseAdmin/UpdateDatabaseDdl",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Drops (aka deletes) a Cloud Spanner database.
        /// Completed backups for the database will be retained according to their
        /// `expire_time`.
        /// Note: Cloud Spanner might continue to accept requests for a few seconds
        /// after the database has been deleted.
        pub async fn drop_database(
            &mut self,
            request: impl tonic::IntoRequest<super::DropDatabaseRequest>,
        ) -> Result<tonic::Response<()>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.admin.database.v1.DatabaseAdmin/DropDatabase",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Returns the schema of a Cloud Spanner database as a list of formatted
        /// DDL statements. This method does not show pending schema updates, those may
        /// be queried using the [Operations][google.longrunning.Operations] API.
        pub async fn get_database_ddl(
            &mut self,
            request: impl tonic::IntoRequest<super::GetDatabaseDdlRequest>,
        ) -> Result<tonic::Response<super::GetDatabaseDdlResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.admin.database.v1.DatabaseAdmin/GetDatabaseDdl",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Sets the access control policy on a database or backup resource.
        /// Replaces any existing policy.
        ///
        /// Authorization requires `spanner.databases.setIamPolicy`
        /// permission on [resource][google.iam.v1.SetIamPolicyRequest.resource].
        /// For backups, authorization requires `spanner.backups.setIamPolicy`
        /// permission on [resource][google.iam.v1.SetIamPolicyRequest.resource].
        pub async fn set_iam_policy(
            &mut self,
            request: impl tonic::IntoRequest<
                super::super::super::super::super::iam::v1::SetIamPolicyRequest,
            >,
        ) -> Result<
            tonic::Response<super::super::super::super::super::iam::v1::Policy>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.admin.database.v1.DatabaseAdmin/SetIamPolicy",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Gets the access control policy for a database or backup resource.
        /// Returns an empty policy if a database or backup exists but does not have a
        /// policy set.
        ///
        /// Authorization requires `spanner.databases.getIamPolicy` permission on
        /// [resource][google.iam.v1.GetIamPolicyRequest.resource].
        /// For backups, authorization requires `spanner.backups.getIamPolicy`
        /// permission on [resource][google.iam.v1.GetIamPolicyRequest.resource].
        pub async fn get_iam_policy(
            &mut self,
            request: impl tonic::IntoRequest<
                super::super::super::super::super::iam::v1::GetIamPolicyRequest,
            >,
        ) -> Result<
            tonic::Response<super::super::super::super::super::iam::v1::Policy>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.admin.database.v1.DatabaseAdmin/GetIamPolicy",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Returns permissions that the caller has on the specified database or backup
        /// resource.
        ///
        /// Attempting this RPC on a non-existent Cloud Spanner database will
        /// result in a NOT_FOUND error if the user has
        /// `spanner.databases.list` permission on the containing Cloud
        /// Spanner instance. Otherwise returns an empty set of permissions.
        /// Calling this method on a backup that does not exist will
        /// result in a NOT_FOUND error if the user has
        /// `spanner.backups.list` permission on the containing instance.
        pub async fn test_iam_permissions(
            &mut self,
            request: impl tonic::IntoRequest<
                super::super::super::super::super::iam::v1::TestIamPermissionsRequest,
            >,
        ) -> Result<
            tonic::Response<
                super::super::super::super::super::iam::v1::TestIamPermissionsResponse,
            >,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.admin.database.v1.DatabaseAdmin/TestIamPermissions",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Starts creating a new Cloud Spanner Backup.
        /// The returned backup [long-running operation][google.longrunning.Operation]
        /// will have a name of the format
        /// `projects/<project>/instances/<instance>/backups/<backup>/operations/<operation_id>`
        /// and can be used to track creation of the backup. The
        /// [metadata][google.longrunning.Operation.metadata] field type is
        /// [CreateBackupMetadata][google.spanner.admin.database.v1.CreateBackupMetadata]. The
        /// [response][google.longrunning.Operation.response] field type is
        /// [Backup][google.spanner.admin.database.v1.Backup], if successful. Cancelling the returned operation will stop the
        /// creation and delete the backup.
        /// There can be only one pending backup creation per database. Backup creation
        /// of different databases can run concurrently.
        pub async fn create_backup(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateBackupRequest>,
        ) -> Result<
            tonic::Response<super::super::super::super::super::longrunning::Operation>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.admin.database.v1.DatabaseAdmin/CreateBackup",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Starts copying a Cloud Spanner Backup.
        /// The returned backup [long-running operation][google.longrunning.Operation]
        /// will have a name of the format
        /// `projects/<project>/instances/<instance>/backups/<backup>/operations/<operation_id>`
        /// and can be used to track copying of the backup. The operation is associated
        /// with the destination backup.
        /// The [metadata][google.longrunning.Operation.metadata] field type is
        /// [CopyBackupMetadata][google.spanner.admin.database.v1.CopyBackupMetadata].
        /// The [response][google.longrunning.Operation.response] field type is
        /// [Backup][google.spanner.admin.database.v1.Backup], if successful. Cancelling the returned operation will stop the
        /// copying and delete the backup.
        /// Concurrent CopyBackup requests can run on the same source backup.
        pub async fn copy_backup(
            &mut self,
            request: impl tonic::IntoRequest<super::CopyBackupRequest>,
        ) -> Result<
            tonic::Response<super::super::super::super::super::longrunning::Operation>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.admin.database.v1.DatabaseAdmin/CopyBackup",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Gets metadata on a pending or completed [Backup][google.spanner.admin.database.v1.Backup].
        pub async fn get_backup(
            &mut self,
            request: impl tonic::IntoRequest<super::GetBackupRequest>,
        ) -> Result<tonic::Response<super::Backup>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.admin.database.v1.DatabaseAdmin/GetBackup",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Updates a pending or completed [Backup][google.spanner.admin.database.v1.Backup].
        pub async fn update_backup(
            &mut self,
            request: impl tonic::IntoRequest<super::UpdateBackupRequest>,
        ) -> Result<tonic::Response<super::Backup>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.admin.database.v1.DatabaseAdmin/UpdateBackup",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Deletes a pending or completed [Backup][google.spanner.admin.database.v1.Backup].
        pub async fn delete_backup(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteBackupRequest>,
        ) -> Result<tonic::Response<()>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.admin.database.v1.DatabaseAdmin/DeleteBackup",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Lists completed and pending backups.
        /// Backups returned are ordered by `create_time` in descending order,
        /// starting from the most recent `create_time`.
        pub async fn list_backups(
            &mut self,
            request: impl tonic::IntoRequest<super::ListBackupsRequest>,
        ) -> Result<tonic::Response<super::ListBackupsResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.admin.database.v1.DatabaseAdmin/ListBackups",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Create a new database by restoring from a completed backup. The new
        /// database must be in the same project and in an instance with the same
        /// instance configuration as the instance containing
        /// the backup. The returned database [long-running
        /// operation][google.longrunning.Operation] has a name of the format
        /// `projects/<project>/instances/<instance>/databases/<database>/operations/<operation_id>`,
        /// and can be used to track the progress of the operation, and to cancel it.
        /// The [metadata][google.longrunning.Operation.metadata] field type is
        /// [RestoreDatabaseMetadata][google.spanner.admin.database.v1.RestoreDatabaseMetadata].
        /// The [response][google.longrunning.Operation.response] type
        /// is [Database][google.spanner.admin.database.v1.Database], if
        /// successful. Cancelling the returned operation will stop the restore and
        /// delete the database.
        /// There can be only one database being restored into an instance at a time.
        /// Once the restore operation completes, a new restore operation can be
        /// initiated, without waiting for the optimize operation associated with the
        /// first restore to complete.
        pub async fn restore_database(
            &mut self,
            request: impl tonic::IntoRequest<super::RestoreDatabaseRequest>,
        ) -> Result<
            tonic::Response<super::super::super::super::super::longrunning::Operation>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.admin.database.v1.DatabaseAdmin/RestoreDatabase",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Lists database [longrunning-operations][google.longrunning.Operation].
        /// A database operation has a name of the form
        /// `projects/<project>/instances/<instance>/databases/<database>/operations/<operation>`.
        /// The long-running operation
        /// [metadata][google.longrunning.Operation.metadata] field type
        /// `metadata.type_url` describes the type of the metadata. Operations returned
        /// include those that have completed/failed/canceled within the last 7 days,
        /// and pending operations.
        pub async fn list_database_operations(
            &mut self,
            request: impl tonic::IntoRequest<super::ListDatabaseOperationsRequest>,
        ) -> Result<
            tonic::Response<super::ListDatabaseOperationsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.admin.database.v1.DatabaseAdmin/ListDatabaseOperations",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Lists the backup [long-running operations][google.longrunning.Operation] in
        /// the given instance. A backup operation has a name of the form
        /// `projects/<project>/instances/<instance>/backups/<backup>/operations/<operation>`.
        /// The long-running operation
        /// [metadata][google.longrunning.Operation.metadata] field type
        /// `metadata.type_url` describes the type of the metadata. Operations returned
        /// include those that have completed/failed/canceled within the last 7 days,
        /// and pending operations. Operations returned are ordered by
        /// `operation.metadata.value.progress.start_time` in descending order starting
        /// from the most recently started operation.
        pub async fn list_backup_operations(
            &mut self,
            request: impl tonic::IntoRequest<super::ListBackupOperationsRequest>,
        ) -> Result<
            tonic::Response<super::ListBackupOperationsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.admin.database.v1.DatabaseAdmin/ListBackupOperations",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// Lists Cloud Spanner database roles.
        pub async fn list_database_roles(
            &mut self,
            request: impl tonic::IntoRequest<super::ListDatabaseRolesRequest>,
        ) -> Result<tonic::Response<super::ListDatabaseRolesResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/google.spanner.admin.database.v1.DatabaseAdmin/ListDatabaseRoles",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
