#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RowAccessPolicyReference {
    /// Required. The ID of the project containing this table.
    pub project_id: String,
    /// Required. The ID of the dataset containing this table.
    pub dataset_id: String,
    /// Required. The ID of the table containing this row access policy.
    pub table_id: String,
    /// Required. The ID of the row access policy.
    /// The ID must contain only letters (a-z, A-Z), numbers (0-9), or underscores (_). The maximum length is 256 characters.
    pub policy_id: String,
}
