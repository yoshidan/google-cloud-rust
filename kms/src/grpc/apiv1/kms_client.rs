use std::time::Duration;

use google_cloud_gax::conn::Channel;
use google_cloud_gax::create_request;
use google_cloud_gax::grpc::{Code, Status};
use google_cloud_gax::retry::{invoke_fn, RetrySetting};
use google_cloud_googleapis::cloud::kms::v1::key_management_service_client::KeyManagementServiceClient;
use google_cloud_googleapis::cloud::kms::v1::CreateCryptoKeyRequest;
use google_cloud_googleapis::cloud::kms::v1::CreateCryptoKeyVersionRequest;
use google_cloud_googleapis::cloud::kms::v1::CreateKeyRingRequest;
use google_cloud_googleapis::cloud::kms::v1::CryptoKey;
use google_cloud_googleapis::cloud::kms::v1::CryptoKeyVersion;
use google_cloud_googleapis::cloud::kms::v1::DestroyCryptoKeyVersionRequest;
use google_cloud_googleapis::cloud::kms::v1::GenerateRandomBytesRequest;
use google_cloud_googleapis::cloud::kms::v1::GenerateRandomBytesResponse;
use google_cloud_googleapis::cloud::kms::v1::GetCryptoKeyRequest;
use google_cloud_googleapis::cloud::kms::v1::GetCryptoKeyVersionRequest;
use google_cloud_googleapis::cloud::kms::v1::GetKeyRingRequest;
use google_cloud_googleapis::cloud::kms::v1::KeyRing;
use google_cloud_googleapis::cloud::kms::v1::ListCryptoKeyVersionsRequest;
use google_cloud_googleapis::cloud::kms::v1::ListCryptoKeyVersionsResponse;
use google_cloud_googleapis::cloud::kms::v1::ListCryptoKeysRequest;
use google_cloud_googleapis::cloud::kms::v1::ListCryptoKeysResponse;
use google_cloud_googleapis::cloud::kms::v1::ListKeyRingsRequest;
use google_cloud_googleapis::cloud::kms::v1::ListKeyRingsResponse;

fn default_setting() -> RetrySetting {
    RetrySetting {
        from_millis: 50,
        max_delay: Some(Duration::from_secs(60)),
        factor: 1u64,
        take: 20,
        codes: vec![Code::Unavailable, Code::Unknown],
    }
}

#[derive(Clone)]
pub struct Client {
    inner: KeyManagementServiceClient<Channel>,
}

impl Client {
    pub fn new(inner: KeyManagementServiceClient<Channel>) -> Self {
        Self {
            inner: inner.max_decoding_message_size(i32::MAX as usize),
        }
    }

