use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::Escape;

/// Request message for `TestIamPermissions` method.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TestIamPermissionsRequest {
    /// REQUIRED: The resource for which the policy detail is being requested.
    /// See the operation documentation for the appropriate value for this field.
    pub resource: String,
    /// The set of permissions to check for the `resource`. Permissions with
    /// wildcards (such as '*' or 'storage.*') are not allowed. For more
    /// information see
    /// [IAM Overview](<https://cloud.google.com/iam/docs/overview#permissions>).
    pub permissions: Vec<String>,
}

/// Response message for `TestIamPermissions` method.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TestIamPermissionsResponse {
    /// A subset of `TestPermissionsRequest.permissions` that the caller is
    /// allowed.
    pub permissions: Vec<String>,
}

pub(crate) fn build(base_url: &str, client: &Client, req: &TestIamPermissionsRequest) -> RequestBuilder {
    let url = format!("{}/b/{}/iam/testPermissions", base_url, req.resource.escape());
    let query: Vec<_> = req.permissions.iter().map(|x| ("permissions", x)).collect();
    client.get(url).query(&query)
}
