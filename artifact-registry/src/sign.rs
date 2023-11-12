use std::fmt::{Debug, Formatter};

pub trait URLStyle {
    fn host(&self, bucket: &str) -> String;
    fn path(&self, bucket: &str, object: &str) -> String;
}

#[derive(Clone)]
pub enum SignBy {
    PrivateKey(Vec<u8>),
    SignBytes,
}

impl Debug for SignBy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SignBy::PrivateKey(_) => f.write_str("private_key"),
            SignBy::SignBytes => f.write_str("sign_bytes"),
        }
    }
}
