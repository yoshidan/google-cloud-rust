use crate::error::Error;
use async_trait::async_trait;
use crate::token::Token;


#[async_trait]
pub trait TokenSource: Send + Sync {
    async fn token(&self) -> Result<Token, Error>;
}

