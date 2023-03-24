pub(crate) mod delete;
pub(crate) mod get;
pub(crate) mod insert;
pub(crate) mod list;
pub(crate) mod patch;

use crate::http::routine::RoutineReference;
use crate::http::table::TableReference;
use crate::http::types::EncryptionConfiguration;

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct GcpTag {
    /// Required. The namespaced friendly name of the tag key, e.g. "12345/environment" where 12345 is org id.
    pub tag_key: String,
    /// Required. The friendly short name of the tag value, e.g. "production"
    pub tag_value: String,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StorageBillingModel {
    /// Value not set.
    #[default]
    StorageBillingModelUnspecified,
    /// Billing for logical bytes.
    Logical,
    /// Billing for physical bytes.
    Physical,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TargetType {
    /// This entry applies to views in the dataset.
    #[default]
    Views,
    /// Do not use. You must set a target type explicitly.
    TargetTypeUnspecified,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct LinkedDatasetSource {
    /// The source dataset reference contains project numbers and not project ids.
    pub source_dataset: DatasetReference,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct DatasetReference {
    /// Required. A unique ID for this dataset, without the project name.
    /// The ID must contain only letters (a-z, A-Z), numbers (0-9), or underscores (_).
    /// The maximum length is 1,024 characters.
    pub dataset_id: String,
    /// Optional. The ID of the project containing this dataset.
    pub project_id: Option<String>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct DatasetAccessEntry {
    /// The dataset this entry applies to
    pub dataset: DatasetReference,
    /// Which resources in the dataset this entry applies to.
    /// Currently, only views are supported, but additional target types may be added in the future.
    pub target_types: Vec<TargetType>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub enum SpecialGroup {
    #[default]
    ProjectOwners,
    ProjectReaders,
    ProjectWriters,
    AllAuthenticatedUsers,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Access {
    /// An IAM role ID that should be granted to the user, group, or domain specified in this access entry.
    /// The following legacy mappings will be applied:
    ///     OWNER <=> roles/bigquery.dataOwner
    ///     WRITER <=> roles/bigquery.dataEditor
    ///     READER <=> roles/bigquery.dataViewer
    /// This field will accept any of the above formats, but will return only the legacy format.
    /// For example, if you set this field to "roles/bigquery.dataOwner", it will be returned back as "OWNER".
    pub role: String,
    /// [Pick one] An email address of a user to grant access to.
    /// For example: fred@example.com.
    /// Maps to IAM policy member "user:EMAIL" or "serviceAccount:EMAIL".
    pub user_by_email: Option<String>,
    /// [Pick one] An email address of a Google Group to grant access to.
    /// Maps to IAM policy member "group:GROUP".
    pub group_by_email: Option<String>,
    /// [Pick one] A domain to grant access to.
    /// Any users signed in with the domain specified will be granted the specified access.
    /// Example: "example.com".
    /// Maps to IAM policy member "domain:DOMAIN".
    pub domain: Option<String>,
    /// [Pick one] A special group to grant access to.
    /// Possible values include:
    ///     projectOwners: Owners of the enclosing project.
    ///     projectReaders: Readers of the enclosing project.
    ///     projectWriters: Writers of the enclosing project.
    ///     allAuthenticatedUsers: All authenticated BigQuery users.
    /// Maps to similarly-named IAM members.
    pub special_group: Option<SpecialGroup>,
    /// [Pick one] Some other type of member that appears in the IAM Policy but isn't a user,
    /// group, domain, or special group.
    pub iam_member: Option<String>,
    /// [Pick one] A view from a different dataset to grant access to.
    /// Queries executed against that view will have read access to views/tables/routines in this dataset.
    /// The role field is not required when this field is set. If that view is updated by any user,
    /// access to the view needs to be granted again via an update operation.
    pub view: Option<TableReference>,
    /// [Pick one] A routine from a different dataset to grant access to.
    /// Queries executed against that routine will have read access to views/tables/routines in this dataset.
    /// Only UDF is supported for now. The role field is not required when this field is set.
    /// If that routine is updated by any user,
    /// access to the routine needs to be granted again via an update operation.
    pub routine: Option<RoutineReference>,
    /// [Pick one] A grant authorizing all resources of a particular type in a particular dataset access to this dataset.
    /// Only views are supported for now. The role field is not required when this field is set.
    /// If that dataset is deleted and re-created, its access needs to be granted again via an update operation.
    pub dataset: Option<DatasetAccessEntry>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Dataset {
    /// Output only. The resource type.
    pub kind: String,
    /// Output only. A hash of the resource.
    pub etag: String,
    /// Output only. The fully-qualified unique name of the dataset in the format projectId:datasetId.
    /// The dataset name without the project name is given in the datasetId field. When creating a new dataset,
    /// leave this field blank, and instead specify the datasetId field.
    pub id: String,
    /// Output only. A URL that can be used to access the resource again.
    /// You can use this URL in Get or Update requests to the resource.
    pub self_link: String,
    /// Required. A reference that identifies the dataset.
    pub dataset_reference: DatasetReference,
    /// Optional. A descriptive name for the dataset.
    pub friendly_name: Option<String>,
    /// Optional. Optional. A user-friendly description of the dataset.
    pub description: Option<String>,
    /// Optional. The default lifetime of all tables in the dataset, in milliseconds.
    /// The minimum lifetime value is 3600000 milliseconds (one hour).
    /// To clear an existing default expiration with a PATCH request, set to 0. Once this property is set,
    /// all newly-created tables in the dataset will have an expirationTime property set to
    /// the creation time plus the value in this property,
    /// and changing the value will only affect new tables, not existing ones.
    /// When the expirationTime for a given table is reached, that table will be deleted automatically.
    /// If a table's expirationTime is modified or removed before the table expires,
    /// or if you provide an explicit expirationTime when creating a table,
    /// that value takes precedence over the default expiration time indicated by this property.
    #[serde(deserialize_with = "crate::http::from_str_option")]
    #[serde(default)]
    pub default_table_expiration_ms: Option<i64>,
    /// This default partition expiration, expressed in milliseconds.
    ///
    /// When new time-partitioned tables are created in a dataset where this property is set,
    /// the table will inherit this value, propagated as the TimePartitioning.expirationMs property on the new table.
    /// If you set TimePartitioning.expirationMs explicitly when creating a table,
    /// the defaultPartitionExpirationMs of the containing dataset is ignored.
    ///
    /// When creating a partitioned table, if defaultPartitionExpirationMs is set,
    /// the defaultTableExpirationMs value is ignored and the table will not be inherit a table expiration deadline.
    #[serde(deserialize_with = "crate::http::from_str_option")]
    #[serde(default)]
    pub default_partition_expiration_ms: Option<i64>,
    /// The labels associated with this dataset.
    /// You can use these to organize and group your datasets.
    /// You can set this property when inserting or updating a dataset.
    /// See Creating and Updating Dataset Labels for more information.
    ///
    /// An object containing a list of "key": value pairs.
    /// Example: { "name": "wrench", "mass": "1.3kg", "count": "3" }.
    pub labels: Option<std::collections::HashMap<String, String>>,
    /// Optional. An array of objects that define dataset access for one or more entities.
    /// You can set this property when inserting or updating a dataset
    /// in order to control who is allowed to access the data.
    /// If unspecified at dataset creation time, BigQuery adds default dataset access for the following entities:
    ///     access.specialGroup: projectReaders;
    ///     access.role: READER;
    ///     access.specialGroup: projectWriters;
    ///     access.role: WRITER;
    ///     access.specialGroup: projectOwners;
    ///     access.role: OWNER;
    ///     access.userByEmail: [dataset creator email];
    ///     access.role: OWNER;
    pub access: Vec<Access>,
    /// Output only. The time when this dataset was created, in milliseconds since the epoch.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub creation_time: i64,
    /// Output only. The date when this dataset was last modified, in milliseconds since the epoch.
    #[serde(deserialize_with = "crate::http::from_str")]
    pub last_modified_time: i64,
    /// The geographic location where the dataset should reside.
    /// See https://cloud.google.com/bigquery/docs/locations for supported locations.
    pub location: String,
    /// The default encryption key for all tables in the dataset.
    /// Once this property is set, all newly-created partitioned tables in the dataset will have encryption key set to this value,
    /// unless table creation request (or query) overrides the key.
    pub default_encryption_configuration: Option<EncryptionConfiguration>,
    /// Output only. Reserved for future use.
    pub satisfies_pzs: Option<bool>,
    /// Optional. The source when the dataset is of type LINKED.
    pub linked_dataset_source: Option<LinkedDatasetSource>,
    /// Optional. TRUE if the dataset and its table names are case-insensitive, otherwise FALSE.
    /// By default, this is FALSE, which means the dataset and its table names are case-sensitive.
    /// This field does not affect routine references.
    pub is_case_insensitive: Option<bool>,
    /// Optional. Defines the default collation specification of future tables created in the dataset.
    /// If a table is created in this dataset without table-level default collation,
    /// then the table inherits the dataset default collation,
    /// which is applied to the string fields that do not have explicit collation specified.
    /// A change to this field affects only tables created afterwards,
    /// and does not alter the existing tables.
    /// The following values are supported:
    ///
    /// 'und:ci': undetermined locale, case insensitive.
    /// '': empty string. Default to case-sensitive behavior.
    pub default_collation: Option<String>,
    /// Optional. Defines the time travel window in hours.
    /// The value can be from 48 to 168 hours (2 to 7 days).
    /// The default value is 168 hours if this is not set.
    #[serde(deserialize_with = "crate::http::from_str_option")]
    #[serde(default)]
    pub max_time_travel_hours: Option<i64>,
    /// Output only. Tags for the Dataset.
    pub tags: Option<Vec<GcpTag>>,
    /// Optional. Updates storageBillingModel for the dataset.
    pub storage_billing_model: Option<StorageBillingModel>,
}
