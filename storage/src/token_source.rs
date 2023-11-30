use google_cloud_token::{TokenSource, TokenSourceProvider};
use std::error::Error;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AnonymousTokenSource {}

#[async_trait::async_trait]
impl TokenSource for AnonymousTokenSource {
    async fn token(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        Ok("".to_string())
    }
}

#[derive(Debug)]
pub struct AnonymousTokenSourceProvider {}

impl TokenSourceProvider for AnonymousTokenSourceProvider {
    fn token_source(&self) -> Arc<dyn TokenSource> {
        Arc::new(AnonymousTokenSource {})
    }
}
