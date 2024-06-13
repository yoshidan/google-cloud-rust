use crate::client::Client;
use ethers_core::k256::ecdsa::RecoveryId;
use ethers_core::k256::elliptic_curve::bigint::{CheckedSub, Encoding};
use ethers_core::k256::elliptic_curve::sec1::ToEncodedPoint;
use ethers_core::k256::elliptic_curve::Curve;
use ethers_core::k256::pkcs8::DecodePublicKey;
use ethers_core::k256::FieldBytes;
use ethers_core::types::{Signature, U256};
use ethers_core::utils::public_key_to_address;
use ethers_core::{
    k256::ecdsa::{Error as K256Error, Signature as KSig, VerifyingKey},
    types::{
        transaction::{eip2718::TypedTransaction, eip712::Eip712},
        Address,
    },
    utils::hash_message,
};
use ethers_signers::Signer as EthSigner;
use google_cloud_gax::grpc::Status;
use google_cloud_gax::retry::RetrySetting;
use google_cloud_googleapis::cloud::kms::v1::{digest, AsymmetricSignRequest, Digest, GetPublicKeyRequest};
use std::fmt::Debug;
use tokio::time::error;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    GRPC(#[from] Status),
    #[error("{0}")]
    K256(#[from] K256Error),
    #[error("{0}")]
    SPKIError(#[from] k256::pkcs8::spki::Error),
    #[error("error encoding eip712 struct: {0:?}")]
    Eip712Error(String),
    #[error("invalid signature: {0:?}")]
    InvalidSignature(Vec<u8>),
}

#[derive(Clone, Debug)]
pub struct Signer {
    client: Client,
    key_name: String,
    pubkey: VerifyingKey,
    address: Address,
    chain_id: u64,
    retry_setting: Option<RetrySetting>,
}

impl Signer {
    pub fn new(
        client: Client,
        key_name: String,
        pubkey: VerifyingKey,
        address: Address,
        chain_id: u64,
        retry_setting: Option<RetrySetting>,
    ) -> Self {
        Self {
            client,
            key_name,
            pubkey,
            address,
            chain_id,
            retry_setting,
        }
    }
    pub async fn create(
        client: Client,
        key_name: &str,
        chain_id: u64,
        retry: Option<RetrySetting>,
    ) -> Result<Self, Error> {
        let pubkey = client
            .get_public_key(
                GetPublicKeyRequest {
                    name: key_name.to_string(),
                },
                retry.clone(),
            )
            .await?;
        let pubkey = VerifyingKey::from_public_key_pem(&pubkey.pem)?;
        let address = public_key_to_address(&pubkey);
        Ok(Self {
            client,
            key_name: key_name.to_string(),
            pubkey,
            address,
            chain_id,
            retry_setting: None,
        })
    }

    pub async fn sign_digest(&self, digest: &[u8]) -> Result<Signature, Error> {
        let request = Self::asymmetric_sign_request(&self.key_name, digest.to_vec());
        let result = self.client.asymmetric_sign(request, self.retry_setting.clone()).await?;
        let mut signature = KSig::from_der(&result.signature)?;
        if let Some(new_sig) = signature.normalize_s() {
            signature = new_sig
        }

        for rid in 0..=1 {
            let recovery_id = RecoveryId::from_byte(rid).unwrap();
            let recovered_pubkey = VerifyingKey::recover_from_prehash(digest, &signature, recovery_id)?;
            let address = public_key_to_address(&recovered_pubkey);
            if recovered_pubkey == self.pubkey {
                let r_bytes: FieldBytes = signature.r().into();
                let s_bytes: FieldBytes = signature.s().into();
                return Ok(Signature {
                    r: U256::from_big_endian(r_bytes.as_slice()),
                    s: U256::from_big_endian(s_bytes.as_slice()),
                    v: rid as u64,
                });
            }
        }
        return Err(Error::InvalidSignature(result.signature));
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

    fn with_eip155(&self, mut signature: Signature) -> Signature {
        signature.v = (self.chain_id * 2 + 35) + signature.v;
        return signature;
    }
}

#[async_trait::async_trait]
impl EthSigner for Signer {
    type Error = Error;

    async fn sign_message<S: Send + Sync + AsRef<[u8]>>(&self, message: S) -> Result<Signature, Self::Error> {
        let message = message.as_ref();
        let message_hash = hash_message(message);
        let signature = self.sign_digest(message_hash.as_bytes()).await?;
        Ok(self.with_eip155(signature))
    }

    async fn sign_transaction(&self, tx: &TypedTransaction) -> Result<Signature, Self::Error> {
        let mut tx_with_chain = tx.clone();
        let chain_id = tx_with_chain.chain_id().map(|id| id.as_u64()).unwrap_or(self.chain_id);
        tx_with_chain.set_chain_id(chain_id);

        let sighash = tx_with_chain.sighash();

        let signature = self.sign_digest(sighash.as_bytes()).await?;
        Ok(self.with_eip155(signature))
    }

    async fn sign_typed_data<T: Eip712 + Send + Sync>(&self, payload: &T) -> Result<Signature, Self::Error> {
        let digest = payload
            .encode_eip712()
            .map_err(|e| Self::Error::Eip712Error(e.to_string()))?;

        let signature = self.sign_digest(&digest).await?;
        Ok(signature)
    }

    fn address(&self) -> Address {
        self.address
    }

    /// Returns the signer's chain id
    fn chain_id(&self) -> u64 {
        self.chain_id
    }

    /// Sets the signer's chain id
    fn with_chain_id<T: Into<u64>>(mut self, chain_id: T) -> Self {
        self.chain_id = chain_id.into();
        self
    }
}

mod tests {
    use crate::client::{Client, ClientConfig};
    use crate::signer::ethereum::Signer;
    use ethers::middleware::SignerMiddleware;
    use ethers::providers::{Http, Middleware, Provider};
    use ethers_core::types::{Address, TransactionRequest};
    use ethers_signers::Signer as EthSigner;
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

        let signer = Signer::create(client, &key, 1, None).await.unwrap();
        let signature = signer
            .sign_digest(&hex!("9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"))
            .await
            .unwrap();
        println!("{:?}", signature);
    }

    #[tokio::test]
    #[serial]
    async fn test_send_ethereum_transaction() {
        let provider = Provider::<Http>::try_from("https://ethereum-sepolia-rpc.publicnode.com").unwrap();

        let amount: u128 = 100_000_000_000_000; // 0.0001 eth

        let (client, project) = new_client().await;
        let key = format!(
            "projects/{project}/locations/asia-northeast1/keyRings/gcr_test/cryptoKeys/eth-sign/cryptoKeyVersions/1"
        );

        let signer = Signer::create(client, &key, 1, None).await.unwrap();
        let signer_address = signer.address();
        tracing::info!("signerAddress = {:?}", signer_address);

        let eth_client = SignerMiddleware::new_with_provider_chain(provider, signer)
            .await
            .unwrap();

        let tx = TransactionRequest::new()
            .to(signer_address)
            .value(amount)
            .gas(8_600_000 as u64)
            .gas_price(40_000_000_000 as u64)
            .chain_id(11155111); // sepolia

        let res = eth_client.send_transaction(tx, None).await.unwrap();
        tracing::info!("tx res: {:?}", res);

        let receipt = res.confirmations(10).await.unwrap();
        tracing::info!("receipt: {:?}", receipt);
    }
}
