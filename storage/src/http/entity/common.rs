use crate::http::entity::StringParam;

impl MetadataGenerationMatch {
    pub fn to_param(&self) -> Vec<StringParam>{
        let mut v = vec![];
        if let Some(v) = self.if_metageneration_match {
            v.push(StringParam("ifMetagenerationMatch", v.to_string()));
        }
        if let Some(v) = self.if_metageneration_not_match {
            v.push(StringParam("ifMetagenerationNotMatch", v.to_string()));
        }
        v
    }
    pub fn to_source_param(&self) -> Vec<StringParam>{
        let mut v = vec![];
        if let Some(v) = self.if_metageneration_match {
            v.push(StringParam("ifSourceMetagenerationMatch", v.to_string()));
        }
        if let Some(v) = self.if_metageneration_not_match {
            v.push(StringParam("ifSourceMetagenerationNotMatch", v.to_string()));
        }
        v
    }
}


/// Generation match parameter.
#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize, Default)]
pub struct GenerationMatch {
    /// If set, only deletes the bucket if its generation matches this value.
    pub if_generation_match: Option<i64>,
    /// If set, only deletes the bucket if its generation does not match this
    /// value.
    pub if_generation_not_match: Option<i64>,
}

impl GenerationMatch {
    pub fn to_param(&self) -> Vec<StringParam>{
        let mut v = vec![];
        if let Some(v) = self.if_generation_match {
            v.push(StringParam("ifGenerationMatch", v.to_string()));
        }
        if let Some(v) = self.if_generation_not_match {
            v.push(StringParam("ifGenerationNotMatch", v.to_string()));
        }
        v
    }
    pub fn to_source_param(&self) -> Vec<StringParam>{
        let mut v = vec![];
        if let Some(v) = self.if_generation_match {
            v.push(StringParam("ifSourceGenerationMatch", v.to_string()));
        }
        if let Some(v) = self.if_generation_not_match {
            v.push(StringParam("ifSourceGenerationNotMatch", v.to_string()));
        }
        v
    }
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

/// Predefined or "canned" aliases for sets of specific bucket ACL entries.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Deserialize, serde::Serialize, Debug)]
#[repr(i32)]
pub enum PredefinedBucketAcl {
    /// Project team owners get `OWNER` access, and
    /// `allAuthenticatedUsers` get `READER` access.
    BucketAclAuthenticatedRead = 1,
    /// Project team owners get `OWNER` access.
    BucketAclPrivate = 2,
    /// Project team members get access according to their roles.
    BucketAclProjectPrivate = 3,
    /// Project team owners get `OWNER` access, and
    /// `allUsers` get `READER` access.
    BucketAclPublicRead = 4,
    /// Project team owners get `OWNER` access, and
    /// `allUsers` get `WRITER` access.
    BucketAclPublicReadWrite = 5,
}

/// Predefined or "canned" aliases for sets of specific object ACL entries.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Deserialize, serde::Serialize, Debug)]
#[repr(i32)]
pub enum PredefinedObjectAcl {
    /// Object owner gets `OWNER` access, and
    /// `allAuthenticatedUsers` get `READER` access.
    ObjectAclAuthenticatedRead = 1,
    /// Object owner gets `OWNER` access, and project team owners get
    /// `OWNER` access.
    ObjectAclBucketOwnerFullControl = 2,
    /// Object owner gets `OWNER` access, and project team owners get
    /// `READER` access.
    ObjectAclBucketOwnerRead = 3,
    /// Object owner gets `OWNER` access.
    ObjectAclPrivate = 4,
    /// Object owner gets `OWNER` access, and project team members get
    /// access according to their roles.
    ObjectAclProjectPrivate = 5,
    /// Object owner gets `OWNER` access, and `allUsers`
    /// get `READER` access.
    ObjectAclPublicRead = 6,
}

impl Projection  {
    pub fn as_param(&self) -> (&'static str, &'static str) {
        ("projection", v.into())
    }
}

impl Projection {
    fn as_str(&self) -> &'static str {
        match v {
            Projection::NoAcl => "noAcl",
            Projection::Full => "full",
        }
    }
}

impl PredefinedBucketAcl  {
    pub fn as_param(&self) -> (&'static str, &'static str) {
        ("predefinedBucketAcl", self.as_str())
    }
}

impl PredefinedBucketAcl {
    fn as_str(&self) -> &'static str {
        match v {
            PredefinedBucketAcl::BucketAclAuthenticatedRead => "authenticatedRead",
            PredefinedBucketAcl::BucketAclPrivate => "private",
            PredefinedBucketAcl::BucketAclProjectPrivate => "projectPrivate",
            PredefinedBucketAcl::BucketAclPublicRead => "publicRead",
            PredefinedBucketAcl::BucketAclPublicReadWrite => "publicReadWrite",
        }
    }
}


impl PredefinedObjectAcl {
    pub fn as_default_object_acl(&self) ->  (&'static str, &'static str) {
        ("predefinedObjectAcl", self.as_str())
    }
    pub fn as_param(&self) ->  (&'static str, &'static str) {
        ("predefinedAcl", self.as_str())
    }
}

impl PredefinedObjectAcl {
    pub fn as_str(&self) -> &'static str {
        match v {
            PredefinedObjectAcl::ObjectAclAuthenticatedRead => "authenticatedRead",
            PredefinedObjectAcl::ObjectAclBucketOwnerFullControl => "bucketOwnerFullControl",
            PredefinedObjectAcl::ObjectAclBucketOwnerRead => "bucketOwnerRead",
            PredefinedObjectAcl::ObjectAclPrivate => "private",
            PredefinedObjectAcl::ObjectAclProjectPrivate => "projectPrivate",
            PredefinedObjectAcl::ObjectAclPublicRead => "publicRead",
        }
    }
}
