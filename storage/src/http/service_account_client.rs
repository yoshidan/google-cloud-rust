use crate::http::bucket_access_controls::delete::DeleteBucketAccessControlRequest;
use crate::http::bucket_access_controls::get::GetBucketAccessControlRequest;
use crate::http::bucket_access_controls::insert::InsertBucketAccessControlRequest;
use crate::http::bucket_access_controls::list::{ListBucketAccessControlsRequest, ListBucketAccessControlsResponse};
use crate::http::bucket_access_controls::patch::PatchBucketAccessControlRequest;
use crate::http::bucket_access_controls::BucketAccessControl;
use crate::http::buckets::delete::DeleteBucketRequest;
use crate::http::buckets::get::GetBucketRequest;
use crate::http::buckets::get_iam_policy::GetIamPolicyRequest;
use crate::http::buckets::insert::InsertBucketRequest;
use crate::http::buckets::list::{ListBucketsRequest, ListBucketsResponse};
use std::collections::HashMap;

use crate::http::buckets::patch::PatchBucketRequest;
use crate::http::buckets::set_iam_policy::SetIamPolicyRequest;
use crate::http::buckets::test_iam_permissions::{TestIamPermissionsRequest, TestIamPermissionsResponse};
use crate::http::buckets::{Bucket, Policy};

use crate::http::default_object_access_controls::delete::DeleteDefaultObjectAccessControlRequest;
use crate::http::default_object_access_controls::get::GetDefaultObjectAccessControlRequest;
use crate::http::default_object_access_controls::insert::InsertDefaultObjectAccessControlRequest;
use crate::http::default_object_access_controls::list::{
    ListDefaultObjectAccessControlsRequest, ListDefaultObjectAccessControlsResponse,
};
use crate::http::default_object_access_controls::patch::PatchDefaultObjectAccessControlRequest;
use crate::http::hmac_keys::create::{CreateHmacKeyRequest, CreateHmacKeyResponse};
use crate::http::hmac_keys::delete::DeleteHmacKeyRequest;
use crate::http::hmac_keys::get::GetHmacKeyRequest;
use crate::http::hmac_keys::list::{ListHmacKeysRequest, ListHmacKeysResponse};
use crate::http::hmac_keys::update::UpdateHmacKeyRequest;
use crate::http::hmac_keys::HmacKeyMetadata;
use crate::http::notifications::delete::DeleteNotificationRequest;
use crate::http::notifications::get::GetNotificationRequest;
use crate::http::notifications::insert::InsertNotificationRequest;
use crate::http::notifications::list::{ListNotificationsRequest, ListNotificationsResponse};
use crate::http::notifications::Notification;
use crate::http::object_access_controls::delete::DeleteObjectAccessControlRequest;
use crate::http::object_access_controls::get::GetObjectAccessControlRequest;
use crate::http::object_access_controls::insert::InsertObjectAccessControlRequest;
use crate::http::object_access_controls::list::ListObjectAccessControlsRequest;
use crate::http::object_access_controls::patch::PatchObjectAccessControlRequest;
use crate::http::object_access_controls::ObjectAccessControl;
use crate::http::objects::compose::ComposeObjectRequest;
use crate::http::objects::delete::DeleteObjectRequest;
use crate::http::objects::get::GetObjectRequest;
use crate::http::objects::list::{ListObjectsRequest, ListObjectsResponse};
use crate::http::objects::patch::PatchObjectRequest;
use crate::http::objects::rewrite::{RewriteObjectRequest, RewriteObjectResponse};
use crate::http::objects::upload::UploadObjectRequest;

use crate::http::objects::Object;
use crate::http::{
    bucket_access_controls, buckets, default_object_access_controls, hmac_keys, notifications, object_access_controls,
    objects, CancellationToken, Error,
};
use futures_util::{Stream, TryStream};
use google_cloud_auth::token_source::TokenSource;

use reqwest::{Body, Client, RequestBuilder, Response};

use std::future::Future;

use crate::http::objects::download::Range;
use std::sync::Arc;

pub struct ServiceAccountClient {
    ts: Arc<dyn TokenSource>,
    v1_endpoint: String,
}

impl ServiceAccountClient {
    pub(crate) fn new(ts: Arc<dyn TokenSource>, endpoint: &str) -> Self {
        Self {
            ts,
            v1_endpoint: format!("{}/v1", endpoint),
        }
    }

    pub(crate) async fn sign_blob(&self, name: &str, data: &[u8]) -> Result<Vec<u8>, Error> {
        let url = format!("{}/{}:signBlob", self.v1_endpoint, name);
        let payload = ("payload", base64::encode(data));
        let request = Client::default().post(url).json(&payload);
        let response = request.send().await?;
        let status = response.status();
        if status.is_success() {
            let body = response.json::<HashMap<String, String>>().await?;
            match body.get("signedBlob") {
                Some(v) => Ok(base64::decode(v)?),
                None => Err(Error::Response(status.as_u16(), "no signedBlob found".to_string())),
            }
        } else {
            Err(Error::Response(status.as_u16(), response.text().await?))
        }
    }
}
