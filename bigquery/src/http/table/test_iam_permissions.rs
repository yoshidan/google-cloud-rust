use reqwest::{Client, RequestBuilder};

use crate::http::types::Policy;

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TestIamPermissionsRequest {
    /// The set of permissions to check for the resource.
    /// Permissions with wildcards (such as * or storage.*) are not allowed.
    /// For more information see IAM Overview.
    pub permissions: Vec<String>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TestIamPermissionsResponse {
    /// A subset of TestPermissionsRequest.permissions that the caller is allowed.
    pub permissions: Vec<String>,
}

pub(crate) fn build(base_url: &str, client: &Client, project_id: &str, dataset_id:&str, table_id: &str, req: &TestIamPermissionsRequest) -> RequestBuilder {
    let url = format!("{}/projects/{}/datasets/{}/tables/{}:testIamPermissions", base_url, project_id, dataset_id, table_id);
    client.post(url).json(&req)
}
