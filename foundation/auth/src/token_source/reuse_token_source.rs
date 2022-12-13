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
        if let Some(token) = self.r_lock_token() {
            return Ok(token);
        }

        // Only single task can refresh token
        let _locking = self.guard.lock().await;

        if let Some(token) = self.r_lock_token() {
            return Ok(token);
        }

        let token = self.target.token().await?;
        tracing::debug!("token refresh success : expiry={:?}", token.expiry);
        *self.current_token.write().unwrap() = token.clone();
        Ok(token)
    }
}

impl ReuseTokenSource {
    fn r_lock_token(&self) -> Option<Token> {
        let token = self.current_token.read().unwrap();
        if token.valid() {
            Some(token.clone())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use crate::error::Error;
    use crate::token::Token;
    use crate::{ReuseTokenSource, TokenSource};
    use async_trait::async_trait;
    use std::fmt::Debug;
    use time::OffsetDateTime;

    use std::sync::Arc;

    use tracing_subscriber::filter::LevelFilter;

    #[derive(Debug)]
    struct EmptyTokenSource {
        pub expiry: OffsetDateTime,
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
    async fn test_all_valid() {
        let ts = Box::new(EmptyTokenSource {
            expiry: OffsetDateTime::now_utc() + time::Duration::seconds(100),
        });
        let token = ts.token().await.unwrap();
        let results = run_task(ts, token).await;
        for v in results {
            assert!(v)
        }
    }

    #[tokio::test]
    async fn test_with_invalid() {
        let mut ts = Box::new(EmptyTokenSource {
            expiry: OffsetDateTime::now_utc(),
        });
        let token = ts.token().await.unwrap();
        ts.expiry = OffsetDateTime::now_utc() + time::Duration::seconds(100);
        let results = run_task(ts, token).await;
        for v in results {
            assert!(v)
        }
    }

    #[tokio::test]
    async fn test_all_invalid() {
        let ts = Box::new(EmptyTokenSource {
            expiry: OffsetDateTime::now_utc(),
        });
        let token = ts.token().await.unwrap();
        let results = run_task(ts, token).await;
        for v in results {
            assert!(!v)
        }
    }

    async fn run_task(ts: Box<EmptyTokenSource>, first_token: Token) -> Vec<bool> {
        let ts = Arc::new(ReuseTokenSource::new(ts, first_token));
        let mut tasks = Vec::with_capacity(100);
        for _n in 1..100 {
            let ts_clone = ts.clone();
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
