use reqwest::{Client, RequestBuilder};

use crate::http::types::GetPolicyOptions;

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetIamPolicyRequest {
    /// OPTIONAL: A GetPolicyOptions object for specifying options to tables.getIamPolicy.
    pub options: Option<GetPolicyOptions>,
}

pub(crate) fn build(base_url: &str, client: &Client,  project_id: &str, dataset_id:&str, table_id: &str, req: &GetIamPolicyRequest) -> RequestBuilder {
    let url = format!("{}/projects/{}/datasets/{}/tables/{}:getIamPolicy", base_url, project_id, dataset_id, table_id);
    client.post(url).json(&req)
}
