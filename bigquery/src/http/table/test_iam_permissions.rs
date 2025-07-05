use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

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

pub(crate) fn build(
    base_url: &str,
    client: &Client,
    project_id: &str,
    dataset_id: &str,
    table_id: &str,
    req: &TestIamPermissionsRequest,
) -> RequestBuilder {
    let url = format!("{base_url}/projects/{project_id}/datasets/{dataset_id}/tables/{table_id}:testIamPermissions");
    client.post(url).json(&req)
}
