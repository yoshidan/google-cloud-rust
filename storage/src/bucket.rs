use crate::bucket;
use crate::http::entity::bucket::{Versioning, Website};
use crate::http::entity::common_enums::{PredefinedBucketAcl, PredefinedObjectAcl, Projection};
use crate::http::entity::{
    Bucket, BucketAccessControl, BucketCreationConfig, DeleteBucketRequest, GetBucketRequest, InsertBucketRequest,
    ListBucketsRequest, ObjectAccessControl, ObjectAccessControlsCreationConfig, PatchBucketRequest,
    RetentionPolicyCreationConfig,
};
use crate::http::storage_client::{Error, StorageClient};
use crate::iam::IAMHandle;
use crate::sign::{signed_url, SignBy, SignedURLError, SignedURLOptions};
use chrono::{DateTime, SecondsFormat, Timelike, Utc};
use google_cloud_auth::credentials::CredentialsFile;
use std::collections::HashMap;
use tokio_util::sync::CancellationToken;

pub struct BucketHandle<'a> {
    name: String,
    private_key: &'a str,
    service_account_email: &'a str,
    project_id: &'a str,
    storage_client: StorageClient,
}

impl<'a> BucketHandle<'a> {
    pub(crate) fn new(
        name: String,
        private_key: &'a str,
        service_account_email: &'a str,
        project_id: &'a str,
        storage_client: StorageClient,
    ) -> Self {
        Self {
            name,
            private_key,
            service_account_email,
            project_id,
            storage_client,
        }
    }

    pub async fn signed_url(&self, object: String, opts: &mut SignedURLOptions) -> Result<String, SignedURLError> {
        let signable = match &opts.sign_by {
            SignBy::PrivateKey(v) => !v.is_empty(),
            _ => true,
        };
        if !opts.google_access_id.is_empty() && signable {
            return signed_url(self.name.to_string(), object, opts);
        }

        if !self.private_key.is_empty() {
            opts.sign_by = SignBy::PrivateKey(self.private_key.into());
        }
        if !self.service_account_email.is_empty() && opts.google_access_id.is_empty() {
            opts.google_access_id = self.service_account_email.to_string();
        }
        return signed_url(self.name.to_string(), object, opts);
    }

    pub async fn delete(&self, cancel: Option<CancellationToken>) -> Result<(), Error> {
        let req = DeleteBucketRequest {
            bucket: self.name.to_string(),
            ..Default::default()
        };
        self.storage_client.delete_bucket(req, cancel).await
    }

    pub async fn insert(
        &self,
        req: &mut InsertBucketRequest,
        cancel: Option<CancellationToken>,
    ) -> Result<Bucket, Error> {
        req.bucket.name = self.name.to_string();
        self.storage_client.insert_bucket(self.project_id, req, cancel).await
    }

    pub async fn get(&self, cancel: Option<CancellationToken>) -> Result<Bucket, Error> {
        let req = GetBucketRequest {
            bucket: self.name.to_string(),
            ..Default::default()
        };
        self.storage_client.get_bucket(&req, cancel).await
    }

    pub async fn patch(&self, req: &PatchBucketRequest, cancel: Option<CancellationToken>) -> Result<Bucket, Error> {
        self.storage_client
            .patch_bucket(self.name.as_str(), self.project_id, &req, cancel)
            .await
    }

    pub fn iam(&self) -> IAMHandle {
        IAMHandle::new(self.name.as_str(), &self.storage_client)
    }
}
