use crate::ethereum::SignByECError::InvalidSignature;
use crate::grpc::apiv1::kms_client::Client as KmsGrpcClient;
use asn1::{BigInt, ParseError, ParseErrorKind, ParseResult, SimpleAsn1Readable, Tag};
use elliptic_curve::sec1::ToEncodedPoint;
use google_cloud_gax::grpc::Status;
use google_cloud_gax::retry::RetrySetting;
use google_cloud_googleapis::cloud::kms::v1::{digest, AsymmetricSignRequest, Digest};
use hex_literal::hex;
use k256::ecdsa::{RecoveryId, Signature as ECSignature, VerifyingKey};
use once_cell::sync::Lazy;
use primitive_types::U256;
use tiny_keccak::{Hasher, Keccak};

const _SECP256K1_N: [u8; 32] = hex!("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141");

static SECP256K1_N: Lazy<U256> = Lazy::new(|| U256::from(_SECP256K1_N.as_slice()));
static SECP256K1_HALF_N: Lazy<U256> = Lazy::new(|| *SECP256K1_N / 2);

struct U256Bridge<'a> {
    value: U256,
}

impl<'a> U256Bridge<'a> {
    pub fn as_bytes32(&self) -> [u8; 32] {
        let mut b: Vec<u8> = vec![];
        self.value.to_big_endian(&mut b);

        while self.len() < 32 {
            b.insert(0, 0);
        }
        b.into()
    }
}

impl<'a> SimpleAsn1Readable<'a> for U256Bridge<'a> {
    const TAG: Tag = Tag::primitive(0x02);
    fn parse_data(data: &'a [u8]) -> ParseResult<Self> {
        let value = U256::try_from(data).map_err(|_| ParseError::new(ParseErrorKind::InvalidValue))?;
        Ok(Self { value })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SignByECError {
    #[error(transparent)]
    GRPC(#[from] Status),
    #[error(transparent)]
    ParseError(#[from] asn1::ParseError),
    #[error("invalid signature")]
    InvalidSignature(Vec<u8>),
}

pub struct Signature {
    value: [u8; 65],
}

impl AsRef<[u8; 65]> for Signature {
    fn as_ref(&self) -> &[u8; 65] {
        &self.value
    }
}

impl Signature {
    pub fn set_recovery_id(&mut self, recovery_id: u8) {
        self.value[64] = recovery_id;
    }

    pub fn get_recovery_id(&self) -> u8 {
        self.value[64]
    }
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
    ) -> Result<Signature, SignByECError> {
        let result = self
            .client
            .asymmetric_sign(
                AsymmetricSignRequest {
                    name,
                    digest: Some(Digest {
                        digest: Some(digest::Digest::Sha256(digest.clone())),
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
            });
        })?;

        let (mut r, mut s) = signature;
        if s.value < *SECP256K1_HALF_N {
            s.value = *SECP256K1_N - s.value
        }

        for rid in 0..1 {
            let sr = [s.as_bytes32(), r.as_bytes32()].concat();
            let address = self.ecrecover(digest.as_slice(), sr.as_slice(), rid)?;
            //TODO check equality
            return Ok(Signature {
                value: [sr, [rid]].concat().into(),
            });
        }
        return Err(InvalidSignature(result.signature));
    }

    fn ecrecover(&self, digest: &[u8], sr: &[u8], rid: u8) -> Result<[u8; 20], SignByECError> {
        let rid = RecoveryId::from_byte(rid)?;
        let ec_signature = ECSignature::try_from(sr)?;
        let verifying_key = VerifyingKey::recover_from_prehash(digest, &ec_signature, rid)?;
        let point = verifying_key.as_affine().to_encoded_point(false);
        let pubkey = point.to_bytes().try_into()?;
        Ok(Self::kecckac256(&pubkey[1..])[12..].try_into()?)
    }

    fn keccak256(v: &[u8]) -> [u8; 32] {
        let mut k = Keccak::v256();
        k.update(v);

        let mut o = [0u8; 32];
        k.finalize(&mut o);
        o
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
