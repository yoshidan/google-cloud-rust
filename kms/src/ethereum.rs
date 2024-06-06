use crate::grpc::apiv1::kms_client::Client as KmsGrpcClient;
use asn1::{BigInt, ParseError, ParseErrorKind, ParseResult, SimpleAsn1Readable, Tag};
use google_cloud_gax::grpc::Status;
use google_cloud_gax::retry::RetrySetting;
use google_cloud_googleapis::cloud::kms::v1::{digest, AsymmetricSignRequest, Digest};
use hex_literal::hex;
use once_cell::sync::Lazy;
use primitive_types::U256;

const _SECP256K1_N: [u8; 32] = hex!("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141");

static SECP256K1_N: Lazy<U256> = Lazy::new(|| U256::from(_SECP256K1_N.as_slice()));
static SECP256K1_HALF_N: Lazy<U256> = Lazy::new(|| *SECP256K1_N / 2);

struct U256Bridge<'a> {
    value: U256,
}

struct Signature {
    r: U256,
    s: U256,
}

impl<'a> SimpleAsn1Readable<'a> for U256Bridge<'a> {
    const TAG: Tag = Tag::primitive(0x02);
    fn parse_data(data: &'a [u8]) -> ParseResult<Self> {
        let value= U256::try_from(data).map_err(|_| ParseError::new(ParseErrorKind::InvalidValue))?;
        Ok(Self {
            value,
            bytes: data
        })
    }
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

        let mut signature = asn1::parse(result.signature.as_slice(), |d| {
            return d.read_element::<asn1::Sequence>()?.parse(|d| {
                let r = d.read_element::<U256Bridge>()?;
                let s = d.read_element::<U256Bridge>()?;
                Ok((r, s))
            })
        })?;
        let (r,s) = signature;
        let s = if s.value < *SECP256K1_HALF_N {
            *SECP256K1_N - s.value
        }else {
            s.value
        };
        let mut s_bytes: Vec<u8> = vec![];
        s.to_big_endian(&mut s_bytes);

        let mut r_bytes: Vec<u8> = vec![];
        r.to_big_endian(&mut r_bytes);
        while s_bytes.len() < 32 {
            s_bytes.insert(0, 0);
        }
        while r_bytes.len() < 32 {
            r_bytes.insert(0, 0);
        }

        //TODO generate recovery_id

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
