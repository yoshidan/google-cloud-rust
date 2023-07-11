use std::fmt::Debug;

use async_trait::async_trait;

#[async_trait]
pub trait SubjectTokenSource: Send + Sync + Debug {
    async fn subject_token(&self) -> Result<String, super::Error>;
}
