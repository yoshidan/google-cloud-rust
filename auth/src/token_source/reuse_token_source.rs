use crate::error::Error;
use crate::token::{Token, TokenSource};
use async_trait::async_trait;

pub struct ReuseTokenSource {
    pub target: Box<dyn TokenSource>,
    pub current_token: std::sync::RwLock<Token>,
}

impl ReuseTokenSource {
    pub fn new(target: Box<dyn TokenSource>, token: Token) -> ReuseTokenSource {
        return ReuseTokenSource {
            target,
            current_token: std::sync::RwLock::new(token),
        };
    }
}

#[async_trait]
impl TokenSource for ReuseTokenSource {
    async fn token(&self) -> Result<Token, Error> {
        let token = self.current_token.read().unwrap().clone();
        if token.valid() {
            return Ok(token);
        }
        let token = self.target.token().await?;
        *self.current_token.write().unwrap() = token.clone();
        return Ok(token);
    }
}
