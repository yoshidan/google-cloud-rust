use crate::error::Error;
use crate::token::Token;
use crate::token_source::TokenSource;
use async_trait::async_trait;
use std::sync::{Arc, RwLock};


use std::time::Duration;
use tokio::task::JoinHandle;


#[derive(Debug)]
pub struct AutoRefreshTokenSource {
    task: JoinHandle<()>,
    current_token: Arc<RwLock<Token>>,
}

impl AutoRefreshTokenSource {
    pub(crate) fn new(target: Box<dyn TokenSource>, token: Token, interval: Duration) -> AutoRefreshTokenSource {
        let current_token = Arc::new(RwLock::new(token));
        let current_token_for_task = current_token.clone();
        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);
            loop {
                interval.tick().await;
                match target.token().await {
                    Ok(token) => {
                        tracing::debug!("token refresh success : expiry={:?}", token.expiry);
                        *current_token_for_task.write().unwrap() = token;
                    }
                    Err(err) => {
                        tracing::error!("failed to refresh token : {:?}", err);
                    }
                };
            }
        });
        AutoRefreshTokenSource { task, current_token }
    }
}

#[async_trait]
impl TokenSource for AutoRefreshTokenSource {
    async fn token(&self) -> Result<Token, Error> {
        return Ok(self.current_token.read().unwrap().clone());
    }
}
