# google-cloud-auth

Google Cloud Platform server application authentication library.

[![crates.io](https://img.shields.io/crates/v/gloud-auth.svg)](https://crates.io/crates/gcloud-auth)

## Installation

```toml
[dependencies]
google-cloud-auth = { package="gcloud-auth", version="1.0.0" }
```

## Quickstart

```rust
#[tokio::main]
async fn main() -> Result<(), error::Error> {
    use google_cloud_auth::{project::Config, token::DefaultTokenSourceProvider};
    // This https://crates.io/crates/token-source
    use token_source::TokenSourceProvider as _;

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
        scopes: Some(&scopes),
        sub: None,
    };
    let tsp = DefaultTokenSourceProvider::new(config).await?;
    let ts = tsp.token_source();
    let token = ts.token().await?;
    println!("token is {}", token);
    Ok(())
}
```

`DefaultTokenSourceProvider::new(config)` looks for credentials in the following places,
preferring the first location found:

1. A JSON file whose path is specified by the
   GOOGLE_APPLICATION_CREDENTIALS environment variable.
2. A JSON file in a location known to the gcloud command-line tool.
   On Windows, this is %APPDATA%/gcloud/application_default_credentials.json.
   On other systems, $HOME/.config/gcloud/application_default_credentials.json.
3. On Google Compute Engine, it fetches credentials from the metadata server.

## Supported Credentials

- [x] [Service Account(JWT)](https://developers.google.com/identity/protocols/oauth2/service-account#jwt-auth)
- [x] [Service Account(OAuth 2.0)](https://developers.google.com/identity/protocols/oauth2/service-account)
- [x] [Authorized User](https://cloud.google.com/docs/authentication/end-user)
- [ ] [External Account](https://cloud.google.com/anthos/clusters/docs/aws/how-to/workload-identity-gcp)
- [ ] Google Developers Console client_credentials.json

## Supported Workload Identity

<https://cloud.google.com/iam/docs/workload-identity-federation>

- [x] AWS
- [ ] Azure Active Directory
- [ ] On-premises Active Directory
- [ ] Okta
- [x] Kubernetes clusters
