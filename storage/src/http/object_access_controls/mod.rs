pub mod delete;
pub mod get;
pub mod insert;
pub mod list;
pub mod patch;

/// An access-control entry.
#[derive(Clone, PartialEq, Eq, Default, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ObjectAccessControl {
    pub bucket: Option<String>,
    pub domain: Option<String>,
    pub email: Option<String>,
    pub entity: String,
    pub entity_id: Option<String>,
    pub etag: String,
    #[serde(deserialize_with = "crate::http::from_str_option")]
    #[serde(default)]
    pub generation: Option<i64>,
    pub id: Option<String>,
    pub kind: String,
    pub object: Option<String>,
    pub project_team: Option<ProjectTeam>,
    pub role: ObjectACLRole,
    pub self_link: Option<String>,
}

/// A set of properties to return in a response.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Deserialize, serde::Serialize, Debug, Default)]
pub enum ObjectACLRole {
    #[default]
    READER,
    OWNER,
}

/// Predefined or "canned" aliases for sets of specific object ACL entries.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum PredefinedObjectAcl {
    /// Object owner gets `OWNER` access, and
    /// `allAuthenticatedUsers` get `READER` access.
    AuthenticatedRead,
    /// Object owner gets `OWNER` access, and project team owners get
    /// `OWNER` access.
    BucketOwnerFullControl,
    /// Object owner gets `OWNER` access, and project team owners get
    /// `READER` access.
    BucketOwnerRead,
    /// Object owner gets `OWNER` access.
    Private,
    /// Object owner gets `OWNER` access, and project team members get
    /// access according to their roles.
    ProjectPrivate,
    /// Object owner gets `OWNER` access, and `allUsers`
    /// get `READER` access.
    PublicRead,
}

/// Represents the Viewers, Editors, or Owners of a given project.
#[derive(Clone, PartialEq, Eq, Default, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProjectTeam {
    /// The project number.
    #[serde(default)]
    pub project_number: String,
    /// The team.
    #[serde(default)]
    pub team: String,
}

/// A set of properties to return in a response.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Projection {
    /// Omit `owner`, `acl`, and `defaultObjectAcl` properties.
    NoAcl,
    /// Include all properties.
    Full,
}
