#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct StandardSqlDataType {
    /// Required. The top level type of this field. Can be any GoogleSQL data type (e.g., "INT64", "DATE", "ARRAY").
    pub type_kind: TypeKind,
}
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct StandardSqlStructType {
    pub fields: Vec<StandardSqlField>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct StandardSqlField {
    /// Optional. The name of this field.
    /// Can be absent for struct fields.
    pub name: Option<String>,
    /// Optional. The type of this parameter.
    /// Absent if not explicitly specified (e.g., CREATE FUNCTION statement can omit the return type; in this case the output parameter does not have this "type" field).
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub field_type: Option<StandardSqlDataType>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum StandardSqlDataSubType {
    /// The type of the array's elements, if typeKind = "ARRAY".
    ArrayElementType(StandardSqlDataType),
    /// The fields of this struct, in order, if typeKind = "STRUCT".
    StructType(StandardSqlStructType),
}

impl Default for StandardSqlDataSubType {
    fn default() -> Self {
        Self::ArrayElementType(StandardSqlDataType::default())
    }
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TypeKind {
    #[default]
    TypeKindUnspecified,
    Int64,
    Bool,
    Float64,
    String,
    Bytes,
    Timestamp,
    Date,
    Time,
    Datetime,
    Geography,
    Numeric,
    Bignumeric,
    Json,
    Array,
    Struct,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct DataFormatOptions {
    /// Optional. Output timestamp as usec int64. Default is false.
    pub use_int64_timestamp: Option<bool>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(untagged)]
pub enum Collation {
    /// '': empty string. Default to case-sensitive behavior.
    #[default]
    #[serde(rename = "")]
    Default,
    /// 'und:ci': undetermined locale, case insensitive.
    #[serde(rename = "und:ci")]
    UndeterminedLocaleCaseInsensitive,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct EncryptionConfiguration {
    /// Optional. Describes the Cloud KMS encryption key that will be used to protect destination BigQuery table.
    /// The BigQuery Service Account associated with your project requires access to this encryption key.
    pub kms_key_name: Option<String>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct QueryParameter {
    /// Optional. If unset, this is a positional parameter. Otherwise, should be unique within a query.
    pub name: Option<String>,
    /// Required. The type of this parameter.
    pub parameter_type: QueryParameterType,
    /// Required. The value of this parameter.
    pub parameter_value: QueryParameterValue,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct QueryParameterStructType {
    /// Optional. The name of this field.
    pub name: Option<String>,
    /// Required. The type of this field.
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub field_type: QueryParameterType,
    /// Optional. Human-oriented description of the field.
    pub description: Option<String>,
}
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct QueryParameterType {
    /// Required. The top level type of this field.
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub parameter_type: String,
    /// Optional. The type of the array's elements, if this is an array.
    pub array_type: Option<Box<QueryParameterType>>,
    /// Optional. The types of the fields of this struct, in order, if this is a struct.
    pub struct_types: Option<Vec<QueryParameterStructType>>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct QueryParameterValue {
    /// Optional. The value of this value, if a simple scalar type.
    pub value: Option<String>,
    /// Optional. The array values, if this is an array type.
    pub array_values: Option<Vec<QueryParameterValue>>,
    /// The struct field values.
    /// An object containing a list of "key": value pairs.
    /// Example: { "name": "wrench", "mass": "1.3kg", "count": "3" }..
    pub struct_values: Option<Box<QueryParameterValue>>,
}

/// Currently supported connection properties:
/// A connection-level property to customize query behavior. Under JDBC, these correspond directly to connection properties passed to the DriverManager.
/// Under ODBC, these correspond to properties in the connection string.
/// dataset_project_id: represents the default project for datasets that are used in the query. Setting the system variable @@dataset_project_id achieves the same behavior.
/// time_zone: represents the default timezone used to run the query.
/// session_id: associates the query with a given session.
/// query_label: associates the query with a given job label. If set, all subsequent queries in a script or session will have this label. For the format in which a you can specify a query label, see labels in the JobConfiguration resource type. Additional properties are allowed, but ignored. Specifying multiple connection properties with the same key returns an error.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionProperty {
    pub key: String,
    pub value: String,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogConfig {
    /// The log type that this config enables.
    pub log_type: LogType,
    /// Specifies the identities that do not cause logging for this type of permission.
    /// Follows the same format of Binding.members.
    pub exempted_members: Vec<String>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LogType {
    /// Admin reads. Example: CloudIAM getIamPolicy.
    #[default]
    AdminRead,
    /// Data writes. Example: CloudSQL Users create.
    DataWrite,
    /// Data reads. Example: CloudSQL Users list.
    DataRead,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct AuditConfig {
    /// Specifies a service that will be enabled for audit logging. For example, storage.googleapis.com, cloudsql.googleapis.com.
    /// allServices is a special value that covers all services.
    pub service: String,
    /// The configuration for logging of each type of permission.
    pub audit_log_configs: Vec<AuditLogConfig>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetPolicyOptions {
    /// Optional. The maximum policy version that will be used to format the policy.
    /// Valid values are 0, 1, and 3. Requests specifying an invalid value will be rejected.
    /// Requests for policies with any conditional role bindings must specify version 3. Policies with no conditional role bindings may specify any valid value or leave the field unset.
    /// The policy in the response might use the policy version that you specified, or it might use a lower policy version. For example, if you specify version 3, but the policy has no conditional role bindings, the response uses version 1.
    /// To learn which resources support conditions in their IAM policies, see the IAM documentation.
    pub requested_policy_version: Option<i32>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct PolicyExpr {
    /// Textual representation of an expression in Common Expression Language syntax.
    pub expression: String,
    /// Optional. Title for the expression, i.e. a short string describing its purpose.
    /// This can be used e.g. in UIs which allow to enter the expression.
    pub title: Option<String>,
    /// Optional. Description of the expression.
    /// This is a longer text which describes the expression, e.g. when hovered over it in a UI.
    pub description: Option<String>,
    /// Optional. String indicating the location of the expression for error reporting,
    /// e.g. a file name and a position in the file.
    pub location: Option<String>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Bindings {
    /// Role that is assigned to the list of members, or principals.
    /// For example, roles/viewer, roles/editor, or roles/owner.
    pub role: String,
    /// Specifies the principals requesting access for a Google Cloud resource.
    /// members can have the following values:
    ///
    /// allUsers: A special identifier that represents anyone who is on the internet; with or without a Google account.
    /// allAuthenticatedUsers: A special identifier that represents anyone who is authenticated with a Google account or a service account. Does not include identities that come from external identity providers (IdPs) through identity federation.
    /// user:{emailid}: An email address that represents a specific Google account. For example, alice@example.com .
    /// serviceAccount:{emailid}: An email address that represents a Google service account. For example, my-other-app@appspot.gserviceaccount.com.
    /// serviceAccount:{projectid}.svc.id.goog[{namespace}/{kubernetes-sa}]: An identifier for a Kubernetes service account. For example, my-project.svc.id.goog[my-namespace/my-kubernetes-sa].
    /// group:{emailid}: An email address that represents a Google group. For example, admins@example.com.
    /// domain:{domain}: The G Suite domain (primary) that represents all the users of that domain. For example, google.com or example.com.
    /// deleted:user:{emailid}?uid={uniqueid}: An email address (plus unique identifier) representing a user that has been recently deleted. For example, alice@example.com?uid=123456789012345678901. If the user is recovered, this value reverts to user:{emailid} and the recovered user retains the role in the binding.
    /// deleted:serviceAccount:{emailid}?uid={uniqueid}: An email address (plus unique identifier) representing a service account that has been recently deleted. For example, my-other-app@appspot.gserviceaccount.com?uid=123456789012345678901. If the service account is undeleted, this value reverts to serviceAccount:{emailid} and the undeleted service account retains the role in the binding.
    /// deleted:group:{emailid}?uid={uniqueid}: An email address (plus unique identifier) representing a Google group that has been recently deleted. For example, admins@example.com?uid=123456789012345678901. If the group is recovered, this value reverts to group:{emailid} and the recovered group retains the role in the binding.
    pub members: Vec<String>,
    /// The condition that is associated with this binding.
    ///
    /// If the condition evaluates to true, then this binding applies to the current request.
    ///
    /// If the condition evaluates to false, then this binding does not apply to the current request. However, a different role binding might grant the same role to one or more of the principals in this binding.
    ///
    /// To learn which resources support conditions in their IAM policies, see the IAM documentation.
    pub condition: Option<PolicyExpr>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Policy {
    /// Specifies the format of the policy.
    ///
    /// Valid values are 0, 1, and 3. Requests that specify an invalid value are rejected.
    ///
    /// Any operation that affects conditional role bindings must specify version 3. This requirement applies to the following operations:
    ///
    /// Getting a policy that includes a conditional role binding
    /// Adding a conditional role binding to a policy
    /// Changing a conditional role binding in a policy
    /// Removing any role binding, with or without a condition, from a policy that includes conditions
    /// Important: If you use IAM Conditions, you must include the etag field whenever you call setIamPolicy. If you omit this field, then IAM allows you to overwrite a version 3 policy with a version 1 policy, and all of the conditions in the version 3 policy are lost.
    ///
    /// If a policy does not include any conditions, operations on that policy may specify any valid version or leave the field unset.
    ///
    /// To learn which resources support conditions in their IAM policies, see the IAM documentation.
    pub version: Option<i32>,
    /// Associates a list of members, or principals, with a role. Optionally, may specify a condition that determines how and when the bindings are applied. Each of the bindings must contain at least one principal.
    ///
    /// The bindings in a Policy can refer to up to 1,500 principals;
    /// up to 250 of these principals can be Google groups.
    /// Each occurrence of a principal counts towards these limits.
    /// For example, if the bindings grant 50 different roles to user:alice@example.com,
    /// and not to any other principal, then you can add another 1,450 principals to the bindings in the Policy.
    pub bindings: Vec<Bindings>,
    /// Specifies cloud audit logging configuration for this policy.
    pub audit_configs: Option<AuditConfig>,
    /// etag is used for optimistic concurrency control as a way to help prevent simultaneous updates of a policy
    /// from overwriting each other.
    /// It is strongly suggested that systems make use of the etag in
    /// the read-modify-write cycle to perform policy updates in order to avoid race conditions:
    /// An etag is returned in the response to getIamPolicy,
    /// and systems are expected to put that etag in the request
    /// to setIamPolicy to ensure that their change will be applied to the same version of the policy.
    ///
    /// Important: If you use IAM Conditions,
    /// you must include the etag field whenever you call setIamPolicy.
    /// If you omit this field, then IAM allows you to overwrite a version 3 policy with a version 1 policy,
    /// and all of the conditions in the version 3 policy are lost.
    ///
    /// A base64-encoded string.
    pub etag: Option<String>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ErrorProto {
    /// A short error code that summarizes the error.
    pub reason: Option<String>,
    /// Specifies where the error occurred, if present.
    pub location: Option<String>,
    /// A human-readable description of the error.
    pub message: Option<String>,
}
