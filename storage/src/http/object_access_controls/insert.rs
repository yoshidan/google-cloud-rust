use crate::http::object_access_controls::ObjectACLRole;

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ObjectAccessControlsCreationConfig {
    pub entity: String,
    pub role: ObjectACLRole,
}
