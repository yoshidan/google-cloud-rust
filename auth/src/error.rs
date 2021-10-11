use std::sync::{PoisonError, RwLockReadGuard};
use crate::token::Token;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("error `{0}`")]
    StringError(String),

    #[error(transparent)]
    JsonError(#[from] json::Error),

    #[error(transparent)]
    JwtError(#[from] jwt::errors::Error),

    #[error(transparent)]
    HyperError(#[from] hyper::Error),

    #[error(transparent)]
    HttpError(#[from] hyper::http::Error),

    #[error(transparent)]
    IOError(#[from] std::io::Error),
}
