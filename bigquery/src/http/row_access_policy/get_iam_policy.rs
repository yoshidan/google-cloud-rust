use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::table::get_iam_policy::GetIamPolicyRequest;

pub fn build(
    base_url: &str,
    client: &Client,
    project_id: &str,
    dataset_id: &str,
    table_id: &str,
    policy_id: &str,
    data: &GetIamPolicyRequest,
) -> RequestBuilder {
    let url = format!(
        "{base_url}/projects/{project_id}/datasets/{dataset_id}/tables/{table_id}/rowAccessPolicies/{policy_id}/:getIamPolicy"
    );
    client.post(url).json(data)
}
