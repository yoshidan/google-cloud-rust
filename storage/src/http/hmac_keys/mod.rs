pub mod create;
pub mod delete;
pub mod get;
pub mod list;
pub mod update;

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
