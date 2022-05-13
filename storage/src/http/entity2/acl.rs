use crate::http::entity2::common::MetadataGenerationMatch;
use crate::http::entity2::StringParam;

pub struct Generation(i64);

impl Generation {
    pub fn to_param(&self) -> StringParam  {
        StringParam("generation", v.0.to_string())
    }
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BucketAccessControlsCreationConfig {
    pub entity: String,
    pub role: BucketACLRole,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertBucketAccessControlsRequest {
    pub bucket: String,
    pub acl: BucketAccessControlsCreationConfig,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetBucketAccessControlsRequest {
    pub bucket: String,
    pub entity: String,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteBucketAccessControlsRequest {
    pub bucket: String,
    pub entity: String,
}

/// An access-control entry.
#[derive(Clone, PartialEq, Default, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BucketAccessControl {
    pub bucket: String,
    pub domain: Option<String>,
    pub email: Option<String>,
    pub entity: String,
    pub entity_id: Option<String>,
    pub etag: String,
    pub id: Option<String>,
    pub kind: String,
    pub project_team: Option<ProjectTeam>,
    pub role: String,
    pub self_link: String,
}

/// Request message for InsertBucketAccessControl.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertBucketAccessControlRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// Properties of the new bucket access control being inserted.
    pub bucket_access_control: Option<BucketAccessControl>,
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ObjectAccessControlsCreationConfig {
    pub entity: String,
    pub role: String,
}

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
    pub role: String,
    pub self_link: Option<String>,
}

/// Request message for DeleteObjectAccessControl.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteObjectAccessControlRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// Required. The entity holding the permission. Can be one of:
    /// * `user-`*userId*
    /// * `user-`*emailAddress*
    /// * `group-`*groupId*
    /// * `group-`*emailAddress*
    /// * `allUsers`
    /// * `allAuthenticatedUsers`
    pub entity: String,
    /// Required. Name of the object.
    pub object: String,
    /// If present, selects a specific revision of this object (as opposed to the
    /// latest version, the default).
    pub generation: i64,
}
/// Request message for GetObjectAccessControl.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetObjectAccessControlRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// Required. The entity holding the permission. Can be one of:
    /// * `user-`*userId*
    /// * `user-`*emailAddress*
    /// * `group-`*groupId*
    /// * `group-`*emailAddress*
    /// * `allUsers`
    /// * `allAuthenticatedUsers`
    pub entity: String,
    /// Required. Name of the object.
    pub object: String,
    /// If present, selects a specific revision of this object (as opposed to the
    /// latest version, the default).
    pub generation: i64,
}
/// Request message for InsertObjectAccessControl.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InsertObjectAccessControlRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// Required. Name of the object.
    pub object: String,
    /// If present, selects a specific revision of this object (as opposed to the
    /// latest version, the default).
    pub generation: Option<Generation>,
    /// Properties of the object access control to be inserted.
    pub object_access_control: ObjectAccessControlsCreationConfig,
}
/// Request message for ListObjectAccessControls.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListObjectAccessControlsRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// Required. Name of the object.
    pub object: String,
    /// If present, selects a specific revision of this object (as opposed to the
    /// latest version, the default).
    pub generation: i64,
}
/// Request message for PatchObjectAccessControl.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PatchObjectAccessControlRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// Required. The entity holding the permission. Can be one of:
    /// * `user-`*userId*
    /// * `user-`*emailAddress*
    /// * `group-`*groupId*
    /// * `group-`*emailAddress*
    /// * `allUsers`
    /// * `allAuthenticatedUsers`
    pub entity: String,
    /// Required. Name of the object.
    /// Required.
    pub object: String,
    /// If present, selects a specific revision of this object (as opposed to the
    /// latest version, the default).
    pub generation: i64,
    /// The ObjectAccessControl for updating.
    pub object_access_control: Option<ObjectAccessControl>,

    /// List of fields to be updated.
    ///
    /// To specify ALL fields, equivalent to the JSON API's "update" function,
    /// specify a single field with the value `*`. Note: not recommended. If a new
    /// field is introduced at a later time, an older client updating with the `*`
    /// may accidentally reset the new field's value.
    ///
    /// Not specifying any fields is an error.
    /// Not specifying a field while setting that field to a non-default value is
    /// an error.
    pub update_mask: Option<()>, //TODO
}

/// The result of a call to ObjectAccessControls.ListObjectAccessControls.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListObjectAccessControlsResponse {
    /// The list of items.
    pub items: Vec<ObjectAccessControl>,
}

/// Request message for DeleteDefaultObjectAccessControl.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteDefaultObjectAccessControlRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// Required. The entity holding the permission. Can be one of:
    /// * `user-`*userId*
    /// * `user-`*emailAddress*
    /// * `group-`*groupId*
    /// * `group-`*emailAddress*
    /// * `allUsers`
    /// * `allAuthenticatedUsers`
    pub entity: String,
}

/// Request message for ListDefaultObjectAccessControls.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListDefaultObjectAccessControlsRequest {
    /// Required. Name of a bucket.
    pub bucket: String,
    /// Metageneration matches this value.
    pub metageneration: MetadataGenerationMatch,
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
pub enum BucketACLRole {
    OWNER,
    READER,
    WRITER,
}

/// A set of properties to return in a response.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Deserialize, serde::Serialize, Debug)]
pub enum ObjectACLRole {
    OWNER,
    READER,
}
