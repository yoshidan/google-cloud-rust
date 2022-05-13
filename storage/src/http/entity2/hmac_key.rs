use crate::http::entity2::{MaxResults, PageToken};

/// Hmac Key Metadata, which includes all information other than the secret.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HmacKeyMetadata {
    /// Resource name ID of the key in the format <projectId>/<accessId>.
    pub id: String,
    /// Globally unique id for keys.
    pub access_id: String,
    /// The project ID that the hmac key is contained in.
    pub project_id: String,
    /// Email of the service account the key authenticates as.
    pub service_account_email: String,
    /// State of the key. One of ACTIVE, INACTIVE, or DELETED.
    pub state: String,
    /// The creation time of the HMAC key in RFC 3339 format.
    pub time_created: Option<chrono::DateTime<chrono::Utc>>,
    /// The last modification time of the HMAC key metadata in RFC 3339 format.
    pub updated: Option<chrono::DateTime<chrono::Utc>>,
    /// Tag updated with each key update.
    pub etag: String,
}
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateHmacKeyRequest {
    /// Required. The project that the HMAC-owning service account lives in.
    pub project_id: String,
    /// Required. The service account to create the HMAC for.
    pub service_account_email: String,
}
/// Create hmac response.  The only time the secret for an HMAC will be returned.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateHmacKeyResponse {
    /// Key metadata.
    pub metadata: HmacKeyMetadata,
    /// HMAC key secret material.
    pub secret: String,
}
/// Request object to delete a given HMAC key.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteHmacKeyRequest {
    /// Required. The identifying key for the HMAC to delete.
    pub access_id: String,
    /// Required. The project id the HMAC key lies in.
    pub project_id: String,
}
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListHmacKeysRequest {
    /// Required. The project id to list HMAC keys for.
    pub project_id: String,
    /// An optional filter to only return HMAC keys for one service account.
    pub service_account_email: Option<String>,
    /// An optional bool to return deleted keys that have not been wiped out yet.
    pub show_deleted_keys: bool,
    /// The maximum number of keys to return.
    pub max_results: Option<MaxResults>,
    /// A previously returned token from ListHmacKeysResponse to get the next page.
    pub page_token: Option<PageToken>,
}
/// Hmac key list response with next page information.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListHmacKeysResponse {
    /// The continuation token, used to page through large result sets. Provide
    /// this value in a subsequent request to return the next page of results.
    pub next_page_token: String,
    /// The list of items.
    pub items: Vec<HmacKeyMetadata>,
}
/// Request object to update an HMAC key state.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateHmacKeyRequest {
    /// Required. The id of the HMAC key.
    pub access_id: String,
    /// Required. The project id the HMAC's service account lies in.
    pub project_id: String,
    /// Required. The service account owner of the HMAC key.
    pub metadata: Option<HmacKeyMetadata>,
}
/// Request object to get metadata on a given HMAC key.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetHmacKeyRequest {
    /// Required. The identifying key for the HMAC to delete.
    pub access_id: String,
    /// Required. The project id the HMAC key lies in.
    pub project_id: String,
}