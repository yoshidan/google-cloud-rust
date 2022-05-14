use tokio_util::sync::CancellationToken;
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

    pub async fn set(&self,  role: ACLRole, cancel: Option<CancellationToken>) -> Result<BucketAccessControl, Error> {
        self.storage_client.insert_bucket_acl(self.name, &BucketAccessControlsCreationConfig {
            entity: self.entity.to_string(),
            role
        }, cancel).await
    }

    pub async fn get(&self, cancel: Option<CancellationToken>) -> Result<BucketAccessControl, Error> {
        self.storage_client.get_bucket_acl(self.name, self.entity, cancel).await
    }

    pub async fn delete(&self, cancel: Option<CancellationToken>) -> Result<(), Error> {
        self.storage_client.delete_bucket_acl(self.name, self.entity, cancel).await
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

    pub async fn set(&self,  role: &str, cancel: Option<CancellationToken>) -> Result<ObjectAccessControl, Error> {
        self.storage_client.insert_default_object_acl(self.name, &ObjectAccessControlsCreationConfig{
            entity: self.entity.to_string(),
            role: role.to_string()
        }, cancel).await
    }

    pub async fn get(&self, cancel: Option<CancellationToken>) -> Result<ObjectAccessControl, Error> {
        self.storage_client.get_default_object_acl(self.name, self.entity, cancel).await
    }

    pub async fn delete(&self, cancel: Option<CancellationToken>) -> Result<(), Error> {
        self.storage_client.delete_default_object_acl(self.name, self.entity, cancel).await
    }

}

#[cfg(test)]
mod test {
    use crate::bucket::BucketHandle;
    use crate::client;
    use crate::http::old_entity::bucket::iam_configuration::{PublicAccessPrevention, UniformBucketLevelAccess};
    use crate::http::old_entity::bucket::lifecycle::rule::{Action, ActionType, Condition};
    use crate::http::old_entity::bucket::lifecycle::Rule;
    use crate::http::old_entity::bucket::{
        Billing, Cors, Encryption, IamConfiguration, Lifecycle, Logging, RetentionPolicy, Versioning, Website,
    };
    use crate::http::old_entity::common_enums::PredefinedBucketAcl;
    use crate::http::old_entity::{ACLRole, Bucket, BucketAccessControl, BucketCreationConfig, BucketPatchConfig, InsertBucketRequest, ObjectAccessControl, ObjectAccessControlsCreationConfig, PatchBucketRequest, RetentionPolicyCreationConfig};
    use crate::http::CancellationToken;
    use chrono::{DateTime, Utc};
    use google_cloud_auth::credentials::CredentialsFile;
    use serde_json;
    use serial_test::serial;
    use std::collections::HashMap;
    use std::time;
    use std::time::Duration;
    use tokio::sync::OnceCell;
    use tracing::{info, Level};
    use crate::client::Client;
    use crate::http::iam::Binding;

    #[ctor::ctor]
    fn init() {
        let _ = tracing_subscriber::fmt::try_init();
    }

    #[tokio::test]
    #[serial]
    async fn set_bucket_acl() {
        let client = Client::new().await.unwrap();
        let bucket = client.bucket("rust-bucket-acl-test");
        // Access Control must be Fine Grained
        bucket.patch(&PatchBucketRequest {
            metadata: Some(BucketPatchConfig {
                iam_configuration: Some(IamConfiguration {
                    uniform_bucket_level_access: Some(UniformBucketLevelAccess {
                        enabled: false,
                        locked_time: None,
                    }),
                    public_access_prevention: None,
                }),
                ..Default::default()
            }),
            ..Default::default()
        }, None).await;
        let acl = bucket.acl("allAuthenticatedUsers");
        let policy = acl.set(ACLRole::READER, None).await.unwrap();
        info!("{:?}", serde_json::to_string(&policy));
        assert_eq!(policy.role, "READER");
        assert_eq!(policy.entity, "allAuthenticatedUsers");
    }

    #[tokio::test]
    #[serial]
    async fn get_bucket_acl() {
        let client = Client::new().await.unwrap();
        let bucket = client.bucket("rust-bucket-acl-test");
        let acl = bucket.acl("allAuthenticatedUsers");
        let policy = acl.get(None).await.unwrap();
        info!("{:?}", serde_json::to_string(&policy));
        assert_eq!(policy.role, "READER");
        assert_eq!(policy.entity, "allAuthenticatedUsers");
    }

    #[tokio::test]
    #[serial]
    async fn delete_bucket_acl() {
        let client = Client::new().await.unwrap();
        let bucket = client.bucket("rust-bucket-acl-test");
        let acl = bucket.acl("allAuthenticatedUsers");
        let _ = acl.delete(None).await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn set_default_object_acl() {
        let client = Client::new().await.unwrap();
        let bucket = client.bucket("rust-default-object-acl-test");
        // Access Control must be Fine Grained
        bucket.patch(&PatchBucketRequest {
            metadata: Some(BucketPatchConfig {
                iam_configuration: Some(IamConfiguration {
                    uniform_bucket_level_access: Some(UniformBucketLevelAccess {
                        enabled: false,
                        locked_time: None,
                    }),
                    public_access_prevention: None,
                }),
                ..Default::default()
            }),
            ..Default::default()
        }, None).await;
        let acl = bucket.default_object_acl("allAuthenticatedUsers");
        let policy = acl.set("READER", None).await.unwrap();
        info!("{:?}", serde_json::to_string(&policy));
        assert_eq!(policy.role, "READER");
        assert_eq!(policy.entity, "allAuthenticatedUsers");
    }

    #[tokio::test]
    #[serial]
    async fn get_default_object_acl() {
        let client = Client::new().await.unwrap();
        let bucket = client.bucket("rust-default-object-acl-test");
        let acl = bucket.default_object_acl("allAuthenticatedUsers");
        let policy = acl.get(None).await.unwrap();
        info!("{:?}", serde_json::to_string(&policy));
        assert_eq!(policy.role, "READER");
        assert_eq!(policy.entity, "allAuthenticatedUsers");
    }

    #[tokio::test]
    #[serial]
    async fn delete_default_object_acl() {
        let client = Client::new().await.unwrap();
        let bucket = client.bucket("rust-bucket-acl-test");
        let acl = bucket.default_object_acl("allAuthenticatedUsers");
        let _ = acl.delete(None).await.unwrap();
    }
}
