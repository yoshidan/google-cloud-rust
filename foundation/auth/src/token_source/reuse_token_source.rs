use crate::error::Error;
use crate::token::Token;
use crate::token_source::TokenSource;
use async_trait::async_trait;

#[derive(Debug)]
pub struct ReuseTokenSource {
    target: Box<dyn TokenSource>,
    current_token: std::sync::RwLock<Token>,
    guard: tokio::sync::Mutex<()>,
}

impl ReuseTokenSource {
    pub(crate) fn new(target: Box<dyn TokenSource>, token: Token) -> ReuseTokenSource {
        ReuseTokenSource {
            target,
            current_token: std::sync::RwLock::new(token),
            guard: tokio::sync::Mutex::new(()),
        }
    }
}

#[async_trait]
impl TokenSource for ReuseTokenSource {
    async fn token(&self) -> Result<Token, Error> {
        if let Ok(token) = self.r_lock_token() {
            return Ok(token);
        }

        // Only single task can refresh token
        let _locking = self.guard.lock().await;

        if let Ok(token) = self.r_lock_token() {
            return Ok(token);
        }

        let token = self.target.token().await?;
        tracing::debug!("token refresh success : expiry={:?}", token.expiry);
        *self.current_token.write()? = token.clone();
        Ok(token)
    }
}

impl ReuseTokenSource {
    fn r_lock_token(&self) -> Result<Token, Error> {
        let token = self.current_token.read()?;
        if token.valid() {
            Ok(token.clone())
        } else {
            Err(Error::InvalidToken)
        }
    }
}
