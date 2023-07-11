pub mod aws_subject_token_source;

use crate::error::Error;
use std::fmt::Debug;

#[async_trait]
pub trait SubjectTokenSource: Send + Sync {
    async fn subject_token(&self) -> Result<String, Error>;
}
