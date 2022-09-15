use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time;
use std::time::Duration;
use crate::error::Error;
use crate::token::Token;
use crate::token_source::TokenSource;
use async_trait::async_trait;
use tokio::task::JoinHandle;
use tokio::time::timeout;

#[derive(Debug)]
pub struct AutoRefreshTokenSource {
    task: JoinHandle<()>,
    current_token: Arc<RwLock<Token>>,
}

impl AutoRefreshTokenSource {
    pub(crate) fn new(target: Box<dyn TokenSource>, token: Token, interval: Duration) -> AutoRefreshTokenSource {
        let current_token = Arc::new(RwLock::new(token));
        let current_token_for_task= current_token.clone();
        let task = tokio::spawn( async move {
            let mut interval = tokio::time::interval(interval);
            loop {
                interval.tick().await;
                match target.token().await {
                    Ok(token) => {
                        *current_token_for_task.write().unwrap() = token;
                    }
                    Err(err) => {
                        panic!("{:?}", err)
                    }
                };
            }
        });
        AutoRefreshTokenSource {
            task,
            current_token
        }
    }
}

#[async_trait]
impl TokenSource for AutoRefreshTokenSource {
    async fn token(&self) -> Result<Token, Error> {
        return Ok(self.current_token.read().unwrap().clone());
    }
}
