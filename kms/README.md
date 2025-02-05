# google-cloud-kms

Google Cloud Platform Key Management Service Client library.

[![crates.io](https://img.shields.io/crates/v/gcloud-kms.svg)](https://crates.io/crates/gcloud-kms)

* [About KMS](https://cloud.google.com/kms/)
* [JSON API Documentation](https://cloud.google.com/kms/docs/reference/rest)
* [RPC Documentation](https://cloud.google.com/kms/docs/reference/rpc)

## Installation

```toml
[dependencies]
google-cloud-kms = { package="gcloud-kms", version="1.0.0" }
```

 ### Authentication
 There are two ways to create a client that is authenticated against the google cloud.

 #### Automatically

 The function `with_auth()` will try and read the credentials from a file specified in the environment variable `GOOGLE_APPLICATION_CREDENTIALS`, `GOOGLE_APPLICATION_CREDENTIALS_JSON` or
 from a metadata server.

 This is also described in [google-cloud-auth](https://github.com/yoshidan/google-cloud-rust/blob/main/foundation/auth/README.md)

 ```rust
 use google_cloud_kms::client::{Client, ClientConfig};

 async fn run() {
     let config = ClientConfig::default().with_auth().await.unwrap();
     let client = Client::new(config);
 }
 ```

 #### Manually

 When you can't use the `gcloud` authentication but you have a different way to get your credentials (e.g a different environment variable)
 you can parse your own version of the 'credentials-file' and use it like that:

 ```rust
 use google_cloud_auth::credentials::CredentialsFile;
 // or google_cloud_kms::client::google_cloud_auth::credentials::CredentialsFile
 use google_cloud_kms::client::{Client, ClientConfig};

 async fn run(cred: CredentialsFile) {
    let config = ClientConfig::default().with_credentials(cred).await.unwrap();
    let client = Client::new(config);
 }
 ```

 ### Usage

 #### Key ring operations

 ```rust
 use std::collections::HashMap;
 use prost_types::FieldMask;
 use google_cloud_googleapis::cloud::kms::v1::{CreateKeyRingRequest, GetKeyRingRequest, ListKeyRingsRequest};
 use google_cloud_kms::client::{Client, ClientConfig};

 async fn run(config: ClientConfig) {

     // Create client
     let client = Client::new(config).await.unwrap();

     // Key ring
     // create
     match client
         .create_key_ring(
             CreateKeyRingRequest {
                 parent: "projects/qovery-gcp-tests/locations/europe-west9".to_string(),
                 key_ring_id: "123-456".to_string(),
                 key_ring: None,
             },
             None,
         )
         .await
     {
         Ok(mut r) => println!("Created key ring {:?}", r),
         Err(err) => panic!("err: {:?}", err),
     };

     // list
     match client
         .list_key_rings(
             ListKeyRingsRequest {
                 parent: "projects/qovery-gcp-tests/locations/europe-west9".to_string(),
                 page_size: 5,
                 page_token: "".to_string(),
                 filter: "".to_string(),
                 order_by: "".to_string(),
             },
             None,
         )
         .await
     {
         Ok(response) => {
             println!("List key rings");
             for r in response.key_rings {
                 println!("- {:?}", r);
             }
         }
         Err(err) => panic!("err: {:?}", err),
     };

     // get
     match client
         .get_key_ring(
             GetKeyRingRequest {
                 name: "projects/qovery-gcp-tests/locations/europe-west9/keyRings/key-ring-for-documentation"
                     .to_string(),
             },
             None,
         )
         .await
     {
         Ok(response) => {
             println!("Get keyring: {:?}", response);
         }
         Err(err) => panic!("err: {:?}", err),
     }
 }
```

### Ethereum Integration

Enable 'eth' feature.

```toml
[dependencies]
google-cloud-kms = { version="version", features=["eth"] }
```

 ```rust
 use ethers::prelude::SignerMiddleware;
 use ethers::providers::{Http, Middleware, Provider};
 use ethers_core::types::{TransactionReceipt, TransactionRequest};
 use ethers_signers::Signer as EthSigner;
 use google_cloud_kms::client::Client;
 use google_cloud_kms::signer::ethereum::{Error, Signer};

 pub async fn send_bnb(client: Client, key_name: &str, rpc_node: &str) {

     // BSC testnet
     let chain_id = 97;

     let signer = Signer::new(client, key_name, chain_id, None).await.unwrap();
     let provider = Provider::<Http>::try_from(rpc_node).unwrap();
 
     let signer_address = signer.address();
     let eth_client = SignerMiddleware::new_with_provider_chain(provider, signer).await.unwrap();

     let tx = TransactionRequest::new()
             .to(signer_address)
             .value(100_000_000_000_000_u128)
             .gas(1_500_000_u64)
             .gas_price(4_000_000_000_u64)
             .chain_id(chain_id); 

     let res = eth_client.send_transaction(tx, None).await.unwrap();
     let receipt: TransactionReceipt = res.confirmations(3).await.unwrap().unwrap();
 }
```