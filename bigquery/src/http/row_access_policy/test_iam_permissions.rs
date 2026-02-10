use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::table::test_iam_permissions::TestIamPermissionsRequest;

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
        "{base_url}/projects/{project_id}/datasets/{dataset_id}/tables/{table_id}/rowAccessPolicies/{policy_id}/:testIamPermissions"
    );
    client.post(url).json(data)
}
