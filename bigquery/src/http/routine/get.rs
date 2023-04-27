use reqwest::{Client, RequestBuilder};

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct GetRoutineRequest {
    /// If set, only the Routine fields in the field mask are returned in the response. If unset, all Routine fields are returned.
    /// This is a comma-separated list of fully qualified names of fields. Example: "user.displayName,photo".
    pub read_mask: Option<String>,
}

pub fn build(
    base_url: &str,
    client: &Client,
    project_id: &str,
    dataset_id: &str,
    routine_id: &str,
    data: &GetRoutineRequest,
) -> RequestBuilder {
    let url = format!(
        "{}/projects/{}/datasets/{}/routines/{}",
        base_url, project_id, dataset_id, routine_id
    );
    client.get(url).query(data)
}
