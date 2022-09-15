use crate::error::Error;
use crate::token::Token;
use crate::token_source::TokenSource;
use async_trait::async_trait;
use std::sync::{Arc, RwLock};

use std::time::Duration;
use tokio::task::JoinHandle;

#[derive(Debug)]
pub struct AutoRefreshTokenSource {
    _task: JoinHandle<()>,
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
        AutoRefreshTokenSource {
            _task: task,
            current_token,
        }
    }
}

#[async_trait]
impl TokenSource for AutoRefreshTokenSource {
    async fn token(&self) -> Result<Token, Error> {
        return Ok(self.current_token.read().unwrap().clone());
    }
}

#[cfg(test)]
mod test {
    use crate::error::Error;
    use crate::token::Token;
    use crate::{AutoRefreshTokenSource, TokenSource};
    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use std::fmt::Debug;

    use std::sync::Arc;

    use std::time::Duration;

    use tracing_subscriber::filter::LevelFilter;

    #[derive(Debug)]
    struct EmptyTokenSource {
        pub expiry: DateTime<Utc>,
    }
    #[async_trait]
    impl TokenSource for EmptyTokenSource {
        async fn token(&self) -> Result<Token, Error> {
            Ok(Token {
                access_token: "empty".to_string(),
                token_type: "empty".to_string(),
                expiry: Some(self.expiry),
            })
        }
    }

    #[ctor::ctor]
    fn init() {
        let filter = tracing_subscriber::filter::EnvFilter::from_default_env().add_directive(LevelFilter::DEBUG.into());
        let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
    }

    #[tokio::test]
    async fn test_with_refresh() {
        let mut ts = Box::new(EmptyTokenSource { expiry: Utc::now() });
        let token = ts.token().await.unwrap();

        ts.expiry = Utc::now() + chrono::Duration::seconds(100);
        let results = run_task(ts, token).await;
        for v in results {
            assert!(v)
        }
    }

    async fn run_task(ts: Box<EmptyTokenSource>, first_token: Token) -> Vec<bool> {
        let ts = Arc::new(AutoRefreshTokenSource::new(ts, first_token, Duration::from_millis(1)));
        let mut tasks = Vec::with_capacity(100);
        for _n in 1..100 {
            let ts_clone = ts.clone();
            tokio::time::sleep(Duration::from_millis(1)).await;
            let task = tokio::spawn(async move {
                match ts_clone.token().await {
                    Ok(new_token) => new_token.valid(),
                    Err(_e) => false,
                }
            });
            tasks.push(task)
        }
        let mut result = Vec::with_capacity(tasks.len());
        for task in tasks {
            result.push(task.await.unwrap());
        }
        result
    }
}
