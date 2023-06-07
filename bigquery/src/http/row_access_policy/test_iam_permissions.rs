use crate::http::row_access_policy::RowAccessPolicyReference;
use crate::http::table::get_iam_policy::GetIamPolicyRequest;
use crate::http::table::test_iam_permissions::TestIamPermissionsRequest;
use crate::http::types::GetPolicyOptions;
use reqwest::{Client, RequestBuilder};

pub fn build(
    base_url: &str,
    client: &Client,
    project_id: &str,
    dataset_id: &str,
    table_id: &str,
    policy_id: &str,
    data: &TestIamPermissionsRequest,
) -> RequestBuilder {
    let url = format!(
        "{}/projects/{}/datasets/{}/tables/{}/rowAccessPolicies/{}/:testIamPermissions",
        base_url, project_id, dataset_id, table_id, policy_id
    );
    client.post(url).json(data)
}