    /// Generate random bytes
    ///
    /// https://cloud.google.com/kms/docs/reference/rpc/google.cloud.kms.v1#google.cloud.kms.v1.KeyManagementService.GenerateRandomBytes
    ///
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn generate_random_bytes(
        &mut self,
        req: GenerateRandomBytesRequest,
        retry: Option<RetrySetting>,
    ) -> Result<GenerateRandomBytesResponse, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("location={}", req.location), req.clone());
                client
                    .generate_random_bytes(request)
                    .await
                    .map(|s| s.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Create crypto key
    ///
    /// https://cloud.google.com/kms/docs/reference/rpc/google.cloud.kms.v1#google.cloud.kms.v1.KeyManagementService.CreateCryptoKey
    ///
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn create_crypto_key(
        &mut self,
        req: CreateCryptoKeyRequest,
        retry: Option<RetrySetting>,
    ) -> Result<CryptoKey, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("parent={}", req.parent), req.clone());
                client
                    .create_crypto_key(request)
                    .await
                    .map(|s| s.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Create crypto key version
    ///
    /// https://cloud.google.com/kms/docs/reference/rpc/google.cloud.kms.v1#google.cloud.kms.v1.KeyManagementService.CreateCryptoKeyVersion
    ///
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn create_crypto_key_version(
        &mut self,
        req: CreateCryptoKeyVersionRequest,
        retry: Option<RetrySetting>,
    ) -> Result<CryptoKeyVersion, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("parent={}", req.parent), req.clone());
                client
                    .create_crypto_key_version(request)
                    .await
                    .map(|s| s.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Create key ring
    ///
    /// https://cloud.google.com/kms/docs/reference/rpc/google.cloud.kms.v1#google.cloud.kms.v1.KeyManagementService.CreateKeyRing
    ///
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn create_key_ring(
        &mut self,
        req: CreateKeyRingRequest,
        retry: Option<RetrySetting>,
    ) -> Result<KeyRing, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("parent={}", req.parent), req.clone());
                client
                    .create_key_ring(request)
                    .await
                    .map(|s| s.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Destroy crypto key version
    ///
    /// https://cloud.google.com/kms/docs/reference/rpc/google.cloud.kms.v1#google.cloud.kms.v1.KeyManagementService.DestroyCryptoKeyVersion
    ///
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn destroy_crypto_key_version(
        &mut self,
        req: DestroyCryptoKeyVersionRequest,
        retry: Option<RetrySetting>,
    ) -> Result<CryptoKeyVersion, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("name={}", req.name), req.clone());
                client
                    .destroy_crypto_key_version(request)
                    .await
                    .map(|s| s.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Get crypto key
    ///
    /// https://cloud.google.com/kms/docs/reference/rpc/google.cloud.kms.v1#google.cloud.kms.v1.KeyManagementService.GetCryptoKey
    ///
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_crypto_key(
        &mut self,
        req: GetCryptoKeyRequest,
        retry: Option<RetrySetting>,
    ) -> Result<CryptoKey, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("name={}", req.name), req.clone());
                client
                    .get_crypto_key(request)
                    .await
                    .map(|s| s.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Get crypto key version
    ///
    /// https://cloud.google.com/kms/docs/reference/rpc/google.cloud.kms.v1#google.cloud.kms.v1.KeyManagementService.GetCryptoKeyVersion
    ///
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_crypto_key_version(
        &mut self,
        req: GetCryptoKeyVersionRequest,
        retry: Option<RetrySetting>,
    ) -> Result<CryptoKeyVersion, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("name={}", req.name), req.clone());
                client
                    .get_crypto_key_version(request)
                    .await
                    .map(|s| s.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// Get key ring
    ///
    /// https://cloud.google.com/kms/docs/reference/rpc/google.cloud.kms.v1#google.cloud.kms.v1.KeyManagementService.GetKeyRing
    ///
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn get_key_ring(
        &mut self,
        req: GetKeyRingRequest,
        retry: Option<RetrySetting>,
    ) -> Result<KeyRing, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("name={}", req.name), req.clone());
                client
                    .get_key_ring(request)
                    .await
                    .map(|s| s.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// List crypto key versions
    ///
    /// https://cloud.google.com/kms/docs/reference/rpc/google.cloud.kms.v1#google.cloud.kms.v1.KeyManagementService.ListCryptoKeyVersions
    ///
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn list_crypto_key_versions(
        &mut self,
        req: ListCryptoKeyVersionsRequest,
        retry: Option<RetrySetting>,
    ) -> Result<ListCryptoKeyVersionsResponse, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("parent={}", req.parent), req.clone());
                client
                    .list_crypto_key_versions(request)
                    .await
                    .map(|s| s.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// List crypto keys
    ///
    /// https://cloud.google.com/kms/docs/reference/rpc/google.cloud.kms.v1#google.cloud.kms.v1.KeyManagementService.ListCryptoKeys
    ///
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn list_crypto_keys(
        &mut self,
        req: ListCryptoKeysRequest,
        retry: Option<RetrySetting>,
    ) -> Result<ListCryptoKeysResponse, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("parent={}", req.parent), req.clone());
                client
                    .list_crypto_keys(request)
                    .await
                    .map(|s| s.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }

    /// List key rings
    ///
    /// https://cloud.google.com/kms/docs/reference/rpc/google.cloud.kms.v1#google.cloud.kms.v1.KeyManagementService.ListKeyRings
    ///
    #[cfg_attr(feature = "trace", tracing::instrument(skip_all))]
    pub async fn list_key_rings(
        &mut self,
        req: ListKeyRingsRequest,
        retry: Option<RetrySetting>,
    ) -> Result<ListKeyRingsResponse, Status> {
        let setting = retry.unwrap_or_else(default_setting);

        invoke_fn(
            Some(setting),
            |client| async {
                let request = create_request(format!("parent={}", req.parent), req.clone());
                client
                    .list_key_rings(request)
                    .await
                    .map(|s| s.into_inner())
                    .map_err(|e| (e, client))
            },
            &mut self.inner,
        )
        .await
    }
}
