use crate::http::entity::bucket::{Versioning, Website};
use crate::http::entity::common_enums::{PredefinedBucketAcl, PredefinedObjectAcl, Projection};
use crate::http::entity::{
    Bucket, BucketAccessControl, DeleteBucketRequest, InsertBucketRequest, ObjectAccessControl,
};
use crate::http::partial::BucketCreationConfig;
use crate::http::storage_client::{Error, StorageClient};
use crate::sign::{signed_url, SignBy, SignedURLError, SignedURLOptions};
use chrono::{DateTime, SecondsFormat, Timelike, Utc};
use google_cloud_auth::credentials::CredentialsFile;
use google_cloud_gax::cancel::CancellationToken;
use google_cloud_gax::grpc::codegen::Body;

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

    pub async fn create(&self, attr: &BucketCreationConfig, cancel: Option<CancellationToken>) -> Result<Bucket, Error> {
        let mut bucket: Bucket = attr.into();
        bucket.name = self.name.to_string();
        let req = InsertBucketRequest {
            predefined_acl: attr.predefined_acl,
            predefined_default_object_acl: attr.predefined_default_object_acl,
            projection: None,
            project: self.project_id.to_string(),
            bucket,
        };
        self.storage_client.insert_bucket(req, cancel).await
    }

}
