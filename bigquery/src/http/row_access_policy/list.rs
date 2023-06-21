use reqwest::{Client, RequestBuilder};
use time::OffsetDateTime;

use crate::http::row_access_policy::RowAccessPolicyReference;

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListRowAccessPoliciesRequest {
    /// The maximum number of results to return in a single response page.
    /// Leverage the page tokens to iterate through the entire collection.
    pub page_size: Option<i64>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListRowAccessPoliciesResponse {
    /// Row access policies on the requested table.
    pub row_access_policies: Option<Vec<RowAccessPolicyOverview>>,
    /// A token to request the next page of results.
    pub next_page_token: Option<String>,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct RowAccessPolicyOverview {
    /// Required. Reference describing the ID of this row access policy.
    pub row_access_policy_reference: RowAccessPolicyReference,
    /// Required.
    /// A SQL boolean expression that represents the rows defined by this row access policy,
    /// similar to the boolean expression in a WHERE clause of a SELECT query on a table.
    /// References to other tables, routines, and temporary functions are not supported.
    /// Examples:
    /// region="EU" date_field = CAST('2019-9-27' as DATE)
    /// nullable_field is not NULL numeric_field BETWEEN 1.0 AND 5.0
    pub filter_predicate: String,
    /// Output only. The time when this row access policy was created, in milliseconds since the epoch.
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub creation_time: Option<OffsetDateTime>,
    /// Output only. The time when this row access policy was last modified, in milliseconds since the epoch.
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub last_modified_time: Option<OffsetDateTime>,
}

pub fn build(
    base_url: &str,
    client: &Client,
    project_id: &str,
    dataset_id: &str,
    table_id: &str,
    data: &ListRowAccessPoliciesRequest,
    page_token: Option<String>,
) -> RequestBuilder {
    let url = format!(
        "{}/projects/{}/datasets/{}/tables/{}/rowAccessPolicies",
        base_url, project_id, dataset_id, table_id
    );
    let builder = client.get(url).query(data);
    if let Some(page_token) = page_token {
        builder.query(&[("pageToken", page_token.as_str())])
    } else {
        builder
    }
}
