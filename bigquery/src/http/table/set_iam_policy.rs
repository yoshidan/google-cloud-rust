use reqwest::{Client, RequestBuilder};

use crate::http::types::Policy;

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetIamPolicyRequest {
    /// REQUIRED: The complete policy to be applied to the resource.
    /// The size of the policy is limited to a few 10s of KB.
    /// An empty policy is a valid policy but certain Google Cloud services (such as Projects) might reject them.
    pub policy: Policy,
    /// OPTIONAL: A FieldMask specifying which fields of the policy to modify. Only the fields in the mask will be modified. If no mask is provided, the following default mask is used:
    ///
    /// paths: "bindings, etag"
    ///
    /// This is a comma-separated list of fully qualified names of fields. Example: "user.displayName,photo".
    pub update_mask: Option<String>
}

pub(crate) fn build(base_url: &str, client: &Client, project_id: &str, dataset_id:&str, table_id: &str, req: &SetIamPolicyRequest) -> RequestBuilder {
    let url = format!("{}/projects/{}/datasets/{}/tables/{}:setIamPolicy?alt=json", base_url, project_id, dataset_id, table_id);
    client.post(url).json(&req)
}
