use crate::error::Error;
use crate::token::Token;
use async_trait::async_trait;

#[async_trait]
pub trait TokenSource: Send + Sync {
    async fn token(&self) -> Result<Token, Error>;
}
