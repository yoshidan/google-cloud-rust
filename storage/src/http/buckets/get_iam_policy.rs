use reqwest_middleware::{ClientWithMiddleware as Client, RequestBuilder};

use crate::http::Escape;

/// Request message for `GetIamPolicy` method.
#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetIamPolicyRequest {
    /// REQUIRED: The resource for which the policy is being requested.
    /// See the operation documentation for the appropriate value for this field.
    #[serde(skip_serializing)]
    pub resource: String,
    /// Optional. The maximum policy version that will be used to format the
    /// policy.
    ///
    /// Valid values are 0, 1, and 3. Requests specifying an invalid value will be
    /// rejected.
    ///
    /// Requests for policies with any conditional role bindings must specify
    /// version 3. Policies with no conditional role bindings may specify any valid
    /// value or leave the field unset.
    ///
    /// The policy in the response might use the policy version that you specified,
    /// or it might use a lower policy version. For example, if you specify version
    /// 3, but the policy has no conditional role bindings, the response uses
    /// version 1.
    ///
    /// To learn which resources support conditions in their IAM policies, see the
    /// [IAM
    /// documentation](<https://cloud.google.com/iam/help/conditions/resource-policies>).
    pub options_requested_policy_version: Option<i32>,
}

pub(crate) fn build(base_url: &str, client: &Client, req: &GetIamPolicyRequest) -> RequestBuilder {
    let url = format!("{}/b/{}/iam", base_url, req.resource.escape());
    client.get(url).query(&req)
}
