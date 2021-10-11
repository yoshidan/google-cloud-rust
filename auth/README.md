# gcpauth

Google Cloud Platform server application authentication library.

[![CI](https://github.com/yoshidan/gcpauth/workflows/CI/badge.svg?branch=main)](https://github.com/yoshidan/gcpauth/workflows/CI)

## Installation

```
[dependencies]
gcpauth = 0.1.2
```
or you can get latest branch.
```
[dependencies]
gcpauth = { git = "https://github.com/yoshidan/gcpauth/", branch = "main"}
```

## Quickstart

```rust
use gcpauth::*;

#[tokio::main]
async fn main() -> Result<(), error::Error> {
    let audience = "https://spanner.googleapis.com/";
    let scopes = [
        "https://www.googleapis.com/auth/cloud-platform",
        "https://www.googleapis.com/auth/spanner.data",
    ];
    let config = Config {
        // audience is required only for service account jwt-auth
        // https://developers.google.com/identity/protocols/oauth2/service-account#jwt-auth
        audience: Some(audience),
        // scopes is required only for service account Oauth2 
        // https://developers.google.com/identity/protocols/oauth2/service-account
        scopes: Some(&scopes) 
    };
    let ts = create_token_source(config).await?;  
    let token = ts.token().await?;
    println!("token is {}",token.access_token);
    Ok(())
}
```

`create_token_source`looks for credentials in the following places,
preferring the first location found:

1. A JSON file whose path is specified by the
   GOOGLE_APPLICATION_CREDENTIALS environment variable.
2. A JSON file in a location known to the gcloud command-line tool.
   On Windows, this is %APPDATA%/gcloud/application_default_credentials.json.
   On other systems, $HOME/.config/gcloud/application_default_credentials.json.
3. On Google Compute Engine, it fetches credentials from the metadata server.

### Async Initialization

```rust
use gcpauth::*;
use tokio::sync::OnceCell;

static AUTHENTICATOR: OnceCell<Box<dyn gcpauth::token::TokenSource>> = OnceCell::const_new();

#[tokio::main]
async fn main() -> Result<(),error::Error> {
    let ts = AUTHENTICATOR.get_or_try_init(|| {
        gcpauth::create_token_source(gcpauth::Config {
            audience: Some("https://spanner.googleapis.com/"),
            scopes: None,
        })
    }).await?;
    let token = ts.token().await?;
    println!("token is {}",token.access_token);
    Ok(())
}
```

## Supported Credentials

- [x] [Service Account(JWT)](https://developers.google.com/identity/protocols/oauth2/service-account#jwt-auth)
- [x] [Service Account(OAuth 2.0)](https://developers.google.com/identity/protocols/oauth2/service-account)
- [x] [Authorized User](https://cloud.google.com/docs/authentication/end-user)
- [ ] [External Account](https://cloud.google.com/anthos/clusters/docs/aws/how-to/workload-identity-gcp?hl=ja)
- [ ] Google Developers Console client_credentials.json

## Supported Workload Identity

https://cloud.google.com/iam/docs/workload-identity-federation

- [ ] AWS
- [ ] Azure Active Directory
- [ ] On-premises Active Directory
- [ ] Okta
- [x] Kubernetes clusters