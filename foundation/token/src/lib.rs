use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;

#[async_trait]
pub trait TokenSource: Send + Sync + Debug {
    /// token returns the valid token
    async fn token(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}

pub trait TokenSourceProvider: Send + Sync + Debug {
    /// token returns the token source implementation
    fn token_source(&self) -> Arc<dyn TokenSource>;
}

#[derive(Debug)]
pub struct NopeTokenSourceProvider {}

impl TokenSourceProvider for NopeTokenSourceProvider {
    fn token_source(&self) -> Arc<dyn TokenSource> {
        panic!("This is dummy token source provider. you can use 'google_cloud_default' crate")
    }
}
