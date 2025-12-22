pub mod credentials;
pub mod error;
pub mod idtoken;
mod misc;
pub mod project;
pub mod token;
pub mod token_source;

#[cfg(all(feature = "jwt-aws-lc-rs", feature = "jwt-rust-crypto"))]
compile_error!("Enable only one feature: `jwt-aws-lc-rs` OR `jwt-rust-crypto`.");

#[cfg(not(any(feature = "jwt-aws-lc-rs", feature = "jwt-rust-crypto")))]
compile_error!("Enable one feature: `jwt-aws-lc-rs` OR `jwt-rust-crypto`.");
