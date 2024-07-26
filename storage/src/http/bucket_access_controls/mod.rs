use crate::http::object_access_controls::ProjectTeam;

pub mod delete;
pub mod get;
pub mod insert;
pub mod list;
pub mod patch;

/// Predefined or "canned" aliases for sets of specific bucket ACL entries.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum PredefinedBucketAcl {
    /// Project team owners get `OWNER` access, and
    /// `allAuthenticatedUsers` get `READER` access.
    AuthenticatedRead,
    /// Project team owners get `OWNER` access.
    Private,
    /// Project team members get access according to their roles.
    ProjectPrivate,
    /// Project team owners get `OWNER` access, and
    /// `allUsers` get `READER` access.
    PublicRead,
    /// Project team owners get `OWNER` access, and
    /// `allUsers` get `WRITER` access.
    PublicReadWrite,
}

/// An access-control entry.
#[derive(Clone, PartialEq, Eq, Default, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BucketAccessControl {
    /// The access permission for the entity.
    pub role: BucketACLRole,
    /// The ID of the access-control entry.
    pub id: String,
    /// The entity holding the permission, in one of the following forms:
    /// * `user-{userid}`
    /// * `user-{email}`
    /// * `group-{groupid}`
    /// * `group-{email}`
    /// * `domain-{domain}`
    /// * `project-{team-projectid}`
    /// * `allUsers`
    /// * `allAuthenticatedUsers`
    ///   Examples:
    /// * The user `liz@example.com` would be `user-liz@example.com`.
    /// * The group `example@googlegroups.com` would be
    ///   `group-example@googlegroups.com`
    /// * All members of the Google Apps for Business domain `example.com` would be
    ///   `domain-example.com`
    pub entity: String,
    /// The ID for the entity, if any.
    pub entity_id: Option<String>,
    /// The email address associated with the entity, if any.
    pub email: Option<String>,
    /// The domain associated with the entity, if any.
    pub domain: Option<String>,
    /// The project team associated with the entity, if any.
    pub project_team: Option<ProjectTeam>,
    /// The link to this access-control entry.
    pub self_link: String,
    /// HTTP 1.1 Entity tag for the access-control entry.
    pub etag: String,
}

/// A set of properties to return in a response.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Deserialize, serde::Serialize, Debug)]
pub enum BucketACLRole {
    OWNER,
    READER,
    WRITER,
}

impl Default for BucketACLRole {
    fn default() -> Self {
        Self::READER
    }
}
