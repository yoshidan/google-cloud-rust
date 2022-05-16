pub mod insert;

/// An access-control entry.
#[derive(Clone, PartialEq, Default, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ObjectAccessControl {
    pub bucket: Option<String>,
    pub domain: Option<String>,
    pub email: Option<String>,
    pub entity: String,
    pub entity_id: Option<String>,
    pub etag: String,
    pub generation: Option<i64>,
    pub id: Option<String>,
    pub kind: String,
    pub object: Option<String>,
    pub project_team: Option<ProjectTeam>,
    pub role: ObjectACLRole,
    pub self_link: Option<String>,
}

/// A set of properties to return in a response.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Deserialize, serde::Serialize, Debug)]
pub enum ObjectACLRole {
    READER,
    OWNER,
}

impl Default for ObjectACLRole {
    fn default() -> Self {
        ObjectACLRole::READER
    }
}

/// Predefined or "canned" aliases for sets of specific object ACL entries.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Deserialize, serde::Serialize, Debug)]
pub enum PredefinedObjectAcl {
    /// Object owner gets `OWNER` access, and
    /// `allAuthenticatedUsers` get `READER` access.
    #[serde(rename="authenticatedRead")]
    ObjectAclAuthenticatedRead = 1,
    /// Object owner gets `OWNER` access, and project team owners get
    /// `OWNER` access.
    #[serde(rename="bucketOwnerFullControl")]
    ObjectAclBucketOwnerFullControl = 2,
    /// Object owner gets `OWNER` access, and project team owners get
    /// `READER` access.
    #[serde(rename="bucketOwnerRead")]
    ObjectAclBucketOwnerRead = 3,
    /// Object owner gets `OWNER` access.
    #[serde(rename="private")]
    ObjectAclPrivate = 4,
    /// Object owner gets `OWNER` access, and project team members get
    /// access according to their roles.
    #[serde(rename="projectPrivate")]
    ObjectAclProjectPrivate = 5,
    /// Object owner gets `OWNER` access, and `allUsers`
    /// get `READER` access.
    #[serde(rename="publicRead")]
    ObjectAclPublicRead = 6,
}

/// Represents the Viewers, Editors, or Owners of a given project.
#[derive(Clone, PartialEq, Default, serde::Deserialize, serde::Serialize, Debug)]
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
#[repr(i32)]
pub enum Projection {
    /// Omit `owner`, `acl`, and `defaultObjectAcl` properties.
    NoAcl = 1,
    /// Include all properties.
    Full = 2,
}