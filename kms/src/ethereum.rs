use crate::grpc::apiv1::kms_client::Client as KmsGrpcClient;
use asn1::BigInt;
use google_cloud_gax::grpc::Status;
use google_cloud_gax::retry::RetrySetting;
use google_cloud_googleapis::cloud::kms::v1::{digest, AsymmetricSignRequest, Digest};
use hex_literal::hex;
use once_cell::sync::Lazy;

const _SECP256K1N: [u8; 32] = hex!("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141");

static SECP256K1N: Lazy<BigInt> = Lazy::new(|| BigInt::new(_SECP256K1N.as_slice()).unwrap());

#[derive(asn1::Asn1Read, asn1::Asn1Write)]
struct Signature<'a> {
    r: BigInt<'a>,
    s: BigInt<'a>,
}

#[derive(thiserror::Error, Debug)]
pub enum SignByECError {
    #[error(transparent)]
    GRPC(#[from] Status),
    #[error(transparent)]
    ParseError(#[from] asn1::ParseError),
}

pub struct EthereumSigner<'a> {
    client: &'a KmsGrpcClient,
}

impl<'a> EthereumSigner<'a> {
    pub fn new(client: &'a KmsGrpcClient) -> Self {
        Self { client }
    }

    pub async fn sign(
        &self,
        name: String,
        digest: Vec<u8>,
        option: Option<RetrySetting>,
    ) -> Result<Vec<u8>, SignByECError> {
        let result = self
            .client
            .asymmetric_sign(
                AsymmetricSignRequest {
                    name,
                    digest: Some(Digest {
                        digest: Some(digest::Digest::Sha256(digest)),
                    }),
                    digest_crc32c: None,
                    data: vec![],
                    data_crc32c: None,
                },
                option,
            )
            .await?;

        let sig = asn1::parse_single::<Signature>(result.signature.as_slice())?;

        //TODO
        println!("{:?}, {:?}", sig.r, sig.s);
        Ok(vec![])
    }
}

mod tests {
    use crate::client::{Client, ClientConfig};
    use serial_test::serial;

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

        let result = client
            .ethereum()
            .sign(
                key,
                hex!("9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08").to_vec(),
                None,
            )
            .await;
    }
}
