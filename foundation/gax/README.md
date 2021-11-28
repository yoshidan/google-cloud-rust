# google-cloud-gax

Google Cloud Platform gRPC retry library.

[![crates.io](https://img.shields.io/crates/v/google-cloud-gax.svg)](https://crates.io/crates/google-cloud-gax)

## Installation

```
[dependencies]
google-cloud-gax = 0.2.0
```

## Usage 
```rust
use google_cloud_gax::invoke::invoke_reuse;

pub async fn create_session(
    &mut self,
    req: CreateSessionRequest,
    opt: Option<BackoffRetrySettings>,
) -> Result<Response<Session>, Status> {
    let mut retry_setting = Client::get_call_setting(opt);
    let database = &req.database;
    let token = self.get_token().await?;
    
    // retry gRPC call
    return invoke_reuse(
        |spanner_client| async {
            let request = create_request(format!("database={}", database), &token, req.clone());
            spanner_client
                .create_session(request)
                .await
                .map_err(|e| (e, spanner_client))
        },
        &mut self.spanner_client,
        &mut retry_setting,
    )
    .await;
})
```
