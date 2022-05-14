/// The owner of a specific resource.
#[derive(Clone, PartialEq, Default, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Owner {
    /// The entity, in the form `user-`*userId*.
    #[serde(default)]
    pub entity: String,
    /// The ID for the entity.
    pub entity_id: Option<String>,
}
