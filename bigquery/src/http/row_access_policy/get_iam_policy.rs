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
        "{}/projects/{}/datasets/{}/tables/{}/rowAccessPolicies/{}/:getIamPolicy",
        base_url, project_id, dataset_id, table_id, policy_id
    );
    client.post(url).json(data)
}
