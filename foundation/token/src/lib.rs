use std::error::Error;
use std::fmt::{Debug, Display};
use std::result::Iter;

pub trait TokenSourceError: Display + Debug + Sync + Send {}

pub trait TokenSource: Send + Sync + Debug {
    /// token returns the valid token
    async fn token(&self) -> Result<String, Box<dyn Error>>;
}

pub trait TokenSourceProvider: Send + Sync + Debug {
    /// token returns the token source implementation
    fn token_source(&self) -> dyn TokenSource;
}

#[derive(Debug)]
pub struct NopeTokenSourceProvider {}

impl TokenSourceProvider for NopeTokenSourceProvider {
    fn token_source(&self) -> Box<dyn TokenSource> {
        panic!("no default token source provider is specified. you can use 'google_cloud_default' crate")
    }
}
