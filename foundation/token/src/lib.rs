
use std::fmt::{Debug};
use std::sync::Arc;
use async_trait::async_trait;

#[async_trait]
pub trait TokenSource: Send + Sync + Debug{
    /// token returns the valid token
    async fn token(&self) -> Result<String, Box<dyn std::error::Error>>;
}

pub trait TokenSourceProvider: Send + Sync + Debug {
    /// token returns the token source implementation
    fn token_source(&self) -> Arc<dyn TokenSource>;
}

#[derive(Debug)]
pub struct NopeTokenSourceProvider {}

impl TokenSourceProvider for NopeTokenSourceProvider {
    fn token_source(&self) -> Arc<dyn TokenSource> {
        panic!("no default token source provider is specified. you can use 'google_cloud_default' crate")
    }
}
