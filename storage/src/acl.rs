use tokio_util::sync::CancellationToken;
use crate::http::bucket_access_controls::BucketAccessControl;
use crate::http::bucket_access_controls::delete::DeleteBucketAccessControlRequest;
use crate::http::bucket_access_controls::get::GetBucketAccessControlRequest;
use crate::http::bucket_access_controls::insert::InsertBucketAccessControlRequest;
use crate::http::bucket_access_controls::patch::PatchBucketAccessControlRequest;
use crate::http::default_object_access_controls::delete::DeleteDefaultObjectAccessControlRequest;
use crate::http::default_object_access_controls::get::GetDefaultObjectAccessControlRequest;
use crate::http::default_object_access_controls::insert::InsertDefaultObjectAccessControlRequest;
use crate::http::object_access_controls::ObjectAccessControl;
use crate::http::old_entity::{ACLRole, BucketAccessControl, BucketAccessControlsCreationConfig, ObjectAccessControl, ObjectAccessControlsCreationConfig};
use crate::http::storage_client::{Error, StorageClient};

pub struct BucketACLHandle<'a,'b> {
    name: &'a str,
    storage_client: &'a StorageClient,
    entity: &'b str
}

impl <'a,'b> BucketACLHandle<'a,'b> {
    pub(crate) fn new(name: &'a str, entity: &'b str, storage_client: &'a StorageClient) -> Self {
        Self { name,storage_client, entity}
    }

    /// Sets the bucket acl
    pub async fn set(&self, role: impl Into<ACLRole>, cancel: Option<CancellationToken>) -> Result<BucketAccessControl, Error> {
        self.storage_client.insert_bucket_access_control(&InsertBucketAccessControlRequest {
            bucket: self.name.to_string(),
            acl: BucketAccessControlsCreationConfig {
                entity: self.entity.to_string(),
                role: role.into()
            }
        }, cancel).await
    }

    /// Gets the bucket acl
    pub async fn get(&self, cancel: Option<CancellationToken>) -> Result<BucketAccessControl, Error> {
        self.storage_client.get_bucket_access_control(&GetBucketAccessControlRequest {
            bucket: self.name.to_string(),
            entity: self.entity.to_string(),
        }, cancel).await
    }

    /// Deletes the bucket acl
    pub async fn delete(&self, cancel: Option<CancellationToken>) -> Result<(), Error> {
        self.storage_client.delete_bucket_acl(&DeleteBucketAccessControlRequest {
            bucket: self.name.to_string(),
            entity: self.entity.to_string(),
        }, cancel).await
    }

    /// Deletes the bucket acl
    pub async fn patch(&self, acl: impl Into<BucketAccessControl>, cancel: Option<CancellationToken>) -> Result<BucketAccessControl, Error> {
        self.storage_client.patch_bucket_access_control(&PatchBucketAccessControlRequest {
            bucket: self.name.to_string(),
            entity: self.entity.to_string(),
            acl: acl.into(),
        }, cancel).await
    }
}

pub struct DefaultObjectACLHandle<'a,'b> {
    name: &'a str,
    storage_client: &'a StorageClient,
    entity: &'b str
}

impl <'a,'b> DefaultObjectACLHandle<'a,'b> {
    pub(crate) fn new(name: &'a str, entity: &'b str, storage_client: &'a StorageClient) -> Self {
        Self { name,storage_client, entity}
    }

    /// Sets the default object acl
    pub async fn set(&self, role: &str, cancel: Option<CancellationToken>) -> Result<ObjectAccessControl, Error> {
        self.storage_client.insert_default_object_access_control( &InsertDefaultObjectAccessControlRequest {
            bucket: self.name.to_string(),
            object_access_control: ObjectAccessControlsCreationConfig {
                entity: self.entity.to_string(),
                role: role.to_string()
            }
        }, cancel).await
    }

    /// Gets the default object acl
    pub async fn get(&self, cancel: Option<CancellationToken>) -> Result<ObjectAccessControl, Error> {
        self.storage_client.get_default_object_access_control(&GetDefaultObjectAccessControlRequest {
            bucket: self.name.to_string(),
            entity: self.entity.to_string(),
        }.name, cancel).await
    }

    /// Deletes the default object acl
    pub async fn delete(&self, cancel: Option<CancellationToken>) -> Result<(), Error> {
        self.storage_client.delete_default_object_access_control(&DeleteDefaultObjectAccessControlRequest {
            bucket: self.name.to_string(),
            entity: self.entity.to_string(),
        }, cancel).await
    }

}
