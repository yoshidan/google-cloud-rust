use crate::http::buckets::Policy;

use crate::http::{Escape, BASE_URL};

use reqwest::{Client, RequestBuilder};

/// Request message for `SetIamPolicy` method.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetIamPolicyRequest {
    /// REQUIRED: The resource for which the policy is being specified.
    /// See the operation documentation for the appropriate value for this field.
    pub resource: String,
    /// REQUIRED: The complete policy to be applied to the `resource`. The size of
    /// the policy is limited to a few 10s of KB. An empty policy is a
    /// valid policy but certain Cloud Platform services (such as Projects)
    /// might reject them.
    pub policy: Policy,
}

pub(crate) fn build(client: &Client, req: &SetIamPolicyRequest) -> RequestBuilder {
    let url = format!("{}/b/{}/iam", BASE_URL, req.resource.escape());
    client.put(url).json(&req.policy)
}
