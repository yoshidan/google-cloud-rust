use crate::error::Error;
use crate::token::Token;
use crate::token_source::TokenSource;
use async_trait::async_trait;

pub struct ReuseTokenSource {
    target: Box<dyn TokenSource>,
    current_token: std::sync::RwLock<Token>,
}

impl ReuseTokenSource {
    pub(crate) fn new(target: Box<dyn TokenSource>, token: Token) -> ReuseTokenSource {
        log::debug!("TokenSource = ReuseTokenSource");
        ReuseTokenSource {
            target,
            current_token: std::sync::RwLock::new(token),
        }
    }
}

#[async_trait]
impl TokenSource for ReuseTokenSource {
    async fn token(&self) -> Result<Token, Error> {
        {
            let r_lock = self.current_token.read().unwrap();
            if r_lock.valid() {
                return Ok(Token {
                    access_token: r_lock.access_token.to_string(),
                    token_type: r_lock.token_type.to_string(),
                    expiry: r_lock.expiry,
                });
            }
        }
        let token = self.target.token().await?;
        *self.current_token.write().unwrap() = token.clone();
        return Ok(token);
    }
}
