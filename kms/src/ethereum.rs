use crate::grpc::apiv1::kms_client::Client as KmsGrpcClient;
use google_cloud_gax::grpc::Status;
use google_cloud_gax::retry::RetrySetting;
use google_cloud_googleapis::cloud::kms::v1::{digest, AsymmetricSignRequest, Digest, GetPublicKeyRequest};
use k256::ecdsa::{RecoveryId, VerifyingKey};
use k256::elliptic_curve::bigint::{CheckedSub, Encoding};
use k256::elliptic_curve::sec1::ToEncodedPoint;
use k256::elliptic_curve::Curve;
use k256::pkcs8::DecodePublicKey;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    GRPC(#[from] Status),
    #[error(transparent)]
    K256Error(#[from] k256::ecdsa::signature::Error),
    #[error(transparent)]
    SPKIError(#[from] k256::pkcs8::spki::Error),
    #[error("invalid signature")]
    InvalidSignature(Vec<u8>),
}

#[derive(Clone, Debug)]
pub struct Signature {
    /// The output of an ECDSA signature
    pub r: [u8; 32],
    /// The output of an ECDSA signature
    pub s: [u8; 32],
    /// The recovery id to get pubkey. The value is 0 or 1.
    pub v: u8,
}

impl Signature {
    pub fn to_bytes(&self) -> [u8; 65] {
        let mut z = [0; 65];
        let (r, rest) = z.split_at_mut(32);
        let (s, v) = rest.split_at_mut(32);
        r.copy_from_slice(&self.r);
        s.copy_from_slice(&self.s);
        v[0] = self.v;
        z
    }
}

pub struct EthereumSigner<'a> {
    client: &'a KmsGrpcClient,
}

impl<'a> EthereumSigner<'a> {
    pub fn new(client: &'a KmsGrpcClient) -> Self {
        Self { client }
    }

    pub async fn sign(&self, name: &str, digest: &[u8], option: Option<RetrySetting>) -> Result<Signature, Error> {
        let request = asymmetric_sign_request(name, digest.to_vec());
        let result = self.client.asymmetric_sign(request, option.clone()).await?;
        let mut signature = k256::ecdsa::Signature::from_der(&result.signature)?;
        if let Some(new_sig) = signature.normalize_s() {
            signature = new_sig
        }
        let expected_key = self.get_pubkey(name, option).await?;

        for rid in 0..1 {
            let recovery_id = RecoveryId::from_byte(rid).unwrap();
            let recovered_pubkey = VerifyingKey::recover_from_prehash(digest, &signature, recovery_id)?;
            if recovered_pubkey == expected_key {
                return Ok(Signature {
                    r: signature.r().to_bytes().into(),
                    s: signature.s().to_bytes().into(),
                    v: rid,
                });
            }
        }
        return Err(Error::InvalidSignature(result.signature));
    }

    async fn get_pubkey(&self, name: &str, option: Option<RetrySetting>) -> Result<VerifyingKey, Error> {
        let pubkey = self
            .client
            .get_public_key(GetPublicKeyRequest { name: name.to_string() }, option)
            .await?;
        Ok(VerifyingKey::from_public_key_pem(&pubkey.pem)?)
    }
}

fn asymmetric_sign_request(name: &str, digest: Vec<u8>) -> AsymmetricSignRequest {
    AsymmetricSignRequest {
        name: name.to_string(),
        digest: Some(Digest {
            digest: Some(digest::Digest::Sha256(digest)),
        }),
        digest_crc32c: None,
        data: vec![],
        data_crc32c: None,
    }
}

mod tests {
    use crate::client::{Client, ClientConfig};
    use serial_test::serial;
    use crate::ethereum::Error;

    async fn new_client() -> (Client, String) {
        let cred = google_cloud_auth::credentials::CredentialsFile::new().await.unwrap();
        let project = cred.project_id.clone().unwrap();
        let config = ClientConfig::default().with_credentials(cred).await.unwrap();
        (Client::new(config).await.unwrap(), project)
    }

    #[tokio::test]
    #[serial]
    async fn test_sign_ecdsa() {
        use hex_literal::hex;

        let (client, project) = new_client().await;
        let key = format!(
            "projects/{project}/locations/asia-northeast1/keyRings/gcr_test/cryptoKeys/eth-sign/cryptoKeyVersions/1"
        );

        let value = client
            .ethereum()
            .sign(
                &key,
                &hex!("9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"),
                None,
            )
            .await.unwrap();
            println!("{:?}", value.to_bytes());
    }
}
